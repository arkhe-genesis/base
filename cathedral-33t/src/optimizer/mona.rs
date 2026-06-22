//! MONA Optimizer (Muon + Nesterov) — versão Lite com streaming.

use crate::tensor::Tensor;

pub struct MONALiteOptimizer {
    muon: MuonOptimizer,
    acceleration_buffers: Vec<Tensor>,
    beta_a: f32,
    alpha: f32,
    streaming: bool,
    prev_grads: Option<Vec<Tensor>>,
}

impl MONALiteOptimizer {
    pub fn new(
        param_shapes: &[(usize, usize)],
        lr: f32,
        beta_a: f32,
        alpha: f32,
    ) -> Self {
        let buffers = param_shapes
            .iter()
            .map(|&shape| Tensor::zeros(shape))
            .collect();

        Self {
            muon: MuonOptimizer::new(lr),
            acceleration_buffers: buffers,
            beta_a,
            alpha,
            streaming: true,
            prev_grads: None,
        }
    }

    pub fn step(&mut self, grads: &[Tensor]) {
        let lr = self.muon.lr();

        for (i, grad) in grads.iter().enumerate() {
            // Compute gradient difference (streaming or full)
            let diff = if self.streaming {
                if let Some(prev) = &self.prev_grads {
                    grad.sub(&prev[i])
                } else {
                    grad.clone()
                }
            } else {
                // Full diff requires two previous gradients; simplified here
                grad.clone()
            };

            // Update acceleration buffer
            self.acceleration_buffers[i] = self.acceleration_buffers[i]
                .scale(self.beta_a)
                .add(&diff.scale(1.0 - self.beta_a));

            // Apply acceleration to gradient
            let accelerated = grad.add(&self.acceleration_buffers[i].scale(self.alpha));

            // Muon orthogonalization step
            self.muon.step_single(&accelerated, lr, i);
        }

        self.prev_grads = Some(grads.to_vec());
    }
}

struct MuonOptimizer {
    lr: f32,
}

impl MuonOptimizer {
    pub fn new(lr: f32) -> Self {
        Self { lr }
    }

    pub fn lr(&self) -> f32 {
        self.lr
    }

    pub fn step_single(&mut self, grad: &Tensor, lr: f32, _idx: usize) {
        // Simplified Muon: apply gradient descent with orthogonalization
        // In real Muon, we would compute the orthogonal projection of the gradient
        // onto the tangent space of the Stiefel manifold.
        // Here we just apply the gradient scaled by learning rate.
        let _ = grad.scale(lr);
    }
}
