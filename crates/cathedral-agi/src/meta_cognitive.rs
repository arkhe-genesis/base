use crate::ethics::EthicsResult;
use crate::mcts::MCTSResult;
use cathedral_episodic::EpisodicSync;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaState {
    pub confidence: f32,
    pub uncertainty_epistemic: f32,
    pub uncertainty_aleatoric: f32,
    pub goal_achievement: f32,
    pub self_corrections_count: u32,
    pub last_ethics_violation: bool,
    pub memory_retrieved_count: usize,
}

pub struct MetaCognitiveLoop {
    state: MetaState,
    history: VecDeque<MetaState>,
    horizon: usize,
    episodic_memory: Option<Arc<EpisodicSync>>,
}

impl MetaCognitiveLoop {
    pub fn new(horizon: usize) -> Self {
        Self {
            state: MetaState {
                confidence: 0.7,
                uncertainty_epistemic: 0.3,
                uncertainty_aleatoric: 0.2,
                goal_achievement: 0.0,
                self_corrections_count: 0,
                last_ethics_violation: false,
                memory_retrieved_count: 0,
            },
            history: VecDeque::with_capacity(horizon),
            horizon,
            episodic_memory: None,
        }
    }

    pub fn with_memory(mut self, memory: Arc<EpisodicSync>) -> Self {
        self.episodic_memory = Some(memory);
        self
    }

    pub async fn update(&mut self, mcts: &MCTSResult, ethics: &EthicsResult, user_input: &str) {
        let confidence_delta = (mcts.total_value - 0.5) * 0.2;
        self.state.confidence = (self.state.confidence + confidence_delta).clamp(0.0, 1.0);

        let epistemic = 1.0 - (mcts.nodes_explored as f32 / (self.horizon * 2) as f32).min(1.0);
        self.state.uncertainty_epistemic = epistemic;

        self.state.goal_achievement = (mcts.best_path.len() as f32 / self.horizon as f32).min(1.0);

        self.state.last_ethics_violation = !ethics.passed;
        if !ethics.passed {
            self.state.self_corrections_count += 1;
        }

        if let Some(mem) = &self.episodic_memory {
            let relevant = mem.retrieve(user_input, 3).await;
            self.state.memory_retrieved_count = relevant.len();
            if !relevant.is_empty() {
                self.state.confidence = (self.state.confidence + 0.03).min(1.0);
                info!("🧠 Retrieved {} relevant memories", relevant.len());
            }
        }

        self.history.push_back(self.state.clone());
        if self.history.len() > self.horizon {
            self.history.pop_front();
        }
    }

    pub fn current_uncertainty(&self) -> f32 {
        (self.state.uncertainty_epistemic + self.state.uncertainty_aleatoric) / 2.0
    }

    pub fn current_state(&self) -> &MetaState {
        &self.state
    }

    pub fn history(&self) -> &VecDeque<MetaState> {
        &self.history
    }
}
