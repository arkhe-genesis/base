use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionAttestation {
    pub id: String,
    pub cost_usd: f64,
}

impl ExecutionAttestation {
    pub fn new(
        _task: &str,
        _result: &str,
        id: &str,
        _cost: f64,
        _policies: Vec<String>,
        _conf: f64,
        _key: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            cost_usd: 0.0,
        }
    }
    pub fn sign(&mut self, _signer: &dyn AttestationSigner) -> Result<(), String> {
        Ok(())
    }
    pub fn is_policy_compliant(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityAttestation {
    pub id: String,
    pub architect_id: String,
    pub voice_hash: String,
    pub biometric_score: f64,
    pub coercion_score: f64,
    pub blockchain_signature_id: Option<String>,
    pub hardware_fingerprint: Option<String>,
    pub confidence: f64,
    pub signature: Option<String>,
    pub signer_key_id: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDescriptor {
    pub name: String,
    pub blocking: bool,
}

pub trait AttestationProvider: Send + Sync {
    fn run(
        &self,
        task: &str,
        cost_cap: Option<f64>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>>;
}

pub trait AttestationSigner: Send + Sync {
    fn sign(&self, data: &str) -> Result<String, String>;
}

pub trait AttestationVerifier: Send + Sync {
    fn verify(&self, data: &str, signature: &str) -> Result<bool, String>;
}

pub struct AttestationManager {}
pub trait IdentityAttestationProvider: Send + Sync {}
