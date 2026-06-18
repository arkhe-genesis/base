//! Cathedral ARKHE v28.3.2 — Identity Derivation (HMAC + BIP32)
//! Derivação hierárquica de identidades usando HMAC-SHA512 e caminhos BIP32.
//! Selo: CATHEDRAL-ARKHE-v28.3.2-IDENTITY-DERIVATION-2026-06-17

use std::sync::Arc;
use hmac::{Hmac, Mac};
use sha2::Sha512;
use blake3::hash as blake3_hash;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::attestation::IdentityAttestation;

// ============================================================================
// 1. Tipos de Derivação
// ============================================================================

type HmacSha512 = Hmac<Sha512>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivationPath {
    pub segments: Vec<String>,
}

impl DerivationPath {
    /// Cria um caminho de derivação a partir de uma string (ex: "parent/purpose/index")
    pub fn from_string(path: &str) -> Self {
        Self {
            segments: path.split('/').map(|s| s.to_string()).collect(),
        }
    }

    /// Retorna o caminho como string.
    pub fn to_string(&self) -> String {
        self.segments.join("/")
    }

    /// Adiciona um segmento ao caminho.
    pub fn push(&mut self, segment: &str) {
        self.segments.push(segment.to_string());
    }

    /// Retorna o último segmento (ex: o índice).
    pub fn last(&self) -> Option<&str> {
        self.segments.last().map(|s| s.as_str())
    }
}

impl Default for DerivationPath {
    fn default() -> Self {
        Self { segments: vec![] }
    }
}

// ============================================================================
// 2. Derivação HMAC-SHA512
// ============================================================================

/// Deriva um valor usando HMAC-SHA512.
pub fn hmac_derive(key: &[u8], data: &[u8]) -> [u8; 64] {
    let mut mac = <HmacSha512 as Mac>::new_from_slice(key)
        .expect("HMAC key deve ser válida");
    mac.update(data);
    let result = mac.finalize();
    let mut bytes = [0u8; 64];
    bytes.copy_from_slice(&result.into_bytes()[..64]);
    bytes
}

/// Deriva uma chave filha a partir de uma chave pai usando HMAC-SHA512.
pub struct DerivedKey {
    pub chain_code: [u8; 32],
    pub private_key: [u8; 32],
    pub public_key: [u8; 32],
    pub depth: u32,
    pub index: u32,
}

impl DerivedKey {
    /// Deriva uma chave filha a partir de uma chave pai (BIP32-like).
    pub fn derive_child(parent_chain: &[u8; 32], parent_key: &[u8; 32], index: u32) -> Self {
        // 1. Prepara o dado para HMAC: parent_key || index
        let mut data = Vec::with_capacity(32 + 4);
        data.extend_from_slice(parent_key);
        data.extend_from_slice(&index.to_be_bytes());

        // 2. HMAC-SHA512(chain_code, data)
        let hmac_result = hmac_derive(parent_chain, &data);

        // 3. Divide em 32+32 bytes
        let (child_key, child_chain) = hmac_result.split_at(32);

        let mut private_key = [0u8; 32];
        private_key.copy_from_slice(child_key);

        let mut chain_code = [0u8; 32];
        chain_code.copy_from_slice(child_chain);

        // 4. Public key derivada (simplificada: hash da privada)
        let binding = blake3_hash(&private_key);
        let public_key = binding.as_bytes();
        let mut pub_key = [0u8; 32];
        pub_key.copy_from_slice(&public_key[..32]);

        Self {
            chain_code,
            private_key,
            public_key: pub_key,
            depth: 0,
            index,
        }
    }

    /// Deriva uma chave a partir de um caminho completo.
    pub fn derive_from_path(
        master_chain: &[u8; 32],
        master_private: &[u8; 32],
        path: &DerivationPath,
    ) -> Self {
        let mut current_chain = *master_chain;
        let mut current_private = *master_private;
        let mut depth = 0;

        for (idx, segment) in path.segments.iter().enumerate() {
            let index = segment.parse::<u32>().unwrap_or(idx as u32);
            let child = Self::derive_child(&current_chain, &current_private, index);
            current_chain = child.chain_code;
            current_private = child.private_key;
            depth += 1;
        }

        // Recalcula public key final
        let binding = blake3_hash(&current_private);
        let public_key = binding.as_bytes();
        let mut pub_key = [0u8; 32];
        pub_key.copy_from_slice(&public_key[..32]);

        Self {
            chain_code: current_chain,
            private_key: current_private,
            public_key: pub_key,
            depth,
            index: 0,
        }
    }

