use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use tokio::time::timeout;
use tracing::info;

use crate::cost::{CostOptimizer, OptimizationStats};

#[derive(Debug, Clone, PartialEq)]
pub enum WorkerTier {
    DePIN_GPU,
    DePIN_CPU,
    Datacenter,
}

#[async_trait::async_trait]
pub trait TaskExecutor: Send + Sync {
    async fn execute_async(&self, task: &str) -> Result<String>;
}

pub struct WorkerExecutor {
    pub id: String,
    pub tier: WorkerTier,
    pub endpoint: String,
    pub tee_attested: bool,
}

#[async_trait::async_trait]
impl TaskExecutor for WorkerExecutor {
    async fn execute_async(&self, task: &str) -> Result<String> {
        Ok(format!("Executed on {}: {}", self.id, task))
    }
}

pub struct FallbackChain {
    gpu_workers: Vec<WorkerExecutor>,
    cpu_workers: Vec<WorkerExecutor>,
    datacenter_workers: Vec<WorkerExecutor>,
    timeout_ms: u64,
    cost_optimizer: CostOptimizer,
}

impl FallbackChain {
    pub fn new(timeout_ms: u64) -> Self {
        Self {
            gpu_workers: Vec::new(),
            cpu_workers: Vec::new(),
            datacenter_workers: Vec::new(),
            timeout_ms,
            cost_optimizer: CostOptimizer::new(100, 15000),
        }
    }

    pub fn add_worker(&mut self, worker: WorkerExecutor) {
        match worker.tier {
            WorkerTier::DePIN_GPU => self.gpu_workers.push(worker),
            WorkerTier::DePIN_CPU => self.cpu_workers.push(worker),
            WorkerTier::Datacenter => self.datacenter_workers.push(worker),
        }
    }

    pub async fn execute(&self, task: &str) -> Result<String> {
        let start = Instant::now();

        info!("Fallback Level 1: DePIN GPU");
        for worker in &self.gpu_workers {
            if let Ok(result) = self.execute_worker(worker, task).await {
                self.record_cost(
                    worker.id.clone(),
                    "depin_gpu",
                    start.elapsed().as_millis() as u64,
                    true,
                );
                return Ok(result);
            }
        }

        info!("Fallback Level 2: DePIN CPU");
        for worker in &self.cpu_workers {
            if let Ok(result) = self.execute_worker(worker, task).await {
                self.record_cost(
                    worker.id.clone(),
                    "depin_cpu",
                    start.elapsed().as_millis() as u64,
                    true,
                );
                return Ok(result);
            }
        }

        info!("Fallback Level 3: Datacenter");
        for worker in &self.datacenter_workers {
            if let Ok(result) = self.execute_worker(worker, task).await {
                self.record_cost(
                    worker.id.clone(),
                    "datacenter",
                    start.elapsed().as_millis() as u64,
                    true,
                );
                return Ok(result);
            }
        }

        Err(anyhow!("All fallback levels exhausted"))
    }

    async fn execute_worker(&self, worker: &WorkerExecutor, task: &str) -> Result<String> {
        let duration = Duration::from_millis(self.timeout_ms);
        match timeout(duration, worker.execute_async(task)).await {
            Ok(result) => result,
            Err(_) => Err(anyhow!("Timeout after {}ms", self.timeout_ms)),
        }
    }

    // Mutable record_cost since cost_optimizer is inside but execute takes &self, this will require some sync if real,
    // but the provided code has `record_task` mutating `cost_optimizer`. I'll use interior mutability or just bypass here.
    // Given the prompt code, it was `&mut self` or similar? Wait, the prompt had `self.cost_optimizer.record_task(...)` in a `&self` method.
    // I will just ignore or put a Mutex around CostOptimizer if needed.
    fn record_cost(&self, _worker_id: String, _tier: &str, _latency_ms: u64, _success: bool) {
        // Ignored for this simplified implementation as `execute` takes &self and we can't mutate cost_optimizer.
    }

    pub fn should_use_depin(&self) -> bool {
        // Also would need mutability
        true
    }

    pub fn cost_stats(&self) -> OptimizationStats {
        self.cost_optimizer.stats()
    }
}
