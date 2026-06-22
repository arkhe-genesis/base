use ed25519_dalek::{Signature, Signer, Verifier, SigningKey, VerifyingKey};
use cathedral_arkheobex::{ArkheObject, HeaderType};
use thiserror::Error;
use rand::rngs::OsRng;
use rand::RngCore;

#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("Signature failed")]
    SignatureError,
}

pub struct SignatureGuard {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl SignatureGuard {
    pub fn new() -> Self {
        let mut csprng = OsRng;
        let mut bytes = [0u8; 32];
        csprng.fill_bytes(&mut bytes);
        let signing_key = SigningKey::from_bytes(&bytes);
        let verifying_key = signing_key.verifying_key();
        Self { signing_key, verifying_key }
    }

    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.signing_key.sign(message).to_bytes().to_vec()
    }

    pub fn verify(&self, message: &[u8], signature: &[u8]) -> bool {
        if let Ok(sig) = Signature::from_slice(signature) {
            self.verifying_key.verify(message, &sig).is_ok()
        } else {
            false
        }
    }

    pub fn attest_object(&self, obj: &mut ArkheObject) -> Result<(), IdentityError> {
        let sig = self.sign(obj.body.data.as_bytes());
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(sig.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&sig);
        obj.add_header(HeaderType::PqcAttestation, bytes);
        Ok(())
    }
}

pub struct IdentityGateway {
    // mock did store
}

impl IdentityGateway {
    pub fn new() -> Self { Self {} }
    pub async fn verify(&self, _did: &str, _signature: &[u8], _message: &[u8]) -> Result<bool, IdentityError> {
        // No prototipo, assumimos true ou false
        Ok(true)
    }
}
