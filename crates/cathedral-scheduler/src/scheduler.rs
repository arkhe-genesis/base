use crate::types::{TaskType, WorkerTier, SchedulingDecision, SchedulerStats};
use crate::registry::WorkerRegistry;
use crate::metrics::SchedulerMetrics;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct HybridScheduler {
    registry: Arc<WorkerRegistry>,
    metrics: SchedulerMetrics,
    cost_weights: HashMap<TaskType, f32>,
    latency_weights: HashMap<TaskType, f32>,
    max_latency_ms: u64,
    min_reputation: f32,
    min_stake_sats: u64,
}

impl HybridScheduler {
    pub fn new(
        registry: Arc<WorkerRegistry>,
        max_latency_ms: u64,
        min_reputation: f32,
        min_stake_sats: u64,
    ) -> Self {
        let mut cost_weights = HashMap::new();
        let mut latency_weights = HashMap::new();

        cost_weights.insert(TaskType::Inference, 0.5);
        latency_weights.insert(TaskType::Inference, 0.5);
        cost_weights.insert(TaskType::MCTS, 0.3);
        latency_weights.insert(TaskType::MCTS, 0.7);
        cost_weights.insert(TaskType::Compression, 0.7);
        latency_weights.insert(TaskType::Compression, 0.3);
        cost_weights.insert(TaskType::WorldModel, 0.5);
        latency_weights.insert(TaskType::WorldModel, 0.5);
        cost_weights.insert(TaskType::EthicsCheck, 0.2);
        latency_weights.insert(TaskType::EthicsCheck, 0.8);
        cost_weights.insert(TaskType::MemorySync, 0.8);
        latency_weights.insert(TaskType::MemorySync, 0.2);

        Self {
            registry,
            metrics: SchedulerMetrics::new(),
            cost_weights,
            latency_weights,
            max_latency_ms,
            min_reputation,
            min_stake_sats,
        }
    }

    pub async fn schedule(&self, task_type: TaskType) -> SchedulingDecision {
        let task_id = format!("task-{}", Uuid::new_v4());
        let candidates = self.registry.available().await;

        self.metrics.record_schedule_attempt();

        if candidates.is_empty() {
            self.metrics.record_schedule_failure();
            return SchedulingDecision {
                task_id,
                task_type,
                selected_worker: "datacenter-local".to_string(),
                selected_tier: WorkerTier::Datacenter,
                estimated_cost: 0.001,
                estimated_latency_ms: 100,
                reason: "No DePIN workers available, using datacenter".to_string(),
            };
        }

        let cost_weight = self.cost_weights.get(&task_type).unwrap_or(&0.5);
        let latency_weight = self.latency_weights.get(&task_type).unwrap_or(&0.5);

        let mut best_worker = &candidates[0];
        let mut best_score = f32::NEG_INFINITY;

        for worker in &candidates {
            let cost_normalized = 1.0 - (worker.cost_per_hour as f32 / 1.0).min(1.0);
            let latency_normalized = 1.0 - (worker.latency_p95_ms as f32 / 5000.0).min(1.0);
            let score = cost_weight * cost_normalized + latency_weight * latency_normalized;

            let latency_penalty = if worker.latency_p95_ms > self.max_latency_ms {
                ((worker.latency_p95_ms - self.max_latency_ms) as f32 / self.max_latency_ms as f32) * 0.5
            } else {
                0.0
            };

            let final_score = score - latency_penalty;

            if final_score > best_score {
                best_score = final_score;
                best_worker = worker;
            }
        }

        let decision = SchedulingDecision {
            task_id,
            task_type,
            selected_worker: best_worker.worker_id.clone(),
            selected_tier: best_worker.tier.clone(),
            estimated_cost: best_worker.cost_per_hour / 3600.0,
            estimated_latency_ms: best_worker.latency_p95_ms,
            reason: format!("Score {:.3}", best_score),
        };

        self.metrics.record_schedule_decision(&decision);
        decision
    }

    pub async fn record_result(&self, worker_id: &str, success: bool) -> Result<()> {
        self.registry.increment_tasks(worker_id, success).await?;
        if success {
            self.metrics.record_task_success();
        } else {
            self.metrics.record_task_failure();
        }
        Ok(())
    }

    pub async fn stats(&self) -> SchedulerStats {
        self.registry.stats().await
    }

    pub fn metrics(&self) -> &SchedulerMetrics {
        &self.metrics
    }
}
