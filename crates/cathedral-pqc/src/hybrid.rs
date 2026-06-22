//! Certificado Híbrido — Ed25519 + ML-DSA
//! Conforme draft-ietf-lamps-pq-composite-sigs
//! Selo: CATHEDRAL-ARKHE-HYBRID-CERT-v1.0.0-2026-06-21

use serde::{Deserialize, Serialize};
use ed25519_dalek::{Signer, Verifier, SigningKey, VerifyingKey};
use crate::ml_dsa::{Mldsa, MldsaKeyPair, MldsaSecurityLevel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridCertificate {
    pub id: String,
    pub agent_id: String,
    pub ed25519_public_key: Vec<u8>,
    pub ml_dsa_public_key: Vec<u8>,
    pub composite_signature: Vec<u8>,
    pub issuer: String,
    pub valid_from: i64,
    pub valid_until: i64,
    pub extensions: Vec<(String, Vec<u8>)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeSignatureValue {
    pub sig1: Vec<u8>,   // Ed25519 (64 bytes)
    pub sig2: Vec<u8>,   // ML-DSA
}

impl HybridCertificate {
    pub fn new(
        id: String,
        agent_id: String,
        ed25519_signing_key: &SigningKey,
        ml_dsa_keypair: &MldsaKeyPair,
        issuer: String,
        valid_from: i64,
        valid_until: i64,
        extensions: Vec<(String, Vec<u8>)>,
    ) -> Self {
        let ed_pub = ed25519_signing_key.verifying_key().to_bytes().to_vec();
        let ml_pub = ml_dsa_keypair.public_key.clone();

        let cert_data = format!(
            "{}{}{}{}{}{}{}",
            id, agent_id,
            hex::encode(&ed_pub),
            hex::encode(&ml_pub),
            issuer, valid_from, valid_until
        );
        let data = cert_data.as_bytes();

        let sig1 = ed25519_signing_key.sign(data).to_bytes().to_vec();
        let ml_sig = Mldsa::sign(data, ml_dsa_keypair).unwrap();
        let sig2 = ml_sig.signature;

        let composite = CompositeSignatureValue { sig1, sig2 };
        let composite_signature = bincode::serialize(&composite).unwrap();

        Self {
            id,
            agent_id,
            ed25519_public_key: ed_pub,
            ml_dsa_public_key: ml_pub,
            composite_signature,
            issuer,
            valid_from,
            valid_until,
            extensions,
        }
    }

    pub fn verify(&self, ed25519_pub: &[u8], ml_dsa_pub: &[u8]) -> bool {
        if self.ed25519_public_key != ed25519_pub || self.ml_dsa_public_key != ml_dsa_pub {
            return false;
        }
        let composite: CompositeSignatureValue = match bincode::deserialize(&self.composite_signature) {
            Ok(c) => c,
            Err(_) => return false,
        };
        let cert_data = format!(
            "{}{}{}{}{}{}{}",
            self.id, self.agent_id,
            hex::encode(&self.ed25519_public_key),
            hex::encode(&self.ml_dsa_public_key),
            self.issuer, self.valid_from, self.valid_until
        );
        let data = cert_data.as_bytes();

        // Verifica Ed25519
        let ed_pub = match VerifyingKey::from_bytes(ed25519_pub.try_into().unwrap()) {
            Ok(k) => k,
            Err(_) => return false,
        };
        let ed_sig = match ed25519_dalek::Signature::from_bytes(composite.sig1.as_slice().try_into().unwrap()) {
            Ok(s) => s,
            Err(_) => return false,
        };
        if ed_pub.verify(data, &ed_sig).is_err() {
            return false;
        }

        // Verifica ML-DSA
        let ml_sig = crate::ml_dsa::MldsaSignature {
            signature: composite.sig2,
            public_key: self.ml_dsa_public_key.clone(),
            level: MldsaSecurityLevel::Level65,
        };
        Mldsa::verify(data, &ml_sig).unwrap_or(false)
    }

    pub fn is_valid_at(&self, timestamp: i64) -> bool {
        timestamp >= self.valid_from && timestamp <= self.valid_until
    }
}