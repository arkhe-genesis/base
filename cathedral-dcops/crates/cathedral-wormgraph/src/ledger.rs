use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub actions: Vec<Action>,
    pub timestamp: i64,
}

impl Block {
    pub fn new(actions: Vec<Action>) -> Self {
        Self {
            actions,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub agent_id: String,
    pub action_type: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Default)]
pub struct ProvenanceTrace {
    pub blocks: Vec<Block>,
}
