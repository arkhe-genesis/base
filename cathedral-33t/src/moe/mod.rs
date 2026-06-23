//! Mixture of Experts (MoE) core

pub mod expert;
pub mod router;
pub mod load_balancer;

pub use expert::Expert;
pub use router::{HierarchicalRouter, RoutingIndex};
pub use load_balancer::LoadBalancer;

use crate::tensor::Tensor;
use crate::config::MoEConfig;

/// MoE Layer com 4096 experts
pub struct MoELayer {
    pub experts: Vec<Expert>,
    pub router: HierarchicalRouter,
    pub load_balancer: LoadBalancer,
    pub num_experts: usize,
    pub top_k: usize,
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub load_balancing_coef: f32,
}

impl MoELayer {
    pub fn new(config: &MoEConfig) -> Self {
        let experts: Vec<Expert> = (0..config.num_experts)
            .map(|_| Expert::new(config.hidden_size, config.intermediate_size))
            .collect();

        let router = HierarchicalRouter::new(
            config.num_experts,
            config.top_k,
            config.hidden_size,
        );

        Self {
            experts,
            router,
            load_balancer: LoadBalancer::new(config.capacity_factor, config.num_experts),
            num_experts: config.num_experts,
            top_k: config.top_k,
            hidden_size: config.hidden_size,
            intermediate_size: config.intermediate_size,
            load_balancing_coef: config.load_balancing_loss_coef,
        }
    }

    /// Forward pass do MoE
    pub fn forward(&mut self, x: &Tensor) -> (Tensor, f32) {
        let batch_size = x.nrows();

        // 1. Router seleciona top-k experts para cada token
        let routing_indices = self.router.route(x);

        // 2. Load balancing (capacity factor + auxiliary loss)
        let balanced_indices = self.load_balancer.apply(&routing_indices);
        let aux_loss = self.load_balancer.compute_loss(batch_size);

        // 3. Dispatch tokens para experts
        let mut expert_outputs: Vec<(usize, usize, f32, Tensor)> = Vec::new();

        for (token_idx, expert_id, weight) in &balanced_indices {
            let token = x.slice_row(*token_idx);
            let output = self.experts[*expert_id].forward(&token);
            expert_outputs.push((*token_idx, *expert_id, *weight, output));
        }

        // 4. Combinar outputs (weighted sum)
        let combined = self.combine(expert_outputs, batch_size);

        (combined, aux_loss)
    }

    /// Combinar outputs dos experts com weighted sum
    fn combine(
        &self,
        outputs: Vec<(usize, usize, f32, Tensor)>,
        batch_size: usize,
    ) -> Tensor {
        // Inicializar resultado com zeros
        let mut result = Tensor::zeros((batch_size, self.hidden_size));

        // Agrupar por token_idx
        let mut token_outputs: Vec<Vec<(f32, Tensor)>> = vec![Vec::new(); batch_size];

        for (token_idx, _expert_id, weight, output) in outputs {
            token_outputs[token_idx].push((weight, output));
        }

        // Computar weighted sum para cada token
        for (token_idx, experts_for_token) in token_outputs.iter().enumerate() {
            if experts_for_token.is_empty() {
                continue; // Token não foi roteado (edge case)
            }

            // Normalizar pesos
            let total_weight: f32 = experts_for_token.iter().map(|(w, _)| w.abs()).sum();
            let normalized_weight = if total_weight > 0.0 {
                1.0 / total_weight
            } else {
                1.0
            };

            let mut token_result = Tensor::zeros((1, self.hidden_size));
            for (weight, output) in experts_for_token {
                let scaled = output.scale(*weight * normalized_weight);
                token_result = token_result.add(&scaled);
            }

            // Atualizar resultado no batch
            for j in 0..self.hidden_size {
                result.data[[token_idx, j]] = token_result.data[[0, j]];
            }
        }

        result
    }
}
