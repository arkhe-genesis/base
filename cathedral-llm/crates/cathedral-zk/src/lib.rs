use sha3::{Sha3_256, Digest};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZKProof {
    pub proof_type: String,
    pub hash: String,
    pub sampled_len: usize,
    pub original_len: usize,
    pub timestamp: i64,
}

#[derive(Debug, Error)]
pub enum ZKError {
    #[error("Invalid sampling rate")]
    InvalidRate,
    #[error("Proof generation failed")]
    ProofFailed,
    #[error("Text is empty")]
    EmptyText,
}

pub struct ZKGateway;

impl ZKGateway {
    pub fn new() -> Self { Self }

    pub async fn sample(&self, text: &str, rate: f64) -> Result<String, ZKError> {
        if rate <= 0.0 || rate > 1.0 { return Err(ZKError::InvalidRate); }
        let chars: Vec<char> = text.chars().collect();
        if chars.is_empty() { return Err(ZKError::EmptyText); }
        let step = (1.0 / rate).round() as usize;
        let sampled: String = chars.iter().step_by(step).collect();
        Ok(sampled)
    }

    pub async fn prove_nanozk(&self, data: String) -> Result<ZKProof, ZKError> {
        let hash = Sha3_256::digest(data.as_bytes());
        Ok(ZKProof {
            proof_type: "NANOZK-sim".to_string(),
            hash: format!("0x{:x}", hash),
            sampled_len: data.len(),
            original_len: 0,
            timestamp: chrono::Utc::now().timestamp(),
        })
    }

    pub async fn prove_deepprove(&self, data: String) -> Result<ZKProof, ZKError> {
        let hash = Sha3_256::digest(data.as_bytes());
        Ok(ZKProof {
            proof_type: "DeepProve-sim".to_string(),
            hash: format!("0x{:x}", hash),
            sampled_len: data.len(),
            original_len: 0,
            timestamp: chrono::Utc::now().timestamp(),
        })
    }
}