    /// Converte para uma IdentityAttestation derivada.
    pub fn to_identity_attestation(&self, parent_id: &str, purpose: &str) -> IdentityAttestation {
        let id = format!("sub-{}", hex::encode(&self.public_key[..8]));
        IdentityAttestation {
            id,
            architect_id: parent_id.to_string(),
            voice_hash: "".to_string(),
            biometric_score: 1.0,
            coercion_score: 0.0,
            blockchain_signature_id: None,
            hardware_fingerprint: Some(hex::encode(&self.public_key)),
            confidence: 0.95,
            signature: None,
            signer_key_id: format!("derived:{}", purpose),
            metadata: serde_json::json!({
                "derivation_path": purpose,
                "depth": self.depth,
                "public_key": hex::encode(&self.public_key),
            }),
        }
    }
}

// ============================================================================
// 3. IdentityDerivationService
// ============================================================================

pub struct IdentityDerivationService {
    master_chain: [u8; 32],
    master_private: [u8; 32],
    parent_id: String,
}

impl IdentityDerivationService {
    /// Cria um novo serviço de derivação a partir de uma chave mestra.
    /// A chave mestra pode ser derivada do IdentityAttestation do pai.
    pub fn new(parent_attestation: &IdentityAttestation, seed: &[u8; 32]) -> Self {
        // Deriva a chave mestra a partir do pai
        let master_private = *seed;
        let binding = blake3_hash(&master_private);
        let master_chain = binding.as_bytes();
        let mut chain = [0u8; 32];
        chain.copy_from_slice(&master_chain[..32]);

        Self {
            master_chain: chain,
            master_private,
            parent_id: parent_attestation.id.clone(),
        }
    }

    /// Cria um serviço de derivação a partir de uma seed e parent_id.
    pub fn from_seed(parent_id: &str, seed: &[u8; 32]) -> Self {
        let master_private = *seed;
        let binding = blake3_hash(&master_private);
        let master_chain = binding.as_bytes();
        let mut chain = [0u8; 32];
        chain.copy_from_slice(&master_chain[..32]);

        Self {
            master_chain: chain,
            master_private,
            parent_id: parent_id.to_string(),
        }
    }

    /// Deriva uma identidade para um propósito específico.
    pub fn derive(&self, purpose: &str, index: u32) -> DerivedKey {
        let path = DerivationPath::from_string(&format!("{}/{}", purpose, index));
        DerivedKey::derive_from_path(&self.master_chain, &self.master_private, &path)
    }

    /// Deriva uma identidade e retorna como IdentityAttestation.
    pub fn derive_attestation(&self, purpose: &str, index: u32) -> IdentityAttestation {
        let key = self.derive(purpose, index);
        key.to_identity_attestation(&self.parent_id, purpose)
    }

    /// Deriva múltiplas identidades para diferentes propósitos.
    pub fn derive_multi(&self, purposes: &[(&str, u32)]) -> Vec<IdentityAttestation> {
        purposes.iter()
            .map(|(purpose, index)| self.derive_attestation(purpose, *index))
            .collect()
    }

    /// Verifica se uma identidade foi derivada deste serviço.
    pub fn verify_derivation(&self, attestation: &IdentityAttestation, purpose: &str, index: u32) -> bool {
        let expected = self.derive_attestation(purpose, index);
        attestation.id == expected.id
            && attestation.signer_key_id == expected.signer_key_id
            && attestation.architect_id == expected.architect_id
    }
}

// ============================================================================
// 5. Testes
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_derive() {
        let key = [0u8; 32];
        let data = b"test data";
        let result = hmac_derive(&key, data);
        assert_eq!(result.len(), 64);
    }

    #[test]
    fn test_derived_key_derive_child() {
        let chain = [0u8; 32];
        let key = [1u8; 32];
        let child = DerivedKey::derive_child(&chain, &key, 0);
        assert_ne!(child.private_key, key);
        assert_ne!(child.chain_code, chain);
    }

    #[test]
    fn test_derivation_path() {
        let path = DerivationPath::from_string("parent/email/0");
        assert_eq!(path.segments.len(), 3);
        assert_eq!(path.segments[0], "parent");
        assert_eq!(path.segments[1], "email");
        assert_eq!(path.segments[2], "0");
    }

    #[test]
    fn test_derive_attestation() {
        let parent_id = "parent-123";
        let seed = [42u8; 32];
        let service = IdentityDerivationService::from_seed(parent_id, &seed);

        let att = service.derive_attestation("email_analyzer", 0);
        assert_eq!(att.architect_id, parent_id);
        assert!(att.id.starts_with("sub-"));
        assert!(att.signer_key_id.contains("derived:"));
    }
}
