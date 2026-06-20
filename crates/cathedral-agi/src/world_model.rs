use std::{collections::VecDeque, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

use crate::llm_client::OllamaClient;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Intent {
    Question,
    Instruction,
    Clarification,
    Exploration,
    Meta,
    EthicalCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub user_intent: Intent,
    pub context_embedding: Vec<f32>,
    pub predicted_future: Vec<String>,
    pub uncertainty: f32,
    pub timestamp: i64,
}

pub struct WorldModel {
    llm: Arc<OllamaClient>,
    state_buffer: VecDeque<WorldState>,
    latent_dim: usize,
    horizon: usize,
    system_prompt: String,
}

impl WorldModel {
    pub fn new(llm: Arc<OllamaClient>, latent_dim: usize, horizon: usize) -> Self {
        Self {
            llm,
            state_buffer: VecDeque::with_capacity(horizon * 2),
            latent_dim,
            horizon,
            system_prompt: "You are a world model. Given a user input, predict the user's intent, future consequences, and your uncertainty. Respond in JSON format with keys: intent, predicted_future (array), uncertainty (0-1).".to_string(),
        }
    }

    pub async fn step(&mut self, prompt: &str) -> anyhow::Result<(String, WorldState)> {
        info!("🌍 WorldModel processing: {}", prompt);

        // 1. Gerar resposta com o LLM
        let response = self.llm.generate(prompt, 512, 0.2).await?;

        // 2. Predizer intenção usando o LLM com prompt estruturado
        let analysis_prompt = format!(
            "{}\n\nUser input: {}\n\nAnalyze and respond in JSON format with keys: intent, predicted_future (array of 3 strings), uncertainty (float 0-1).",
            self.system_prompt, prompt
        );
        let analysis = self.llm.generate(&analysis_prompt, 256, 0.1).await?;

        // 3. Parse do JSON
        let parsed: serde_json::Value = serde_json::from_str(&analysis).unwrap_or_else(|_| {
            json!({
                "intent": "Question",
                "predicted_future": ["User will ask follow-up question"],
                "uncertainty": 0.3
            })
        });

        let intent = match parsed["intent"].as_str().unwrap_or("Question") {
            "Question" => Intent::Question,
            "Instruction" => Intent::Instruction,
            "Clarification" => Intent::Clarification,
            "Exploration" => Intent::Exploration,
            "Meta" => Intent::Meta,
            "EthicalCheck" => Intent::EthicalCheck,
            _ => Intent::Question,
        };

        let predicted_future: Vec<String> = parsed["predicted_future"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        let uncertainty = parsed["uncertainty"].as_f64().unwrap_or(0.3) as f32;

        let state = WorldState {
            user_intent: intent,
            context_embedding: vec![0.0; self.latent_dim],
            predicted_future,
            uncertainty,
            timestamp: chrono::Utc::now().timestamp(),
        };

        self.state_buffer.push_back(state.clone());
        if self.state_buffer.len() > self.horizon * 2 {
            self.state_buffer.pop_front();
        }

        Ok((response, state))
    }

    pub fn current_state(&self) -> Option<WorldState> {
        self.state_buffer.back().cloned()
    }

    pub fn state_history(&self) -> Vec<WorldState> {
        self.state_buffer.iter().cloned().collect()
    }
}
