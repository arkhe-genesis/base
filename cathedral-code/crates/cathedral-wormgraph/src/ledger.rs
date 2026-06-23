use serde::{Deserialize, Serialize};
use cathedral_identity::Did;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub actions: Vec<Action>,
    pub timestamp: i64,
    pub hash: String,
}

impl Block {
    pub fn new(actions: Vec<Action>) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            actions,
            timestamp,
            hash: uuid::Uuid::new_v4().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub did: Did,
    pub action_type: String,
    pub data: serde_json::Value,
    pub signature: Vec<u8>,
    pub proof: Option<String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Default)]
pub struct ProvenanceTrace {
    pub blocks: Vec<Block>,
}
