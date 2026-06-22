//! Occult: Expert Placement Optimization

use super::{DeviceId, ExpertId};
use std::collections::{HashMap, HashSet};

pub struct OccultPlacementOptimizer {
    expert_device_map: HashMap<ExpertId, DeviceId>,
    coactivation_matrix: Vec<Vec<f32>>,
    num_devices: usize,
    experts_per_device: usize,
    num_experts: usize,
}

impl OccultPlacementOptimizer {
    pub fn new(num_devices: usize, num_experts: usize) -> Self {
        let experts_per_device = (num_experts + num_devices - 1) / num_devices;
        Self {
            expert_device_map: HashMap::new(),
            coactivation_matrix: vec![vec![0.0; num_experts]; num_experts],
            num_devices,
            experts_per_device,
            num_experts,
        }
    }

    pub fn update_coactivation(&mut self, routing_stats: &[Vec<ExpertId>]) {
        for token_experts in routing_stats {
            for i in 0..token_experts.len() {
                for j in (i + 1)..token_experts.len() {
                    let e1 = token_experts[i];
                    let e2 = token_experts[j];
                    if e1 < self.num_experts && e2 < self.num_experts {
                        self.coactivation_matrix[e1][e2] += 1.0;
                        self.coactivation_matrix[e2][e1] += 1.0;
                    }
                }
            }
        }
    }

    pub fn optimize_placement(&mut self) {
        for expert_id in 0..self.num_experts {
            let device_id = (expert_id / self.experts_per_device).min(self.num_devices - 1);
            self.expert_device_map.insert(expert_id, device_id as u32);
        }

        for _ in 0..100 {
            let mut improved = false;

            for expert_id in 0..self.num_experts {
                let current_device = self.expert_device_map[&expert_id];
                let current_score = self.placement_score(expert_id, current_device);

                for device_id in 0..self.num_devices {
                    if device_id as u32 == current_device {
                        continue;
                    }

                    let new_score = self.placement_score(expert_id, device_id as u32);
                    if new_score > current_score * 1.05 {
                        self.expert_device_map.insert(expert_id, device_id as u32);
                        improved = true;
                        break;
                    }
                }
            }

            if !improved {
                break;
            }
        }
    }

    fn placement_score(&self, expert_id: ExpertId, device_id: DeviceId) -> f32 {
        let mut score = 0.0;
        for (other_id, &other_device) in &self.expert_device_map {
            if *other_id == expert_id {
                continue;
            }
            if other_device == device_id {
                score += self.coactivation_matrix[expert_id][*other_id];
            }
        }
        score
    }

    pub fn route_with_occult(&self, experts: &[ExpertId]) -> Vec<DeviceId> {
        let mut devices = HashSet::new();
        for &expert in experts {
            if let Some(&device) = self.expert_device_map.get(&expert) {
                devices.insert(device);
            }
        }
        devices.into_iter().collect()
    }

    pub fn stats(&self) -> OccultStats {
        let mut device_counts = vec![0; self.num_devices];
        for &device in self.expert_device_map.values() {
            device_counts[device as usize] += 1;
        }

        let max_count = device_counts.iter().copied().max().unwrap_or(0);
        let min_count = device_counts.iter().copied().min().unwrap_or(0);

        OccultStats {
            max_experts_per_device: max_count,
            min_experts_per_device: min_count,
            total_coactivation: self.total_coactivation(),
        }
    }

    fn total_coactivation(&self) -> f32 {
        let mut total = 0.0;
        for i in 0..self.num_experts {
            for j in (i + 1)..self.num_experts {
                if self.expert_device_map.get(&i) == self.expert_device_map.get(&j) {
                    total += self.coactivation_matrix[i][j];
                }
            }
        }
        total
    }
}

#[derive(Debug, Clone)]
pub struct OccultStats {
    pub max_experts_per_device: u32,
    pub min_experts_per_device: u32,
    pub total_coactivation: f32,
}
