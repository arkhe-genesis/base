//! Quantum-Safe Certificates (QSC)
//! Selo: CATHEDRAL-ARKHE-QSC-v1.0.0-2026-06-21

use serde::{Deserialize, Serialize};
use crate::ml_dsa::{Mldsa, MldsaKeyPair, MldsaSecurityLevel};
use crate::slh_dsa::{SlhDsa, SlhDsaKeyPair, SlhDsaSecurityLevel};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CertificateExtensions {
    pub zk_proof_hash: Option<Vec<u8>>,
    pub reputation_score: Option<u32>,
    pub capabilities: Vec<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumSafeCertificate {
    pub id: String,
    pub agent_id: String,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
    pub issuer: String,
    pub valid_from: i64,
    pub valid_until: i64,
    pub algorithm: String,
    pub extensions: CertificateExtensions,
}

impl QuantumSafeCertificate {
    pub fn new_ml_dsa(
        id: String,
        agent_id: String,
        keypair: &MldsaKeyPair,
        issuer: String,
        valid_from: i64,
        valid_until: i64,
        extensions: CertificateExtensions,
    ) -> Self {
        let cert_data = format!(
            "{}{}{}{}{}{}",
            id, agent_id,
            hex::encode(&keypair.public_key),
            issuer, valid_from, valid_until
        );
        let sig = Mldsa::sign(cert_data.as_bytes(), keypair).unwrap();
        Self {
            id,
            agent_id,
            public_key: keypair.public_key.clone(),
            signature: sig.signature,
            issuer,
            valid_from,
            valid_until,
            algorithm: format!("ML-DSA-{:?}", keypair.level),
            extensions,
        }
    }

    pub fn verify(&self, public_key: &[u8]) -> bool {
        if self.public_key != public_key { return false; }
        let cert_data = format!(
            "{}{}{}{}{}{}",
            self.id, self.agent_id,
            hex::encode(&self.public_key),
            self.issuer, self.valid_from, self.valid_until
        );
        // Tenta ML-DSA, depois SLH-DSA
        let ml_sig = crate::ml_dsa::MldsaSignature {
            signature: self.signature.clone(),
            public_key: self.public_key.clone(),
            level: MldsaSecurityLevel::Level65,
        };
        if let Ok(valid) = Mldsa::verify(cert_data.as_bytes(), &ml_sig) {
            if valid { return true; }
        }
        let slh_sig = crate::slh_dsa::SlhDsaSignature {
            signature: self.signature.clone(),
            public_key: self.public_key.clone(),
            level: SlhDsaSecurityLevel::Level128s,
        };
        SlhDsa::verify(cert_data.as_bytes(), &slh_sig).unwrap_or(false)
    }

    pub fn is_valid_at(&self, timestamp: i64) -> bool {
        timestamp >= self.valid_from && timestamp <= self.valid_until
    }
}