//! PQC signatures com fallback Ed25519

#[cfg(feature = "mldsa")]
use pqcrypto_dilithium::dilithium5::*;

use ed25519_dalek::{SigningKey, Signature, VerifyingKey};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PqcError {
    #[error("Key generation failed")]
    KeygenFailed,
    #[error("Signing failed")]
    SigningFailed,
    #[error("Verification failed")]
    VerificationFailed,
    #[error("Invalid key")]
    InvalidKey,
    #[error("ML-DSA not available (feature not enabled)")]
    MldsaUnavailable,
}

pub enum PqcAlgorithm {
    Ed25519,
    Mldsa65,
}

pub struct PqcKeyPair {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub algorithm: PqcAlgorithm,
}

impl PqcKeyPair {
    pub fn generate(algorithm: PqcAlgorithm) -> Result<Self, PqcError> {
        match algorithm {
            PqcAlgorithm::Ed25519 => {
                use rand::rngs::OsRng;
                let mut csprng = OsRng {};
                use ed25519_dalek::SecretKey;
                let mut secret = [0u8; 32];
                use rand::RngCore;
                csprng.fill_bytes(&mut secret);
                let sk = SigningKey::from_bytes(&secret);
                Ok(Self {
                    public_key: sk.verifying_key().to_bytes().to_vec(),
                    private_key: sk.to_bytes().to_vec(),
                    algorithm,
                })
            }
            #[cfg(feature = "mldsa")]
            PqcAlgorithm::Mldsa65 => {
                let (pk, sk) = keypair()
                    .map_err(|_| PqcError::KeygenFailed)?;
                Ok(Self {
                    public_key: pk.as_bytes().to_vec(),
                    private_key: sk.as_bytes().to_vec(),
                    algorithm,
                })
            }
            #[cfg(not(feature = "mldsa"))]
            PqcAlgorithm::Mldsa65 => Err(PqcError::MldsaUnavailable),
        }
    }

    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, PqcError> {
        match &self.algorithm {
            PqcAlgorithm::Ed25519 => {
                let sk = SigningKey::from_bytes(&self.private_key.clone().try_into().unwrap());
                use ed25519_dalek::Signer;
                let sig: Signature = sk.sign(message);
                Ok(sig.to_bytes().to_vec())
            }
            #[cfg(feature = "mldsa")]
            PqcAlgorithm::Mldsa65 => {
                let sk = SecretKey::from_bytes(&self.private_key.try_into().unwrap())
                    .map_err(|_| PqcError::InvalidKey)?;
                let sig = sign(message, &sk)
                    .map_err(|_| PqcError::SigningFailed)?;
                Ok(sig.as_bytes().to_vec())
            }
            #[cfg(not(feature = "mldsa"))]
            PqcAlgorithm::Mldsa65 => Err(PqcError::MldsaUnavailable),
        }
    }

    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<bool, PqcError> {
        match &self.algorithm {
            PqcAlgorithm::Ed25519 => {
                let pk = VerifyingKey::from_bytes(&self.public_key.clone().try_into().unwrap())
                    .map_err(|_| PqcError::InvalidKey)?;
                let sig = Signature::from_bytes(signature.try_into().unwrap());
                use ed25519_dalek::Verifier;
                Ok(pk.verify(message, &sig).is_ok())
            }
            #[cfg(feature = "mldsa")]
            PqcAlgorithm::Mldsa65 => {
                let pk = PublicKey::from_bytes(&self.public_key.try_into().unwrap())
                    .map_err(|_| PqcError::InvalidKey)?;
                let sig = Signature::from_bytes(signature.try_into().unwrap())
                    .map_err(|_| PqcError::InvalidKey)?;
                Ok(verify(message, &sig, &pk).is_ok())
            }
            #[cfg(not(feature = "mldsa"))]
            PqcAlgorithm::Mldsa65 => Err(PqcError::MldsaUnavailable),
        }
    }
}
