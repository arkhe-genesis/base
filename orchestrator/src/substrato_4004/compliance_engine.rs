//! src/substrato_4004/compliance_engine.rs
//! Engine de compliance que integra EthicalFilter com B20 policies

use std::sync::Arc;
use ethers::types::Address;
use ethers::providers::{Provider, Http};
use crate::substrato_4004::b20_mapper::{
    Action, B20TokenMapper, B20Operation, EthicalFilter, FilterVerdict, MapperError, PausableFeature, PolicyScope, BurnType
};
use crate::substrato_4004::policy_adapter::PolicyAdapter;

pub struct EventStore;
impl EventStore {
    pub async fn emit(&self, _event: OrchestratorEvent) -> Result<(), ComplianceError> { Ok(()) }
}

pub enum OrchestratorEvent {
    ComplianceChecked { action_id: String, verdict: ComplianceVerdict, timestamp: i64 },
    B20BatchSettled { batch_id: String, receipt: SettlementReceipt, timestamp: i64 },
    B20Memo { tx_hash: String, log_index: u64, caller: String, memo: String, timestamp: i64 },
    ActionProposed { action: Action, timestamp: i64 },
    B20ToXrplBridge { b20_tx_hash: String, xrpl_escrow_id: String, amount: String, token: String, timestamp: i64 },
    XrplToB20Release { xrpl_escrow_id: String, b20_tx_hash: String, recipient: String, timestamp: i64 },
}

#[derive(Debug, Clone)]
pub struct SettlementReceipt {
    pub batch_id: String,
    pub successful: usize,
    pub rejected: usize,
    pub tx_hashes: Vec<String>,
    pub proof: String,
    pub rejected_reasons: Vec<(String, ComplianceVerdict)>,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ComplianceError {
    Mapping(MapperError),
    Policy(String),
    EventEmit(String),
}

impl std::fmt::Display for ComplianceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct ComplianceEngine {
    ethical_filter: Arc<EthicalFilter>,
    policy_registry: Arc<PolicyAdapter>,
    b20_mapper: Arc<B20TokenMapper>,
    event_store: Arc<EventStore>,
    provider: Arc<Provider<Http>>,
}

struct B20Constants;
impl B20Constants {
    const MINT_ROLE: [u8; 32] = [0; 32];
    const BURN_ROLE: [u8; 32] = [1; 32];
    const BURN_BLOCKED_ROLE: [u8; 32] = [2; 32];
    const PAUSE_ROLE: [u8; 32] = [3; 32];
    const UNPAUSE_ROLE: [u8; 32] = [4; 32];
    const OPERATOR_ROLE: [u8; 32] = [5; 32];
}

impl ComplianceEngine {
    pub fn new(
        ethical_filter: Arc<EthicalFilter>,
        policy_registry: Arc<PolicyAdapter>,
        b20_mapper: Arc<B20TokenMapper>,
        event_store: Arc<EventStore>,
        provider: Arc<Provider<Http>>
    ) -> Self {
        Self { ethical_filter, policy_registry, b20_mapper, event_store, provider }
    }

    /// Avalia compliance completa: etica + politicas + pausa
    pub async fn evaluate_compliance(
        &self,
        action: &Action,
    ) -> Result<ComplianceVerdict, ComplianceError> {
        let ethical = match self.ethical_filter.evaluate(action).await {
            FilterVerdict::Passed => EthicalCompliance::Passed,
            FilterVerdict::Failed(v) => EthicalCompliance::Failed(v),
        };

        let b20_op = match self.b20_mapper.map_action(action).await {
            Ok(op) => op,
            Err(e) => return Err(ComplianceError::Mapping(e)),
        };

        let policy = self.check_policies(&b20_op).await?;
        let pause = self.check_pause_state(&b20_op).await?;
        let role = self.check_roles(&b20_op, action).await?;

        let verdict = ComplianceVerdict {
            ethical: ethical.clone(),
            policy: policy.clone(),
            pause: pause.clone(),
            role: role.clone(),
            overall: ethical.is_passed() && policy.is_passed() && pause.is_passed() && role.is_passed(),
        };

        self.event_store.emit(OrchestratorEvent::ComplianceChecked {
            action_id: action.id.clone(),
            verdict: verdict.clone(),
            timestamp: chrono::Utc::now().timestamp(),
        }).await?;

        Ok(verdict)
    }

