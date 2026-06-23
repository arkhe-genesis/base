use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;
use sha3::{Sha3_256, Digest};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MemoryEntry {
    pub did: String,
    pub content: String,
    pub thinking: Option<String>,
    pub signature: Vec<u8>,
    pub timestamp: i64,
    pub embedding: Option<Vec<f32>>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ExecutionReceipt {
    pub id: String,
    pub merkle_root: String,
    pub timestamp: i64,
}

#[derive(Debug, Error)]
pub enum WormGraphError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Agent not found")]
    NotFound,
}

pub struct WormGraphClient {
    store: Arc<DashMap<String, Vec<MemoryEntry>>>,
    storage_path: PathBuf,
}

impl WormGraphClient {
    pub fn new() -> Self {
        let path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".cathedral/memory.json");
        let store = Arc::new(DashMap::new());
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(map) = serde_json::from_str::<HashMap<String, Vec<MemoryEntry>>>(&data) {
                    for (k, v) in map { store.insert(k, v); }
                }
            }
        }
        Self { store, storage_path: path }
    }

    fn persist(&self) {
        let snapshot: HashMap<String, Vec<MemoryEntry>> = self.store
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();
        if let Some(parent) = self.storage_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&snapshot) {
            let _ = fs::write(&self.storage_path, json);
        }
    }

    pub async fn get_memories(&self, did: &str, limit: usize) -> Result<Vec<MemoryEntry>, WormGraphError> {
        let entries = self.store.get(did);
        let mut vec = entries.map(|e| e.value().clone()).unwrap_or_default();
        vec.sort_by_key(|e| -e.timestamp);
        Ok(vec.into_iter().take(limit).collect())
    }

    pub async fn record(&self, did: &str, content: &str, thinking: &Option<String>, signature: &[u8]) -> Result<ExecutionReceipt, WormGraphError> {
        let entry = MemoryEntry {
            did: did.to_string(),
            content: content.to_string(),
            thinking: thinking.clone(),
            signature: signature.to_vec(),
            timestamp: Utc::now().timestamp(),
            embedding: None,
        };
        self.store.entry(did.to_string()).or_insert_with(Vec::new).push(entry);
        self.persist();
        let receipt = ExecutionReceipt {
            id: uuid::Uuid::new_v4().to_string(),
            merkle_root: format!("0x{:x}", Sha3_256::digest(b"mock")),
            timestamp: Utc::now().timestamp(),
        };
        Ok(receipt)
    }

    pub async fn search_similar(&self, did: &str, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, WormGraphError> {
        let entries = self.get_memories(did, 100).await?;
        let query_lower = query.to_lowercase();
        let mut scored: Vec<_> = entries.into_iter().filter_map(|e| {
            if e.content.to_lowercase().contains(&query_lower) { Some((e, 1)) } else { None }
        }).collect();
        scored.sort_by_key(|(_, score)| -score);
        let results = scored.into_iter().take(limit).map(|(e, _)| e).collect();
        Ok(results)
    }
}
