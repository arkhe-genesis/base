use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Did(pub String);

impl Did {
    pub fn new(method: &str, identifier: &str) -> Self {
        Self(format!("did:{}:{}", method, identifier))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    pub agent_id: AgentId,
    pub capability: Capability,
    pub expires_at: u64,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Capability {
    TransferAssets,
    VerifyProofs,
    QueryUniverse,
    DelegateTasks,
    ReadMemory,
    WriteMemory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRef {
    pub chain: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Intent {
    TransferAsset {
        asset: AssetRef,
        amount: u64,
        recipient: Did,
        priority: u8,
    },
    VerifyProof {
        proof: Vec<u8>,
        public_inputs: Vec<u8>,
        priority: u8,
    },
    DelegateTask {
        task: Task,
        to: AgentId,
        priority: u8,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub description: String,
    pub payload: Vec<u8>,
}
