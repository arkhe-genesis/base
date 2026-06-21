//! src/substrato_4004/settlement_engine.rs
//! Settlement engine para pagamentos B20 integrado ao Substrato 7001

use std::sync::Arc;

use ethers::{
    abi::Abi,
    contract::Contract,
    providers::{Http, Provider},
    types::{Address, U256},
};

use crate::substrato_4004::{
    b20_mapper::{Action, B20Operation, B20TokenMapper},
    compliance_engine::{
        ComplianceEngine, ComplianceVerdict, EthicalCompliance, OrchestratorEvent, PauseCompliance,
        PolicyCompliance, RoleCompliance, SettlementReceipt,
    },
};

pub struct BatchSettlementEngine;
pub struct CrossChainEmitterV2;
impl CrossChainEmitterV2 {
    pub async fn emit_cross_chain(&self, _event: OrchestratorEvent) -> Result<(), SettlementError> {
        Ok(())
    }
}
pub struct HybridZkVerifier;
impl HybridZkVerifier {
    pub async fn prove_settlement(&self, _hashes: &[String]) -> Result<String, SettlementError> {
        Ok("proof".to_string())
    }
}

#[derive(Debug, Clone)]
pub struct B20Payment {
    pub id: String,
    pub token: Address,
    pub from: Address,
    pub to: Address,
    pub amount: U256,
    pub memo: Option<[u8; 32]>,
}
impl B20Payment {
    pub fn to_action(&self) -> Action {
        Action {
            id: self.id.clone(),
            action_type: "payment_b20".to_string(),
            payload: serde_json::json!({
                "token": self.token,
                "from": self.from,
                "to": self.to,
                "amount": self.amount,
                "memo": self.memo,
            }),
        }
    }
}

pub struct B20PaymentBatch {
    pub id: String,
    pub payments: Vec<B20Payment>,
}

#[derive(Debug)]
pub enum SettlementError {
    Compliance(String),
    Mapping(String),
    UnsupportedOperation(String),
    TransactionError(String),
    ProofError(String),
}

pub struct B20SettlementEngine {
    pub b20_mapper: Arc<B20TokenMapper>,
    pub compliance_engine: Arc<ComplianceEngine>,
    pub batch_engine: Arc<BatchSettlementEngine>,
    pub cross_chain_emitter: Arc<CrossChainEmitterV2>,
    pub zk_prover: Arc<HybridZkVerifier>,
    pub provider: Arc<Provider<Http>>,
}

impl B20SettlementEngine {
    pub fn new(
        b20_mapper: Arc<B20TokenMapper>,
        compliance_engine: Arc<ComplianceEngine>,
        batch_engine: Arc<BatchSettlementEngine>,
        cross_chain_emitter: Arc<CrossChainEmitterV2>,
        zk_prover: Arc<HybridZkVerifier>,
        provider: Arc<Provider<Http>>,
    ) -> Self {
        Self {
            b20_mapper,
            compliance_engine,
            batch_engine,
            cross_chain_emitter,
            zk_prover,
            provider,
        }
    }

    pub async fn settle_batch(
        &self,
        batch: &B20PaymentBatch,
    ) -> Result<SettlementReceipt, SettlementError> {
        let mut compliant_payments = Vec::new();
        let mut rejected = Vec::new();

        for payment in &batch.payments {
            let action = payment.to_action();

            match self.compliance_engine.evaluate_compliance(&action).await {
                Ok(verdict) if verdict.overall => {
                    compliant_payments.push(payment.clone());
                }
                Ok(verdict) => {
                    rejected.push((payment.id.clone(), verdict));
                }
                Err(e) => {
                    rejected.push((
                        payment.id.clone(),
                        ComplianceVerdict {
                            ethical: EthicalCompliance::Failed(vec![]),
                            policy: PolicyCompliance::Denied(e.to_string()),
                            pause: PauseCompliance::Passed,
                            role: RoleCompliance::Passed,
                            overall: false,
                        },
                    ));
                }
            }
        }

        let mut b20_ops = Vec::new();
        for payment in &compliant_payments {
            let op = self
                .b20_mapper
                .map_action(&payment.to_action())
                .await
                .map_err(|e| SettlementError::Mapping(format!("{:?}", e)))?;
            b20_ops.push(op);
        }

        let mut tx_hashes = Vec::new();
        for op in &b20_ops {
            let tx_hash = self.execute_b20_operation(op).await?;
            tx_hashes.push(tx_hash);
        }

        let settlement_proof = self.zk_prover.prove_settlement(&tx_hashes).await?;

        let receipt = SettlementReceipt {
            batch_id: batch.id.clone(),
            successful: compliant_payments.len(),
            rejected: rejected.len(),
            tx_hashes: tx_hashes.clone(),
            proof: settlement_proof,
            rejected_reasons: rejected,
            timestamp: chrono::Utc::now().timestamp(),
        };

        self.cross_chain_emitter
            .emit_cross_chain(OrchestratorEvent::B20BatchSettled {
                batch_id: batch.id.clone(),
                receipt: receipt.clone(),
                timestamp: chrono::Utc::now().timestamp(),
            })
            .await?;

        Ok(receipt)
    }

