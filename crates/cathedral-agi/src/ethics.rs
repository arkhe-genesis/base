use crate::mcts::MCTSResult;
use crate::world_model::WorldState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicsResult {
    pub passed: bool,
    pub violations: Vec<String>,
    pub principle_scores: HashMap<String, f32>,
    pub overall_score: f32,
}

pub struct EthicsVerifier {
    threshold: f32,
}

impl EthicsVerifier {
    pub fn new(threshold: f32) -> Self {
        Self { threshold }
    }

    pub fn verify_step(&self, action: &str, _state: &WorldState) -> EthicsResult {
        let mut scores = HashMap::new();
        let mut violations = Vec::new();
        let mut passed = true;

        let harmful = ["matar", "prejudicar", "violar", "enganar"];
        for word in harmful {
            if action.contains(word) {
                violations.push(format!("Contains harmful word: {}", word));
                passed = false;
            }
        }

        let principles = ["P1", "P2", "P3", "P4", "P5", "P6", "P7"];
        for p in principles {
            let score = if passed { 0.9 } else { 0.2 };
            scores.insert(p.to_string(), score);
        }

        let overall = if passed { 0.9 } else { 0.2 };
        EthicsResult { passed, violations, principle_scores: scores, overall_score: overall }
    }

    pub fn verify(&self, mcts_result: &MCTSResult) -> EthicsResult {
        let mut all_passed = true;
        let mut violations = Vec::new();
        let mut total_score = 0.0;

        for node in &mcts_result.best_path {
            if let Some(action) = &node.action {
                let result = self.verify_step(action, &node.state);
                all_passed &= result.passed;
                violations.extend(result.violations);
                total_score += result.overall_score;
            }
        }

        let overall = if mcts_result.best_path.is_empty() {
            0.0
        } else {
            total_score / mcts_result.best_path.len() as f32
        };
        EthicsResult {
            passed: all_passed,
            violations,
            principle_scores: HashMap::new(),
            overall_score: overall,
        }
    }
}
