//! Hierarchical Router for 4096 experts with deduplication.

use crate::tensor::Tensor;
use std::collections::HashSet;

/// Routing index for a token (expert id + weight)
#[derive(Debug, Clone)]
pub struct RoutingIndex {
    pub expert_id: usize,
    pub weight: f32,
}

/// Hierarchical Gating Network: 64 groups × 64 experts
pub struct HierarchicalRouter {
    pub num_groups: usize,
    pub experts_per_group: usize,
    pub top_k: usize,
    pub hidden_size: usize,
    pub group_weights: Tensor,   // [64, hidden_size]
    pub expert_weights: Tensor,  // [64, 64, hidden_size]
}

impl HierarchicalRouter {
    pub fn new(num_experts: usize, top_k: usize, hidden_size: usize) -> Self {
        let num_groups = 64;
        let experts_per_group = num_experts / num_groups;

        let group_weights = Tensor::randn((num_groups, hidden_size));
        let expert_weights = Tensor::randn((num_groups * experts_per_group, hidden_size));

        Self {
            num_groups,
            experts_per_group,
            top_k,
            hidden_size,
            group_weights,
            expert_weights,
        }
    }

    /// Route a batch of tokens to top‑k experts.
    pub fn route(&self, x: &Tensor) -> Vec<Vec<RoutingIndex>> {
        let batch_size = x.nrows();
        let mut routing = Vec::with_capacity(batch_size);

        for i in 0..batch_size {
            let token = x.slice_row(i);
            let entry = self.route_single(&token);
            routing.push(entry);
        }

        routing
    }

    /// Route a single token.
    fn route_single(&self, token: &Tensor) -> Vec<RoutingIndex> {
        // Compute group logits: token ⊗ group_weights
        // token: [1, hidden_size]
        // group_weights: [64, hidden_size]
        // result: [1, 64]
        let group_logits = token.matmul(&self.group_weights.transpose()); // [1, 64]
        let group_logits = group_logits.slice_row(0).to_vec();

        // Select top‑2 groups (unique)
        let top_groups = self.top_k_indices(&group_logits, 2);

        let mut expert_indices = Vec::with_capacity(self.top_k);
        let mut seen = HashSet::new();

        // For each group, select top‑4 experts
        let experts_per_group = (self.top_k + 1) / 2; // 4 for top_k=8

        for &(group_idx, _) in &top_groups {
            // Expert weights for this group: slice from flattened tensor
            let start = group_idx * self.experts_per_group;
            let end = start + self.experts_per_group;
            let group_expert_weights = Tensor::from(
                self.expert_weights
                    .data
                    .slice(ndarray::s![start..end, ..])
                    .to_owned(),
            );

            let expert_logits = token.matmul(&group_expert_weights.transpose());
            let expert_logits = expert_logits.slice_row(0).to_vec();
            let top_experts = self.top_k_indices(&expert_logits, experts_per_group);

            for (idx, weight) in top_experts {
                let expert_id = group_idx * self.experts_per_group + idx;
                if seen.insert(expert_id) {
                    expert_indices.push(RoutingIndex { expert_id, weight });
                }
            }
        }

        // Ensure exactly top_k experts
        if expert_indices.len() > self.top_k {
            expert_indices.truncate(self.top_k);
        }

        expert_indices
    }

    /// Returns indices of top k values with their weights.
    fn top_k_indices(&self, values: &[f32], k: usize) -> Vec<(usize, f32)> {
        let mut indexed: Vec<_> = values.iter().enumerate().map(|(i, &v)| (v, i)).collect();
        indexed.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        indexed
            .into_iter()
            .take(k)
            .map(|(v, i)| (i, v))
            .collect()
    }
}