    async fn check_policies(&self, op: &B20Operation) -> Result<PolicyCompliance, ComplianceError> {
        match op {
            B20Operation::Transfer { token, from, to, .. } => {
                let sender_policy = self.policy_registry.get_policy(*token, PolicyScope::TransferSender).await.map_err(|e| ComplianceError::Policy(format!("{:?}", e)))?;
                let receiver_policy = self.policy_registry.get_policy(*token, PolicyScope::TransferReceiver).await.map_err(|e| ComplianceError::Policy(format!("{:?}", e)))?;

                let sender_ok = self.policy_registry.is_authorized(sender_policy, *from).await.map_err(|e| ComplianceError::Policy(format!("{:?}", e)))?;
                let receiver_ok = self.policy_registry.is_authorized(receiver_policy, *to).await.map_err(|e| ComplianceError::Policy(format!("{:?}", e)))?;

                if !sender_ok {
                    return Ok(PolicyCompliance::Denied(format!("sender {} blocked by policy {}", from, sender_policy)));
                }
                if !receiver_ok {
                    return Ok(PolicyCompliance::Denied(format!("receiver {} blocked by policy {}", to, receiver_policy)));
                }

                Ok(PolicyCompliance::Passed)
            }
            B20Operation::Mint { token, to, .. } => {
                let policy = self.policy_registry.get_policy(*token, PolicyScope::MintReceiver).await.map_err(|e| ComplianceError::Policy(format!("{:?}", e)))?;
                if !self.policy_registry.is_authorized(policy, *to).await.map_err(|e| ComplianceError::Policy(format!("{:?}", e)))? {
                    return Ok(PolicyCompliance::Denied(format!("mint receiver {} blocked", to)));
                }
                Ok(PolicyCompliance::Passed)
            }
            _ => Ok(PolicyCompliance::Passed),
        }
    }

    async fn check_pause_state(&self, _op: &B20Operation) -> Result<PauseCompliance, ComplianceError> {
        // Mock pause check for now
        Ok(PauseCompliance::Passed)
    }

    async fn check_roles(&self, _op: &B20Operation, _action: &Action) -> Result<RoleCompliance, ComplianceError> {
        // Mock role check for now
        Ok(RoleCompliance::Passed)
    }
}

#[derive(Debug, Clone)]
pub struct ComplianceVerdict {
    pub ethical: EthicalCompliance,
    pub policy: PolicyCompliance,
    pub pause: PauseCompliance,
    pub role: RoleCompliance,
    pub overall: bool,
}

#[derive(Debug, Clone)]
pub enum EthicalCompliance {
    Passed,
    Failed(Vec<String>),
}
impl EthicalCompliance { pub fn is_passed(&self) -> bool { matches!(self, Self::Passed) } }

#[derive(Debug, Clone)]
pub enum PolicyCompliance {
    Passed,
    Denied(String),
}
impl PolicyCompliance { pub fn is_passed(&self) -> bool { matches!(self, Self::Passed) } }

#[derive(Debug, Clone)]
pub enum PauseCompliance {
    Passed,
    Paused(PausableFeature),
}
impl PauseCompliance { pub fn is_passed(&self) -> bool { matches!(self, Self::Passed) } }

#[derive(Debug, Clone)]
pub enum RoleCompliance {
    Passed,
    MissingRole([u8; 32]),
}
impl RoleCompliance { pub fn is_passed(&self) -> bool { matches!(self, Self::Passed) } }
