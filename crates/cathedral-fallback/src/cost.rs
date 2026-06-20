use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRecord {
    pub timestamp: i64,
    pub cost_usd: f64,
    pub latency_ms: u64,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationStats {
    pub total_tasks: u64,
    pub avg_cost_per_task: f64,
    pub avg_latency_ms: u64,
    pub p95_latency_ms: u64,
    pub cost_saved_usd: f64,
    pub depin_usage_pct: f32,
}

pub struct CostOptimizer {
    history: VecDeque<CostRecord>,
    max_history: usize,
    target_latency_p95_ms: u64,
    depin_threshold: u64,
    depin_counter: u64,
}

impl CostOptimizer {
    pub fn new(max_history: usize, target_latency_p95_ms: u64) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            max_history,
            target_latency_p95_ms,
            depin_threshold: 10,
            depin_counter: 0,
        }
    }

    pub fn record_task(&mut self, cost_usd: f64, latency_ms: u64, success: bool) {
        self.history.push_back(CostRecord {
            timestamp: chrono::Utc::now().timestamp(),
            cost_usd,
            latency_ms,
            success,
        });
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    pub fn should_use_depin(&mut self) -> bool {
        let current_p95 = self.calculate_p95_latency();
        if current_p95 > self.target_latency_p95_ms {
            return false;
        }

        self.depin_counter += 1;
        if self.depin_counter >= self.depin_threshold {
            self.depin_counter = 0;
            return true;
        }

        let avg_latency = self.history.iter()
            .filter(|r| r.success)
            .map(|r| r.latency_ms)
            .take(10)
            .sum::<u64>() / 10.min(self.history.len()) as u64;

        avg_latency < 1000
    }

    fn calculate_p95_latency(&self) -> u64 {
        let mut latencies: Vec<u64> = self.history
            .iter()
            .filter(|r| r.success)
            .map(|r| r.latency_ms)
            .collect();
        if latencies.is_empty() { return 0; }
        latencies.sort_unstable();
        let idx = (latencies.len() as f64 * 0.95) as usize;
        latencies.get(idx.min(latencies.len() - 1)).copied().unwrap_or(0)
    }

    pub fn stats(&self) -> OptimizationStats {
        let total = self.history.len() as u64;
        let total_cost: f64 = self.history.iter().map(|r| r.cost_usd).sum();
        let avg_cost = if total > 0 { total_cost / total as f64 } else { 0.0 };
        let avg_latency = if total > 0 {
            self.history.iter().map(|r| r.latency_ms).sum::<u64>() / total
        } else { 0 };
        let depin_count = self.history.iter()
            .filter(|r| r.cost_usd < 0.001)
            .count();
        let depin_pct = if total > 0 { (depin_count as f32 / total as f32) * 100.0 } else { 0.0 };

        OptimizationStats {
            total_tasks: total,
            avg_cost_per_task: avg_cost,
            avg_latency_ms: avg_latency,
            p95_latency_ms: self.calculate_p95_latency(),
            cost_saved_usd: 0.0,
            depin_usage_pct: depin_pct,
        }
    }
}
