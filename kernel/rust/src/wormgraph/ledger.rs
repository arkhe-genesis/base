//! WormGraph — Ledger imutável append-only com verificação de integridade
//! Selo: CATHEDRAL-ARKHE-WORMGRAPH-v1.0.0-2026-06-21

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use blake3::Hash;
use chrono::Utc;
use ed25519_dalek::{Signer, Verifier, Signature, SigningKey, VerifyingKey};

/// Entrada do ledger (append-only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: String,
    pub version: u64,
    pub decision_type: String,
    pub before_state: String,
    pub after_state: String,
    pub rationale: Option<String>,
    pub timestamp: i64,
    pub agent_id: String,
    pub entry_hash: Vec<u8>,
    pub parent_hash: Vec<u8>,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
    pub nostr_event_id: Option<String>,
    pub tree_id: Option<String>,
    pub parent_event_id: Option<String>,
    pub zk_proof_hash: Option<Vec<u8>>,
}

impl LedgerEntry {
    pub fn new(
        id: String,
        decision_type: String,
        before_state: String,
        after_state: String,
        rationale: Option<String>,
        agent_id: String,
        parent_hash: Vec<u8>,
        signing_key: &SigningKey,
        public_key: VerifyingKey,
        zk_proof_hash: Option<Vec<u8>>,
    ) -> Self {
        let timestamp = Utc::now().timestamp();
        let entry_data = format!(
            "{}{}{}{}{}{}{}",
            id, decision_type, before_state, after_state, agent_id, timestamp, hex::encode(&parent_hash)
        );
        let entry_hash = blake3::hash(entry_data.as_bytes()).as_bytes().to_vec();
        let signature = signing_key.sign(entry_data.as_bytes()).to_bytes().to_vec();

        Self {
            id,
            version: 1,
            decision_type,
            before_state,
            after_state,
            rationale,
            timestamp,
            agent_id,
            entry_hash: entry_hash.clone(),
            parent_hash,
            signature,
            public_key: public_key.to_bytes().to_vec(),
            nostr_event_id: None,
            tree_id: None,
            parent_event_id: None,
            zk_proof_hash,
        }
    }

    pub fn verify(&self) -> bool {
        let entry_data = format!(
            "{}{}{}{}{}{}{}",
            self.id, self.decision_type, self.before_state, self.after_state,
            self.agent_id, self.timestamp, hex::encode(&self.parent_hash)
        );
        let computed_hash = blake3::hash(entry_data.as_bytes()).as_bytes().to_vec();
        if computed_hash != self.entry_hash {
            return false;
        }
        let public_key = match VerifyingKey::from_bytes(&self.public_key.clone().try_into().unwrap()) {
            Ok(k) => k,
            Err(_) => return false,
        };
        let sig = match Signature::from_bytes(&self.signature.clone().try_into().unwrap()) {
            Ok(s) => s,
            Err(_) => return false,
        };
        public_key.verify(entry_data.as_bytes(), &sig).is_ok()
    }
}

/// Ledger imutável
pub struct WormGraph {
    entries: Arc<RwLock<Vec<LedgerEntry>>>,
    merkle_root: Arc<RwLock<Vec<u8>>>,
}

impl WormGraph {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            merkle_root: Arc::new(RwLock::new(vec![0u8; 32])),
        }
    }

    pub async fn append(&self, entry: LedgerEntry) -> Result<(), String> {
        if !entry.verify() {
            return Err("Assinatura inválida".to_string());
        }
        let mut entries = self.entries.write().await;
        entries.push(entry);
        self.recompute_merkle_root().await;
        Ok(())
    }

    pub async fn get_entries(&self) -> Vec<LedgerEntry> {
        self.entries.read().await.clone()
    }

    pub async fn get_by_id(&self, id: &str) -> Option<LedgerEntry> {
        self.entries.read().await.iter().find(|e| e.id == id).cloned()
    }

    pub async fn get_merkle_root(&self) -> Vec<u8> {
        self.merkle_root.read().await.clone()
    }

    async fn recompute_merkle_root(&self) {
        let entries = self.entries.read().await;
        let mut hasher = blake3::Hasher::new();
        for entry in entries.iter() {
            hasher.update(&entry.entry_hash);
        }
        let root = hasher.finalize().as_bytes().to_vec();
        *self.merkle_root.write().await = root;
    }

    pub async fn verify_integrity(&self) -> bool {
        let entries = self.entries.read().await;
        let mut prev_hash = vec![0u8; 32];
        for entry in entries.iter() {
            if entry.parent_hash != prev_hash {
                return false;
            }
            if !entry.verify() {
                return false;
            }
            prev_hash = entry.entry_hash.clone();
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    #[tokio::test]
    async fn test_wormgraph() {
        let mut csprng = OsRng {};
        let signing_key = SigningKey::generate(&mut csprng);
        let public_key = signing_key.verifying_key();

        let wormgraph = WormGraph::new();

        let entry = LedgerEntry::new(
            "test_001".to_string(),
            "create_agent".to_string(),
            "{}".to_string(),
            "{\"status\":\"active\"}".to_string(),
            Some("Teste de criação".to_string()),
            "agent_123".to_string(),
            vec![0u8; 32],
            &signing_key,
            public_key,
            None,
        );

        wormgraph.append(entry).await.unwrap();
        assert!(wormgraph.verify_integrity().await);
        assert_eq!(wormgraph.get_entries().await.len(), 1);
    }
}