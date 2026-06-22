//! ML-KEM (Kyber) — Key Encapsulation Mechanism
//! Conforme FIPS 203 (agosto 2024)
//! Implementação usando a crate `ml-kem` v0.3.2
//! Selo: CATHEDRAL-ARKHE-ML-KEM-v1.0.0-2026-06-21

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MlKemError {
    #[error("Falha na geração de chaves")]
    KeyGenerationFailed,
    #[error("Falha no encapsulamento")]
    EncapsulationFailed,
    #[error("Falha no decapsulamento")]
    DecapsulationFailed,
    #[error("Chave inválida")]
    InvalidKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MlKemSecurityLevel {
    Level512,
    Level768,  // RECOMENDADO
    Level1024,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlKemKeyPair {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub level: MlKemSecurityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlKemCiphertext {
    pub ciphertext: Vec<u8>,
    pub level: MlKemSecurityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlKemSharedSecret {
    pub secret: Vec<u8>,
    pub level: MlKemSecurityLevel,
}

pub struct MlKem;

impl MlKem {
    pub fn generate_keypair(level: MlKemSecurityLevel) -> Result<MlKemKeyPair, MlKemError> {
        let (pk, sk) = match level {
            MlKemSecurityLevel::Level512 => ml_kem::generate_keypair_512(),
            MlKemSecurityLevel::Level768 => ml_kem::generate_keypair_768(),
            MlKemSecurityLevel::Level1024 => ml_kem::generate_keypair_1024(),
        };
        Ok(MlKemKeyPair {
            public_key: pk.to_vec(),
            private_key: sk.to_vec(),
            level,
        })
    }

    pub fn encapsulate(public_key: &[u8], level: MlKemSecurityLevel) -> Result<(MlKemCiphertext, MlKemSharedSecret), MlKemError> {
        let (ct, ss) = match level {
            MlKemSecurityLevel::Level512 => ml_kem::encapsulate_512(public_key),
            MlKemSecurityLevel::Level768 => ml_kem::encapsulate_768(public_key),
            MlKemSecurityLevel::Level1024 => ml_kem::encapsulate_1024(public_key),
        }.map_err(|_| MlKemError::EncapsulationFailed)?;
        Ok((MlKemCiphertext { ciphertext: ct.to_vec(), level }, MlKemSharedSecret { secret: ss.to_vec(), level }))
    }

    pub fn decapsulate(ciphertext: &[u8], private_key: &[u8], level: MlKemSecurityLevel) -> Result<MlKemSharedSecret, MlKemError> {
        let ss = match level {
            MlKemSecurityLevel::Level512 => ml_kem::decapsulate_512(ciphertext, private_key),
            MlKemSecurityLevel::Level768 => ml_kem::decapsulate_768(ciphertext, private_key),
            MlKemSecurityLevel::Level1024 => ml_kem::decapsulate_1024(ciphertext, private_key),
        }.map_err(|_| MlKemError::DecapsulationFailed)?;
        Ok(MlKemSharedSecret { secret: ss.to_vec(), level })
    }

    pub fn key_sizes(level: MlKemSecurityLevel) -> (usize, usize, usize, usize) {
        match level {
            MlKemSecurityLevel::Level512 => (800, 1632, 768, 32),
            MlKemSecurityLevel::Level768 => (1184, 2400, 1088, 32),
            MlKemSecurityLevel::Level1024 => (1568, 3168, 1568, 32),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_kem() {
        let kp = MlKem::generate_keypair(MlKemSecurityLevel::Level768).unwrap();
        let (ct, ss1) = MlKem::encapsulate(&kp.public_key, MlKemSecurityLevel::Level768).unwrap();
        let ss2 = MlKem::decapsulate(&ct.ciphertext, &kp.private_key, MlKemSecurityLevel::Level768).unwrap();
        assert_eq!(ss1.secret, ss2.secret);
    }
}