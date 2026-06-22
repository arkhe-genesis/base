//! Hybrid-EP communication strategy

use crate::tensor::Tensor;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommunicationStrategy {
    AllToAll,
    AllGatherReduceScatter,
}

pub struct HybridEP {
    num_devices: usize,
    experts_per_device: usize,
    top_k: usize,
}

impl HybridEP {
    pub fn new(num_devices: usize, num_experts: usize, top_k: usize) -> Self {
        let experts_per_device = (num_experts + num_devices - 1) / num_devices;
        Self {
            num_devices,
            experts_per_device,
            top_k,
        }
    }

    pub fn choose_strategy(&self) -> CommunicationStrategy {
        if self.top_k > 6 && self.top_k > self.experts_per_device {
            CommunicationStrategy::AllGatherReduceScatter
        } else {
            CommunicationStrategy::AllToAll
        }
    }

    pub fn communicate(&self, tokens: &Tensor, _expert_assignments: &[usize]) -> Tensor {
        match self.choose_strategy() {
            CommunicationStrategy::AllToAll => tokens.clone(),
            CommunicationStrategy::AllGatherReduceScatter => tokens.clone(),
        }
    }

    pub fn strategy_name(&self) -> &'static str {
        match self.choose_strategy() {
            CommunicationStrategy::AllToAll => "all-to-all",
            CommunicationStrategy::AllGatherReduceScatter => "all-gather+reduce-scatter",
        }
    }
}
