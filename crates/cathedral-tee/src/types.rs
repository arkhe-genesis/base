use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeeType {
    SGX,
    SevSnp,
    IoNet,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationReport {
    pub worker_id: String,
    pub tee_type: TeeType,
    pub binary_hash: String,
    pub config_hash: String,
    pub tee_quote: Vec<u8>,
    pub timestamp: i64,
    pub nonce: String,
    pub signature: Vec<u8>,
    pub arkhe_version: String,
    pub enclave_measurement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationResult {
    pub valid: bool,
    pub tee_verified: bool,
    pub binary_hash_match: bool,
    pub details: String,
}
