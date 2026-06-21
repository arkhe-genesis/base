//! Anticipatory Routing — loss spike prevention with circular buffer.

use crate::tensor::Tensor;
use std::sync::{Arc, RwLock};
use crate::moe::RoutingIndex;

pub struct AnticipatoryRouter {
    routing_index: Arc<RwLock<Option<RoutingIndex>>>,
    loss_spike_detector: LossSpikeDetector,
    routing_delay: usize,
    delayed_features: Vec<Tensor>,
    features_index: usize,
}

impl AnticipatoryRouter {
    pub fn new(routing_delay: usize) -> Self {
        Self {
            routing_index: Arc::new(RwLock::new(None)),
            loss_spike_detector: LossSpikeDetector::new(0.1),
            routing_delay,
            delayed_features: Vec::with_capacity(routing_delay + 1),
            features_index: 0,
        }
    }

    pub fn route_with_anticipation(&mut self, token: &Tensor) -> Vec<usize> {
        // If a loss spike is detected, recompute routing index.
        if self.loss_spike_detector.detect() {
            let index = RoutingIndex {
                expert_id: 0, // placeholder: in real impl would run full routing
                weight: 1.0,
            };
            *self.routing_index.write().unwrap() = Some(index);
        }

        // Use delayed features if available.
        let _delayed = self.get_delayed_feature(token);

        // Use cached routing index if set, else use direct routing.
        if let Some(index) = self.routing_index.read().unwrap().as_ref() {
            vec![index.expert_id]
        } else {
            // Fallback: simple routing (select expert 0).
            vec![0]
        }
    }

    fn get_delayed_feature(&mut self, token: &Tensor) -> Tensor {
        // Store current token in circular buffer.
        self.delayed_features.push(token.clone());
        if self.delayed_features.len() > self.routing_delay + 1 {
            self.delayed_features.remove(0);
        }

        let idx = self.features_index % self.delayed_features.len();
        self.features_index += 1;
        self.delayed_features[idx].clone()
    }
}

struct LossSpikeDetector {
    threshold: f32,
    last_loss: f32,
}

impl LossSpikeDetector {
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold,
            last_loss: 0.0,
        }
    }

    pub fn detect(&mut self) -> bool {
        // In production: monitor training loss.
        // For now, always false (can be overridden by setting last_loss).
        false
    }
}
