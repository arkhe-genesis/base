//! Load balancing for MoE.

use crate::tensor::Tensor;
use crate::moe::router::RoutingIndex;

/// Maintains load per expert and applies capacity factor.
pub struct LoadBalancer {
    pub capacity_factor: f32,
    pub expert_loads: Vec<usize>,
}

impl LoadBalancer {
    pub fn new(capacity_factor: f32, num_experts: usize) -> Self {
        Self {
            capacity_factor,
            expert_loads: vec![0; num_experts],
        }
    }

    /// Apply capacity factor: drop tokens if an expert is overloaded.
    pub fn apply(
        &mut self,
        routing: &[Vec<RoutingIndex>],
    ) -> Vec<(usize, usize, f32)> {
        let mut result = Vec::new();
        let capacity = (self.capacity_factor * (routing.len() as f32 / self.expert_loads.len() as f32)) as usize;

        // Reset loads
        for load in self.expert_loads.iter_mut() {
            *load = 0;
        }

        for (token_idx, indices) in routing.iter().enumerate() {
            // Try to assign to the highest-weight expert that still has capacity
            for routing_idx in indices {
                let expert_id = routing_idx.expert_id;
                let weight = routing_idx.weight;
                if self.expert_loads[expert_id] < capacity {
                    self.expert_loads[expert_id] += 1;
                    result.push((token_idx, expert_id, weight));
                    break;
                }
            }
        }

        result
    }

    /// Compute load balancing loss (auxiliary loss).
    pub fn compute_loss(&self, total_tokens: usize) -> f32 {
        let num_experts = self.expert_loads.len();
        let ideal = total_tokens as f32 / num_experts as f32;
        let sum_sq: f32 = self.expert_loads.iter().map(|&l| (l as f32 - ideal).powi(2)).sum();
        sum_sq / (total_tokens as f32 * total_tokens as f32)
    }
}
