//! Cathedral ARKHE v28.3.1 — Tool Simulator
//! Simula chamadas de ferramentas com consistência causal via CIP.
//! Selo: CATHEDRAL-ARKHE-v28.3.1-TOOL-SIMULATOR-2026-06-16

use std::sync::Arc;

use ndarray::Array1;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{geometry::service::CausalGeometryService, llm::client::LlmClient};

/// Resposta de uma ferramenta simulada
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    pub tool_name: String,
    pub response: String,
    pub causal_fidelity: f32, // quão consistente causalmente
}

/// Histórico de chamadas de ferramentas
#[derive(Debug, Clone, Default)]
pub struct ToolCallHistory {
    pub calls: Vec<(String, serde_json::Value, ToolResponse)>,
}

impl ToolCallHistory {
    pub fn add(&mut self, tool: &str, params: &serde_json::Value, response: &ToolResponse) {
        self.calls.push((tool.to_string(), params.clone(), response.clone()));
    }

    pub fn last_n(&self, n: usize) -> Vec<(String, serde_json::Value, ToolResponse)> {
        let start = self.calls.len().saturating_sub(n);
        self.calls[start..].to_vec()
    }
}

/// Simulador de ferramentas com consistência causal
pub struct ToolSimulator {
    geometry: Arc<CausalGeometryService>,
    llm_client: Arc<dyn LlmClient>,
    /// Limite de similaridade causal para regeneração
    similarity_threshold: f32,
}

impl ToolSimulator {
    pub fn new(
        geometry: Arc<CausalGeometryService>,
        llm_client: Arc<dyn LlmClient>,
        similarity_threshold: f32,
    ) -> Self {
        Self { geometry, llm_client, similarity_threshold }
    }

    /// Simula uma chamada de ferramenta mantendo consistência causal
    pub async fn simulate_tool_call(
        &self,
        tool_name: &str,
        parameters: &serde_json::Value,
        context_embedding: &Array1<f32>,
        history: &ToolCallHistory,
    ) -> Result<ToolResponse, String> {
        debug!("Simulando chamada de ferramenta: {}", tool_name);

        // 1. Projeta o contexto no espaço causal
        let causal_context = self.geometry.project_causal(context_embedding);

        // 2. Constrói prompt com histórico
        let history_str = self.format_history(history);
        let ctx_slice = causal_context.as_slice().unwrap_or(&[]);
        let ctx_slice_trunc = if ctx_slice.len() > 10 { &ctx_slice[0..10] } else { ctx_slice };

        let prompt = format!(
            r#"Simulate the response for tool '{}' with parameters: {}.

Context embedding (causal projection): {:?}

Previous tool calls:
{}

Generate a realistic and causally-consistent response. Consider:
- The tool's purpose and expected output format
- The causal relationships between previous calls and this one
- The semantic constraints of the context

Tool response:"#,
            tool_name,
            serde_json::to_string_pretty(parameters).unwrap_or_default(),
            ctx_slice_trunc,
            history_str
        );

        // 3. Gera resposta via LLM
        let response_text = self.llm_client.generate(&prompt).await?;

        // 4. Valida consistência causal
        let response_emb = self.geometry.embed(&response_text);
        let similarity =
            self.geometry.causal_similarity(&context_embedding.view(), &response_emb.view());

        // Se a similaridade for muito alta, pode ser um reflexo do contexto (não realista)
        // Se for muito baixa, pode ser inconsistente
        let fidelity = if similarity > self.similarity_threshold {
            // Similaridade moderada indica consistência causal
            1.0 - (similarity - 0.5).abs() * 2.0
        } else {
            // Muito diferente: provavelmente inconsistente
            0.3
        };

        debug!(
            "Tool {} simulation: fidelity={:.3}, similarity={:.3}",
            tool_name, fidelity, similarity
        );

        Ok(ToolResponse {
            tool_name: tool_name.to_string(),
            response: response_text,
            causal_fidelity: fidelity.clamp(0.0, 1.0),
        })
    }

    /// Simula múltiplas chamadas de ferramenta em sequência
    pub async fn simulate_tool_calls(
        &self,
        tool_calls: &[(String, serde_json::Value)],
        context_embedding: &Array1<f32>,
    ) -> Result<Vec<ToolResponse>, String> {
        let mut history = ToolCallHistory::default();
        let mut responses = Vec::new();

        for (tool, params) in tool_calls {
            let response =
                self.simulate_tool_call(tool, params, context_embedding, &history).await?;

            history.add(tool, params, &response);
            responses.push(response);
        }

        Ok(responses)
    }

    /// Formata o histórico para o prompt
    fn format_history(&self, history: &ToolCallHistory) -> String {
        if history.calls.is_empty() {
            return "None".to_string();
        }

        let mut lines = Vec::new();
        for (tool, params, response) in history.last_n(5) {
            lines.push(format!(
                "Tool: {} | Params: {} | Response: {}",
                tool,
                serde_json::to_string(&params).unwrap_or_default(),
                response.response.chars().take(100).collect::<String>()
            ));
        }
        lines.join("\n")
    }
}
