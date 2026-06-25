use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    sqlite_storage::SqliteStorage,
    types::{EpisodicEntry, Ordering, VectorClock},
};

pub struct EpisodicSync {
    storage: Arc<SqliteStorage>,
    worker_id: String,
    cache: Arc<RwLock<Vec<EpisodicEntry>>>,
}

impl EpisodicSync {
    pub async fn new(worker_id: String, database_url: &str) -> Result<Self> {
        let storage = Arc::new(SqliteStorage::new(database_url).await?);
        let entries = storage.list_all().await?;
        Ok(Self { storage, worker_id, cache: Arc::new(RwLock::new(entries)) })
    }

    pub async fn upsert(
        &self,
        user_input: &str,
        assistant_output: &str,
        confidence: f32,
    ) -> Result<String> {
        let id = format!("ep-{}", Uuid::new_v4());
        let now = Utc::now().timestamp();
        let mut clock = VectorClock::new();
        clock.increment(&self.worker_id);

        let entry = EpisodicEntry {
            id: id.clone(),
            user_input: user_input.to_string(),
            assistant_output: assistant_output.to_string(),
            timestamp: now,
            version: 1,
            vector_clock: clock,
            worker_id: self.worker_id.clone(),
            confidence,
            deleted: false,
        };

        self.storage.upsert(&entry).await?;
        let mut cache = self.cache.write().await;
        cache.push(entry);
        Ok(id)
    }

    pub async fn merge(&self, remote: EpisodicEntry) -> Result<()> {
        let mut cache = self.cache.write().await;
        if let Some(local) = cache.iter_mut().find(|e| e.id == remote.id) {
            let cmp = local.vector_clock.compare(&remote.vector_clock);
            match cmp {
                Ordering::Less => {
                    *local = remote;
                }
                Ordering::Concurrent => {
                    local.assistant_output =
                        format!("{} [merged] {}", local.assistant_output, remote.assistant_output);
                    local.confidence = (local.confidence + remote.confidence) / 2.0;
                    local.version += 1;
                    local.vector_clock.merge(&remote.vector_clock);
                    local.vector_clock.increment(&self.worker_id);
                }
                _ => {}
            }
            self.storage.upsert(local).await?;
        } else {
            cache.push(remote.clone());
            self.storage.upsert(&remote).await?;
        }
        Ok(())
    }

    pub async fn retrieve(&self, query: &str, limit: usize) -> Vec<EpisodicEntry> {
        let cache = self.cache.read().await;
        let mut entries: Vec<EpisodicEntry> =
            cache.iter().filter(|e| !e.deleted).cloned().collect();
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        entries.truncate(limit);
        entries
    }

    pub async fn get_snapshot(&self) -> Vec<EpisodicEntry> {
        self.cache.read().await.clone()
    }
}
