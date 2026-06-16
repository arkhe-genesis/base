// Cathedral ARKHE v28.3 — Async RL Orchestrator

use std::sync::Arc;
use tokio::sync::Mutex;

pub trait RewardModel: Send + Sync {}
pub trait CathedralAgent: Send + Sync {}
pub trait ConsensusLedger: Send + Sync {}
pub struct ReplayBuffer {}
pub struct AsyncRLConfig {}

pub struct AsyncRLOrchestrator {
    // ... outros campos ...
    pub reward_model: Arc<dyn RewardModel>,   // agora aceita tanto LLM-judge quanto debate
}

impl AsyncRLOrchestrator {
    // Adicione o método new_with_debate(...) explícito no orquestrador
    pub fn new_with_debate(
        _config: AsyncRLConfig,
        _agent: Arc<Mutex<dyn CathedralAgent>>,
        _buffer: Arc<ReplayBuffer>,
        reward_model: Arc<dyn RewardModel>,  // <- pode ser DebateConsensusRewardModel
        _ledger: Option<Arc<dyn ConsensusLedger>>,
    ) -> Self {
        Self {
            reward_model,
        }
    }
}
