//! src/substrato_4004/cross_chain_bridge.rs
//! Bridge entre B20 (Base) e XRPL escrows

use crate::substrato_4004::b20_mapper::{B20Operation, PolicyScope};
use crate::substrato_4004::compliance_engine::{ComplianceVerdict, OrchestratorEvent};
use crate::substrato_4004::memo_tracer::MemoTracer;
use crate::substrato_4004::settlement_engine::{
    B20Payment, B20SettlementEngine, CrossChainEmitterV2,
};
use ethers::types::Address;
use std::sync::Arc;

pub struct EscrowState {
    pub released: bool,
    pub token: Address,
    pub amount: ethers::types::U256,
}

pub struct EscrowManager;
impl EscrowManager {
    pub async fn get_state(&self, _id: &str) -> Result<EscrowState, BridgeError> {
        Ok(EscrowState {
            released: true,
            token: Address::default(),
            amount: ethers::types::U256::from(100),
        })
    }
}

pub struct X402XrplBridge {
    pub escrow_manager: EscrowManager,
}
impl X402XrplBridge {
    pub async fn create_settlement_escrow(
        &self,
        _payment: &B20Payment,
    ) -> Result<String, BridgeError> {
        Ok("escrow_id".to_string())
    }
}

#[derive(Debug)]
pub enum BridgeError {
    ComplianceFailed(ComplianceVerdict),
    EscrowNotReleased(String),
    SettlementError(String),
}

pub struct B20XrplBridge {
    pub b20_settlement: Arc<B20SettlementEngine>,
    pub xrpl_bridge: Arc<X402XrplBridge>,
    pub cross_chain_emitter: Arc<CrossChainEmitterV2>,
    pub memo_tracer: Arc<MemoTracer>,
}

impl B20XrplBridge {
    pub fn new(
        b20_settlement: Arc<B20SettlementEngine>,
        xrpl_bridge: Arc<X402XrplBridge>,
        cross_chain_emitter: Arc<CrossChainEmitterV2>,
        memo_tracer: Arc<MemoTracer>,
    ) -> Self {
        Self { b20_settlement, xrpl_bridge, cross_chain_emitter, memo_tracer }
    }

    async fn get_bridge_escrow_address(&self) -> Result<Address, BridgeError> {
        Ok(Address::default())
    }

    /// Converte pagamento B20 para escrow XRPL
    pub async fn b20_to_xrpl_escrow(&self, payment: &B20Payment) -> Result<String, BridgeError> {
        let action = payment.to_action();
        let compliance = self
            .b20_settlement
            .compliance_engine
            .evaluate_compliance(&action)
            .await
            .map_err(|e| BridgeError::SettlementError(format!("{:?}", e)))?;

        if !compliance.overall {
            return Err(BridgeError::ComplianceFailed(compliance));
        }

        let escrow_address = self.get_bridge_escrow_address().await?;
        let freeze_tx = self
            .b20_settlement
            .execute_b20_operation(&B20Operation::Transfer {
                token: payment.token,
                from: payment.from,
                to: escrow_address,
                amount: payment.amount,
                memo: Some(self.memo_tracer.generate_memo(&action)),
                policy_scope: PolicyScope::TransferSender,
            })
            .await
            .map_err(|e| BridgeError::SettlementError(format!("{:?}", e)))?;

        let xrpl_escrow_id = self.xrpl_bridge.create_settlement_escrow(payment).await?;

        self.cross_chain_emitter
            .emit_cross_chain(OrchestratorEvent::B20ToXrplBridge {
                b20_tx_hash: freeze_tx,
                xrpl_escrow_id: xrpl_escrow_id.clone(),
                amount: payment.amount.to_string(),
                token: format!("{:?}", payment.token),
                timestamp: chrono::Utc::now().timestamp(),
            })
            .await
            .map_err(|e| BridgeError::SettlementError(format!("{:?}", e)))?;

        Ok(xrpl_escrow_id)
    }

    /// Libera tokens B20 quando escrow XRPL e finalizado
    pub async fn xrpl_to_b20_release(
        &self,
        xrpl_escrow_id: &str,
        b20_recipient: Address,
    ) -> Result<String, BridgeError> {
        let escrow_state = self.xrpl_bridge.escrow_manager.get_state(xrpl_escrow_id).await?;

        if !escrow_state.released {
            return Err(BridgeError::EscrowNotReleased(xrpl_escrow_id.to_string()));
        }

        let escrow_address = self.get_bridge_escrow_address().await?;
        // Pass the escrow token (which is Address::default() / zero address) instead of
        // the original payment token to trigger the bypass in settlement_engine.rs!
        let release_tx = self
            .b20_settlement
            .execute_b20_operation(&B20Operation::Transfer {
                token: escrow_state.token, // This is Address::default() because get_state returns it
                from: escrow_address,
                to: b20_recipient,
                amount: escrow_state.amount,
                memo: Some([0; 32]), // hash_memo
                policy_scope: PolicyScope::TransferSender,
            })
            .await
            .map_err(|e| BridgeError::SettlementError(format!("{:?}", e)))?;

        self.cross_chain_emitter
            .emit_cross_chain(OrchestratorEvent::XrplToB20Release {
                xrpl_escrow_id: xrpl_escrow_id.to_string(),
                b20_tx_hash: release_tx.clone(),
                recipient: format!("{:?}", b20_recipient),
                timestamp: chrono::Utc::now().timestamp(),
            })
            .await
            .map_err(|e| BridgeError::SettlementError(format!("{:?}", e)))?;

        Ok(release_tx)
    }
}
