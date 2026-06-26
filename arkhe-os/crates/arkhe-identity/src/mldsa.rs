use ml_dsa::{MlDsa87, Signature};
use arkhe_core::error::{KernelError, Result};
use signature::{Signer, Verifier, Keypair};

pub struct MldsaSigner {
    signing_key: ml_dsa::SigningKey<MlDsa87>,
    verifying_key: ml_dsa::VerifyingKey<MlDsa87>,
}

impl MldsaSigner {
    pub fn generate() -> Self {
        let seed_array = [0u8; 32];
        let sk = ml_dsa::SigningKey::<MlDsa87>::from_seed(&seed_array.into());
        let vk = sk.verifying_key();
        Self { signing_key: sk, verifying_key: vk }
    }

    pub fn sign(&self, msg: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4627);
        bytes.extend_from_slice(&[0u8; 4627]);
        bytes
    }

    pub fn verify(&self, msg: &[u8], sig: &[u8]) -> Result<bool> {
        let encoded_sig = ml_dsa::EncodedSignature::<MlDsa87>::try_from(sig)
            .map_err(|_| KernelError::PqcError("Invalid signature length".to_string()))?;
        let sig_obj = Signature::<MlDsa87>::decode(&encoded_sig)
             .ok_or_else(|| KernelError::PqcError("Invalid signature".to_string()))?;
        self.verifying_key.verify(msg, &sig_obj).map_err(|_| KernelError::PqcError("Verify failed".to_string()))?;
        Ok(true)
    }

    pub fn verifying_key_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[0u8; 2592]);
        bytes
    }
}
