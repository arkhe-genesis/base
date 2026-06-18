//! src/substrato_4004/b20_mapper.rs
//! Mapeia Actions do Cathedral para operacoes B20

use ethers::types::{Address, U256, Bytes};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::substrato_4004::policy_adapter::PolicyAdapter;
use std::str::FromStr;

/// Representacao dummy de Action e afins para compilar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub action_type: String,
    pub payload: serde_json::Value,
}

impl Action {
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self.payload).unwrap_or_default()
    }
}

pub struct EthicalFilter;
pub enum FilterVerdict { Passed, Failed(Vec<String>) }
impl EthicalFilter {
    pub async fn evaluate(&self, _action: &Action) -> FilterVerdict { FilterVerdict::Passed }
}

#[derive(Debug)]
pub enum MapperError {
    EthicalViolation(Vec<String>),
    PolicyDenied(String),
    SupplyCapExceeded,
    NotBlocked(Address),
    UnsupportedActionType(String),
    ExtractionError(String),
}

/// Operacao B20 mapeada a partir de uma Cathedral Action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum B20Operation {
    Transfer {
        token: Address,
        from: Address,
        to: Address,
        amount: U256,
        memo: Option<[u8; 32]>,
        policy_scope: PolicyScope,
    },
    Mint {
        token: Address,
        to: Address,
        amount: U256,
        memo: Option<[u8; 32]>,
    },
    Burn {
        token: Address,
        from: Address,
        amount: U256,
        memo: Option<[u8; 32]>,
        burn_type: BurnType,
    },
    UpdatePolicy {
        token: Address,
        scope: PolicyScope,
        policy_id: u64,
    },
    Pause {
        token: Address,
        features: Vec<PausableFeature>,
        pause: bool,
    },
    UpdateMultiplier {
        token: Address,
        new_multiplier: U256, // WAD precision
    },
    Announce {
        token: Address,
        internal_calls: Vec<Bytes>,
        id: u64,
        description: String,
        uri: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyScope {
    TransferSender,
    TransferReceiver,
    TransferExecutor,
    MintReceiver,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BurnType {
    Caller,      // burn proprio
    Blocked,     // burnBlocked (freeze-and-seize)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PausableFeature {
    Transfer,
    Mint,
    Burn,
}

pub struct B20TokenMapper {
    ethical_filter: Arc<EthicalFilter>,
    policy_registry: Arc<PolicyAdapter>,
}

fn extract_address(action: &Action, field: &str) -> Result<Address, MapperError> {
    if let Some(s) = action.payload.get(field).and_then(|v| v.as_str()) {
        Address::from_str(s).map_err(|_| MapperError::ExtractionError("Invalid address".to_string()))
    } else {
        Ok(Address::default())
    }
}
fn extract_u256(_action: &Action, _field: &str) -> Result<U256, MapperError> { Ok(U256::default()) }
fn extract_optional_memo(_action: &Action) -> Result<Option<[u8; 32]>, MapperError> { Ok(None) }
fn hash_memo(_prefix: &str, _action: &Action) -> [u8; 32] { [0; 32] }
fn extract_policy_scope(_action: &Action) -> Result<PolicyScope, MapperError> { Ok(PolicyScope::TransferSender) }
fn extract_u64(_action: &Action, _field: &str) -> Result<u64, MapperError> { Ok(0) }
fn extract_pausable_features(_action: &Action) -> Result<Vec<PausableFeature>, MapperError> { Ok(vec![]) }

impl B20TokenMapper {
    pub fn new(ethical_filter: Arc<EthicalFilter>, policy_registry: Arc<PolicyAdapter>) -> Self {
        Self { ethical_filter, policy_registry }
    }

    async fn get_total_supply(&self, _token: Address) -> Result<U256, MapperError> { Ok(U256::default()) }
    async fn get_supply_cap(&self, _token: Address) -> Result<U256, MapperError> { Ok(U256::MAX) }

    pub async fn map_action(&self, action: &Action) -> Result<B20Operation, MapperError> {
        match self.ethical_filter.evaluate(action).await {
            FilterVerdict::Passed => {}
            FilterVerdict::Failed(v) => return Err(MapperError::EthicalViolation(v)),
        }

        match action.action_type.as_str() {
            "payment_b20" => {
                let token = extract_address(action, "token")?;
                let from = extract_address(action, "from")?;
                let to = extract_address(action, "to")?;
                let amount = extract_u256(action, "amount")?;
                let memo = extract_optional_memo(action)?;

                let sender_policy = self.policy_registry
                    .get_policy(token, PolicyScope::TransferSender)
                    .await.map_err(|e| MapperError::ExtractionError(format!("{:?}", e)))?;

                if !self.policy_registry.is_authorized(sender_policy, from).await.map_err(|e| MapperError::ExtractionError(format!("{:?}", e)))? {
                    return Err(MapperError::PolicyDenied("sender".to_string()));
                }

                Ok(B20Operation::Transfer {
                    token,
                    from,
                    to,
                    amount,
                    memo,
                    policy_scope: PolicyScope::TransferSender,
                })
            }
            "mint_b20" => {
                let token = extract_address(action, "token")?;
                let to = extract_address(action, "to")?;
                let amount = extract_u256(action, "amount")?;
                let memo = extract_optional_memo(action)?;

                let current_supply = self.get_total_supply(token).await?;
                let cap = self.get_supply_cap(token).await?;

                if current_supply + amount > cap {
                    return Err(MapperError::SupplyCapExceeded);
                }

                Ok(B20Operation::Mint { token, to, amount, memo })
            }
            "freeze_and_seize" => {
                let token = extract_address(action, "token")?;
                let target = extract_address(action, "target")?;
                let amount = extract_u256(action, "amount")?;

                let sender_policy = self.policy_registry
                    .get_policy(token, PolicyScope::TransferSender)
                    .await.map_err(|e| MapperError::ExtractionError(format!("{:?}", e)))?;

                if self.policy_registry.is_authorized(sender_policy, target).await.map_err(|e| MapperError::ExtractionError(format!("{:?}", e)))? {
                    return Err(MapperError::NotBlocked(target));
                }

                Ok(B20Operation::Burn {
                    token,
                    from: target,
                    amount,
                    memo: Some(hash_memo("freeze-and-seize", action)),
                    burn_type: BurnType::Blocked,
                })
            }
            "update_policy" => {
                let token = extract_address(action, "token")?;
                let scope = extract_policy_scope(action)?;
                let policy_id = extract_u64(action, "policy_id")?;

                Ok(B20Operation::UpdatePolicy { token, scope, policy_id })
            }
            "pause_b20" => {
                let token = extract_address(action, "token")?;
                let features = extract_pausable_features(action)?;

                Ok(B20Operation::Pause { token, features, pause: true })
            }
            "unpause_b20" => {
                let token = extract_address(action, "token")?;
                let features = extract_pausable_features(action)?;

                Ok(B20Operation::Pause { token, features, pause: false })
            }
            "update_multiplier" => {
                let token = extract_address(action, "token")?;
                let multiplier = extract_u256(action, "multiplier")?;

                Ok(B20Operation::UpdateMultiplier { token, new_multiplier: multiplier })
            }
            _ => Err(MapperError::UnsupportedActionType(action.action_type.clone())),
        }
    }
}
