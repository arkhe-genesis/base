use crate::world_model::WorldState;
use crate::llm_client::OllamaClient;
use crate::ethics::EthicsResult;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;
use serde_json::json;

#[derive(Debug, Clone)]
pub struct MCTSNode {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub state: WorldState,
    pub action: Option<String>,
    pub visits: u32,
    pub value: f32,
    pub ethics: Option<EthicsResult>,
    pub children: Vec<u64>,
    pub untried_actions: Vec<String>,
    pub expanded: bool,
    pub step_verified: bool,
}

#[derive(Debug, Clone)]
pub struct MCTSResult {
    pub best_path: Vec<MCTSNode>,
    pub total_value: f32,
    pub nodes_explored: usize,
    pub max_depth: usize,
    pub verification_passed: bool,
    pub action_history: Vec<String>,
}

pub struct MCTSEngine {
    llm: Arc<OllamaClient>,
    exploration_constant: f32,
    max_iterations: usize,
    max_depth: usize,
    system_prompt: String,
}

impl MCTSEngine {
    pub fn new(llm: Arc<OllamaClient>, exploration_constant: f32, max_iterations: usize, max_depth: usize) -> Self {
        Self {
            llm,
            exploration_constant,
            max_iterations,
            max_depth,
            system_prompt: "You are a reasoning engine. Given a problem, generate possible actions to solve it. Return a list of actions as a JSON array of strings. Each action should be a specific, concrete step. Limit to 5 actions.".to_string(),
        }
    }

    pub async fn search(&self, initial_state: &WorldState, abstraction: &str) -> MCTSResult {
        info!("🧠 MCTS starting with abstraction: {}", abstraction);

        let root = MCTSNode {
            id: 0,
            parent_id: None,
            state: initial_state.clone(),
            action: None,
            visits: 0,
            value: 0.0,
            ethics: None,
            children: Vec::new(),
            untried_actions: self.generate_actions(abstraction).await.unwrap_or_else(|_| vec![]),
            expanded: false,
            step_verified: true,
        };

        let mut tree = HashMap::new();
        tree.insert(0, root);
        let mut node_count = 1;

        for _ in 0..self.max_iterations {
            let mut current_id = 0;
            let mut path = Vec::new();
            while let Some(node) = tree.get(&current_id) {
                path.push(current_id);
                if node.expanded && !node.children.is_empty() {
                    current_id = self.select_best_child(&tree, current_id);
                } else {
                    break;
                }
            }

            if let Some(node) = tree.get_mut(&current_id) {
                if !node.untried_actions.is_empty() {
                    let action = node.untried_actions.pop().unwrap();
                    let new_id = node_count;
                    node_count += 1;
                    let child_state = self.simulate_action(&node.state, &action).await;
                    let child = MCTSNode {
                        id: new_id,
                        parent_id: Some(current_id),
                        state: child_state,
                        action: Some(action.clone()),
                        visits: 0,
                        value: 0.0,
                        ethics: None,
                        children: Vec::new(),
                        untried_actions: vec![],
                        expanded: false,
                        step_verified: true,
                    };
                    tree.insert(new_id, child);
                    tree.get_mut(&current_id).unwrap().children.push(new_id);
                    current_id = new_id;
                    path.push(current_id);
                }
            }

            let mut reward = 0.0;
            let mut depth = 0;
            let mut current_state = tree.get(&current_id).unwrap().state.clone();
            while depth < self.max_depth {
                let action = self.rollout_action(&current_state).await;
                reward += 0.5;
                depth += 1;
            }

            for &node_id in path.iter().rev() {
                if let Some(node) = tree.get_mut(&node_id) {
                    node.visits += 1;
                    node.value += reward / (depth as f32);
                }
            }
        }

        let best_path = self.extract_best_path(&tree);
        let total_value = best_path.iter().map(|n| n.value).sum::<f32>() / best_path.len() as f32;
        let verification_passed = best_path.iter().all(|n| n.step_verified);
        let action_history: Vec<String> = best_path.iter()
            .filter_map(|n| n.action.clone())
            .collect();

        info!("✅ MCTS complete: {} nodes explored, {} actions in best path", node_count, action_history.len());

        MCTSResult {
            best_path,
            total_value,
            nodes_explored: node_count as usize,
            max_depth: self.max_depth,
            verification_passed,
            action_history,
        }
    }

    async fn generate_actions(&self, abstraction: &str) -> anyhow::Result<Vec<String>> {
        let prompt = format!("{}\n\nProblem/context: {}\n\nGenerate actions:", self.system_prompt, abstraction);
        let response = self.llm.generate(&prompt, 200, 0.3).await?;

        let parsed: serde_json::Value = serde_json::from_str(&response).unwrap_or_else(|_| {
            json!({ "actions": [response] })
        });

        let actions: Vec<String> = parsed["actions"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        Ok(if actions.is_empty() {
            vec!["question".to_string(), "explore".to_string(), "conclude".to_string()]
        } else {
            actions
        })
    }

    async fn simulate_action(&self, state: &WorldState, action: &str) -> WorldState {
        let mut new_state = state.clone();
        new_state.predicted_future.push(action.to_string());
        new_state.uncertainty *= 0.9;
        new_state
    }

    async fn rollout_action(&self, _state: &WorldState) -> String {
        let possible = vec!["aprofundar".to_string(), "concluir".to_string(), "questionar".to_string()];
        let idx = rand::rng().random_range(0..possible.len());
        possible[idx].clone()
    }

    fn select_best_child(&self, tree: &HashMap<u64, MCTSNode>, parent_id: u64) -> u64 {
        let parent = tree.get(&parent_id).unwrap();
        let total_visits = parent.visits as f32;
        let mut best_child_id = parent_id;
        let mut best_ucb = -f32::INFINITY;
        for &child_id in &parent.children {
            let child = tree.get(&child_id).unwrap();
            let exploitation = child.value / (child.visits as f32 + 1e-6);
            let exploration = self.exploration_constant * (total_visits.ln() / (child.visits as f32 + 1e-6)).sqrt();
            let ucb = exploitation + exploration;
            if ucb > best_ucb {
                best_ucb = ucb;
                best_child_id = child_id;
            }
        }
        best_child_id
    }

    fn extract_best_path(&self, tree: &HashMap<u64, MCTSNode>) -> Vec<MCTSNode> {
        let mut path = Vec::new();
        let mut current_id = 0;
        while let Some(node) = tree.get(&current_id) {
            path.push(node.clone());
            if node.children.is_empty() { break; }
            let mut best_child_id = current_id;
            let mut best_value = -f32::INFINITY;
            for &child_id in &node.children {
                let child = tree.get(&child_id).unwrap();
                if child.value > best_value {
                    best_value = child.value;
                    best_child_id = child_id;
                }
            }
            if best_child_id == current_id { break; }
            current_id = best_child_id;
        }
        path
    }
}
