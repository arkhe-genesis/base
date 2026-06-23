//! SLH-DSA (SPHINCS+) — Assinatura Digital Pós-Quantum
//! Conforme FIPS 205 (agosto 2024)
//! Implementação usando a crate `slh-dsa` v0.1.0
//! Selo: CATHEDRAL-ARKHE-SLH-DSA-v1.0.0-2026-06-21

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SlhDsaError {
    #[error("Falha na geração de chaves")]
    KeyGenerationFailed,
    #[error("Falha na assinatura")]
    SigningFailed,
    #[error("Falha na verificação")]
    VerificationFailed,
    #[error("Chave inválida")]
    InvalidKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SlhDsaSecurityLevel {
    Level128s,  // 7.856 bytes assinatura
    Level128f,  // 17.088 bytes assinatura, mais rápido
    Level192s,  // 16.224 bytes
    Level256s,  // 29.792 bytes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlhDsaKeyPair {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub level: SlhDsaSecurityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlhDsaSignature {
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
    pub level: SlhDsaSecurityLevel,
}

pub struct SlhDsa;

impl SlhDsa {
    pub fn generate_keypair(level: SlhDsaSecurityLevel) -> Result<SlhDsaKeyPair, SlhDsaError> {
        let (pk, sk) = match level {
            SlhDsaSecurityLevel::Level128s => slh_dsa::generate_keypair_128s(),
            SlhDsaSecurityLevel::Level128f => slh_dsa::generate_keypair_128f(),
            SlhDsaSecurityLevel::Level192s => slh_dsa::generate_keypair_192s(),
            SlhDsaSecurityLevel::Level256s => slh_dsa::generate_keypair_256s(),
        };
        Ok(SlhDsaKeyPair {
            public_key: pk.to_vec(),
            private_key: sk.to_vec(),
            level,
        })
    }

    pub fn sign(message: &[u8], keypair: &SlhDsaKeyPair) -> Result<SlhDsaSignature, SlhDsaError> {
        let sig = match keypair.level {
            SlhDsaSecurityLevel::Level128s => slh_dsa::sign_128s(message, &keypair.private_key),
            SlhDsaSecurityLevel::Level128f => slh_dsa::sign_128f(message, &keypair.private_key),
            SlhDsaSecurityLevel::Level192s => slh_dsa::sign_192s(message, &keypair.private_key),
            SlhDsaSecurityLevel::Level256s => slh_dsa::sign_256s(message, &keypair.private_key),
        }.map_err(|_| SlhDsaError::SigningFailed)?;
        Ok(SlhDsaSignature {
            signature: sig.to_vec(),
            public_key: keypair.public_key.clone(),
            level: keypair.level,
        })
    }

    pub fn verify(message: &[u8], signature: &SlhDsaSignature) -> Result<bool, SlhDsaError> {
        let valid = match signature.level {
            SlhDsaSecurityLevel::Level128s => slh_dsa::verify_128s(message, &signature.signature, &signature.public_key),
            SlhDsaSecurityLevel::Level128f => slh_dsa::verify_128f(message, &signature.signature, &signature.public_key),
            SlhDsaSecurityLevel::Level192s => slh_dsa::verify_192s(message, &signature.signature, &signature.public_key),
            SlhDsaSecurityLevel::Level256s => slh_dsa::verify_256s(message, &signature.signature, &signature.public_key),
        }.map_err(|_| SlhDsaError::VerificationFailed)?;
        Ok(valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slh_dsa() {
        let kp = SlhDsa::generate_keypair(SlhDsaSecurityLevel::Level128s).unwrap();
        let msg = b"Cathedral-OS SLH-DSA test";
        let sig = SlhDsa::sign(msg, &kp).unwrap();
        assert!(SlhDsa::verify(msg, &sig).unwrap());
    }
}