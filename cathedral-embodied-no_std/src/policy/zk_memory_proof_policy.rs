use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ZkMemoryProofPolicy {
    pub require_memory_proof_for_recommendations: bool,
}
impl Default for ZkMemoryProofPolicy {
    fn default() -> Self {
        Self { require_memory_proof_for_recommendations: false }
    }
}
