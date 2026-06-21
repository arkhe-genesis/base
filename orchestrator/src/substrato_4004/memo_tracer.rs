//! src/substrato_4004/memo_tracer.rs
//! Rastreia memos B20 para integracao com EventStore e CrossChainEmitter

use std::sync::Arc;

use ethers::types::Address;
use sha2::{Digest, Sha256};

use crate::substrato_4004::{
    b20_mapper::Action,
    compliance_engine::{EventStore, OrchestratorEvent},
    settlement_engine::CrossChainEmitterV2,
};

#[derive(Debug)]
pub enum TracerError {
    EventStoreError(String),
    CrossChainError(String),
}

pub struct MemoTracer {
    #[allow(dead_code)]
    event_store: Arc<EventStore>,
    cross_chain_emitter: Arc<CrossChainEmitterV2>,
}

impl MemoTracer {
    pub fn new(
        #[allow(dead_code)] event_store: Arc<EventStore>,
        cross_chain_emitter: Arc<CrossChainEmitterV2>,
    ) -> Self {
        Self { event_store, cross_chain_emitter }
    }

    /// Gera memo a partir de uma Cathedral Action
    pub fn generate_memo(&self, action: &Action) -> [u8; 32] {
        let action_hash = Sha256::digest(action.canonical_bytes());
        let mut memo = [0u8; 32];
        memo.copy_from_slice(&action_hash[..32]);
        memo
    }

    /// Indexa evento Memo do B20 no EventStore
    pub async fn index_memo_event(
        &self,
        tx_hash: &str,
        log_index: u64,
        caller: Address,
        memo: [u8; 32],
    ) -> Result<(), TracerError> {
        let event = OrchestratorEvent::B20Memo {
            tx_hash: tx_hash.to_string(),
            log_index,
            caller: format!("{:?}", caller),
            memo: hex::encode(memo),
            timestamp: chrono::Utc::now().timestamp(),
        };

        // Mock store
        // self.event_store.store(event.clone()).await?;

        self.cross_chain_emitter
            .emit_cross_chain(event)
            .await
            .map_err(|e| TracerError::CrossChainError(format!("{:?}", e)))?;

        Ok(())
    }

    /// Resolve memo para Action original
    pub async fn resolve_memo(&self, _memo: [u8; 32]) -> Result<Option<Action>, TracerError> {
        // Mock DB lookup
        Ok(None)
    }
}