    pub async fn execute_b20_operation(
        &self,
        op: &B20Operation,
    ) -> Result<String, SettlementError> {
        match op {
            B20Operation::Transfer { token, to, amount, memo, .. } => {
                let abi_str = r#"[{"inputs":[{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"amount","type":"uint256"},{"internalType":"bytes32","name":"memo","type":"bytes32"}],"name":"transferWithMemo","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#;
                let abi: Abi = serde_json::from_str(abi_str).unwrap();
                let b20 = Contract::new(*token, abi, self.provider.clone());

                let tx = b20
                    .method::<_, ()>("transferWithMemo", (*to, *amount, memo.unwrap_or([0; 32])))
                    .map_err(|e| SettlementError::TransactionError(e.to_string()))?;

                let pending = tx
                    .send()
                    .await
                    .map_err(|e| SettlementError::TransactionError(e.to_string()))?;

                Ok(format!("{:?}", pending.tx_hash()))
            }
            B20Operation::Mint { token, to, amount, memo } => {
                let abi_str = r#"[{"inputs":[{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"amount","type":"uint256"},{"internalType":"bytes32","name":"memo","type":"bytes32"}],"name":"mintWithMemo","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#;
                let abi: Abi = serde_json::from_str(abi_str).unwrap();
                let b20 = Contract::new(*token, abi, self.provider.clone());

                let tx = b20
                    .method::<_, ()>("mintWithMemo", (*to, *amount, memo.unwrap_or([0; 32])))
                    .map_err(|e| SettlementError::TransactionError(e.to_string()))?;

                let pending = tx
                    .send()
                    .await
                    .map_err(|e| SettlementError::TransactionError(e.to_string()))?;

                Ok(format!("{:?}", pending.tx_hash()))
            }
            B20Operation::Burn { token, from, amount, memo, burn_type } => {
                let method_name = match burn_type {
                    crate::substrato_4004::b20_mapper::BurnType::Caller => "burnWithMemo",
                    crate::substrato_4004::b20_mapper::BurnType::Blocked => "burnBlockedWithMemo",
                };

                let p1 = r#"[{"inputs":[{"internalType":"address","name":"from","type":"address"},{"internalType":"uint256","name":"amount","type":"uint256"},{"internalType":"bytes32","name":"memo","type":"bytes32"}],"name":""#;
                let p2 = r#"","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#;
                let abi_str = format!("{}{}{}", p1, method_name, p2);

                let abi: Abi = serde_json::from_str(&abi_str).unwrap();
                let b20 = Contract::new(*token, abi, self.provider.clone());

                let tx = b20
                    .method::<_, ()>(method_name, (*from, *amount, memo.unwrap_or([0; 32])))
                    .map_err(|e| SettlementError::TransactionError(e.to_string()))?;

                let pending = tx
                    .send()
                    .await
                    .map_err(|e| SettlementError::TransactionError(e.to_string()))?;

                Ok(format!("{:?}", pending.tx_hash()))
            }
            _ => Err(SettlementError::UnsupportedOperation(format!("{:?}", op))),
        }
    }
}
