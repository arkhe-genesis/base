pub mod ledger;

use crate::ledger::{Action, Block, ProvenanceTrace};
use cathedral_identity::Did;
use dashmap::DashMap;

#[async_trait::async_trait]
pub trait WormGraphBackend: Send + Sync {
    async fn append_entry(&self, block: Block) -> Result<(), WormGraphError>;
    async fn get_entries(&self, limit: Option<usize>) -> Result<Vec<Block>, WormGraphError>;
}

pub struct WormGraphClient {
    backend: Box<dyn WormGraphBackend>,
}

impl WormGraphClient {
    pub fn new(backend: Box<dyn WormGraphBackend>) -> Self {
        Self { backend }
    }

    pub async fn record_action(&self, did: &Did, action_type: &str, data: serde_json::Value) -> Result<String, WormGraphError> {
        let action = Action {
            id: uuid::Uuid::new_v4().to_string(),
            did: did.clone(),
            action_type: action_type.to_string(),
            data,
            signature: vec![],
            proof: None,
            timestamp: chrono::Utc::now().timestamp(),
        };

        let block = Block::new(vec![action]);
        self.backend.append_entry(block.clone()).await?;

        Ok(block.hash.clone())
    }

    pub async fn query_provenance(&self, _id: &str) -> Result<ProvenanceTrace, WormGraphError> {
        Ok(ProvenanceTrace::default())
    }
}

pub struct MemoryGateway {
    entries: DashMap<String, Block>,
}

impl MemoryGateway {
    pub fn new() -> Self {
        Self {
            entries: DashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl WormGraphBackend for MemoryGateway {
    async fn append_entry(&self, block: Block) -> Result<(), WormGraphError> {
        self.entries.insert(block.hash.clone(), block);
        Ok(())
    }

    async fn get_entries(&self, limit: Option<usize>) -> Result<Vec<Block>, WormGraphError> {
        let mut blocks: Vec<Block> = self.entries.iter().map(|kv| kv.value().clone()).collect();
        blocks.sort_by_key(|b| -b.timestamp);
        if let Some(l) = limit {
            blocks.truncate(l);
        }
        Ok(blocks)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum WormGraphError {
    #[error("Storage error: {0}")]
    Storage(String),
}
