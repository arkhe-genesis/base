use ml_kem::MlKem1024;

pub struct HybridKeyExchange {
    // Stub
}

impl HybridKeyExchange {
    pub fn generate() -> Self {
        Self {}
    }

    pub fn public_key_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(32 + 1568);
        bytes.extend_from_slice(&[0u8; 32]);
        bytes.extend_from_slice(&[0u8; 1568]); // stub export
        bytes
    }

    pub fn derive_shared(&self, _peer_public: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(32 + 32);
        bytes.extend_from_slice(&[0u8; 64]);
        let hash = blake3::hash(&bytes);
        hash.as_bytes().to_vec()
    }
}
