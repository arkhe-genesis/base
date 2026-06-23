pub mod ledger;

use crate::ledger::{Action, Block, ProvenanceTrace};

#[async_trait::async_trait]
pub trait WormGraphBackend: Send + Sync {
    async fn append_entry(&self, block: Block) -> Result<(), WormGraphError>;
    async fn get_entries(&self, limit: Option<usize>) -> Result<Vec<Block>, WormGraphError>;
}

pub struct MemoryGateway {
    backend: Box<dyn WormGraphBackend>,
}

impl MemoryGateway {
    pub fn new(backend: Box<dyn WormGraphBackend>) -> Self {
        Self { backend }
    }

    pub async fn record_action(&self, agent_id: &str, action: Action) -> Result<(), WormGraphError> {
        let block = Block::new(vec![action]);
        self.backend.append_entry(block).await
    }

    pub async fn query_provenance(&self, _id: &str) -> Result<ProvenanceTrace, WormGraphError> {
        Ok(ProvenanceTrace::default())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum WormGraphError {
    #[error("Storage error: {0}")]
    Storage(String),
}
