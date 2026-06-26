use std::collections::HashMap;

use crate::types::{AttestationReport, AttestationResult, TeeType};

pub struct Verifier {
    trusted_hashes: HashMap<String, String>,
}

impl Verifier {
    pub fn new() -> Self {
        Self { trusted_hashes: HashMap::new() }
    }

    pub fn register_trusted_hash(&mut self, worker_id: &str, hash: &str) {
        self.trusted_hashes.insert(worker_id.to_string(), hash.to_string());
    }

    pub fn verify(&self, report: &AttestationReport) -> AttestationResult {
        let hash_match = if let Some(expected) = self.trusted_hashes.get(&report.worker_id) {
            &report.binary_hash == expected
        } else {
            matches!(report.tee_type, TeeType::SGX | TeeType::SevSnp)
        };

        if !hash_match {
            return AttestationResult {
                valid: false,
                tee_verified: false,
                binary_hash_match: false,
                details: "Binary hash mismatch".to_string(),
            };
        }

        let tee_verified = match report.tee_type {
            TeeType::SGX | TeeType::SevSnp | TeeType::IoNet => true,
            TeeType::None => false,
        };

        if !tee_verified {
            return AttestationResult {
                valid: false,
                tee_verified: false,
                binary_hash_match: hash_match,
                details: "TEE verification failed".to_string(),
            };
        }

        AttestationResult {
            valid: true,
            tee_verified: true,
            binary_hash_match: hash_match,
            details: format!("Attestation passed for {:?}", report.tee_type),
        }
    }
}
