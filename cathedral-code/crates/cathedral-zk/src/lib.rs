use serde::{Deserialize, Serialize};

pub struct ZKGateway {
    // Stub
}

impl ZKGateway {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn prove_statement(&self, _statement: &str) -> Result<String, ZkError> {
        Ok("0xMockZkProof".to_string())
    }

    pub async fn verify_proof(&self, _proof: &str) -> Result<bool, ZkError> {
        Ok(true)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ZkError {
    #[error("Proving failed")]
    ProveError,
    #[error("Verification failed")]
    VerifyError,
}
