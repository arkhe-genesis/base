use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use tokio::sync::RwLock;

use crate::types::{SchedulerStats, WorkerProfile, WorkerTier};

pub struct WorkerRegistry {
    workers: Arc<RwLock<HashMap<String, WorkerProfile>>>,
}

impl WorkerRegistry {
    pub fn new() -> Self {
        Self { workers: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn register(&self, profile: WorkerProfile) -> Result<()> {
        let mut map = self.workers.write().await;
        map.insert(profile.worker_id.clone(), profile);
        Ok(())
    }

    pub async fn update(&self, worker_id: &str, profile: WorkerProfile) -> Result<()> {
        let mut map = self.workers.write().await;
        if map.contains_key(worker_id) {
            map.insert(worker_id.to_string(), profile);
            Ok(())
        } else {
            anyhow::bail!("Worker not found: {}", worker_id)
        }
    }

    pub async fn remove(&self, worker_id: &str) -> Result<()> {
        let mut map = self.workers.write().await;
        if map.remove(worker_id).is_some() {
            Ok(())
        } else {
            anyhow::bail!("Worker not found: {}", worker_id)
        }
    }

    pub async fn get(&self, worker_id: &str) -> Option<WorkerProfile> {
        let map = self.workers.read().await;
        map.get(worker_id).cloned()
    }

    pub async fn list(&self, tier: Option<WorkerTier>) -> Vec<WorkerProfile> {
        let map = self.workers.read().await;
        map.values().filter(|p| tier.as_ref().map_or(true, |t| &p.tier == t)).cloned().collect()
    }

    pub async fn available(&self) -> Vec<WorkerProfile> {
        let map = self.workers.read().await;
        map.values().filter(|p| p.available && p.reputation >= 0.7).cloned().collect()
    }

    pub async fn stats(&self) -> SchedulerStats {
        let map = self.workers.read().await;
        let total = map.len();
        let depin_gpu = map.values().filter(|p| p.tier == WorkerTier::DePIN_GPU).count();
        let depin_cpu = map.values().filter(|p| p.tier == WorkerTier::DePIN_CPU).count();
        let datacenter = map.values().filter(|p| p.tier == WorkerTier::Datacenter).count();
        let avg_reputation = if total > 0 {
            map.values().map(|p| p.reputation).sum::<f32>() / total as f32
        } else {
            0.0
        };
        let active = map.values().filter(|p| p.available).count();
        SchedulerStats {
            total_workers: total,
            depin_gpu_count: depin_gpu,
            depin_cpu_count: depin_cpu,
            datacenter_count: datacenter,
            avg_reputation,
            active_workers: active,
        }
    }

    pub async fn increment_tasks(&self, worker_id: &str, success: bool) -> Result<()> {
        let mut map = self.workers.write().await;
        if let Some(p) = map.get_mut(worker_id) {
            if success {
                p.tasks_completed += 1;
                p.reputation = (p.reputation + 0.01).min(1.0);
            } else {
                p.tasks_failed += 1;
                p.reputation = (p.reputation - 0.02).max(0.0);
            }
            Ok(())
        } else {
            anyhow::bail!("Worker not found: {}", worker_id)
        }
    }
}
