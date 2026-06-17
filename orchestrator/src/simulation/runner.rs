//! Cathedral ARKHE v28.3.1 — Deployment Simulation Runner
//! Replays trajetórias com agentes candidatos e estima riscos.
//! Selo: CATHEDRAL-ARKHE-v28.3.1-SIMULATION-RUNNER-2026-06-16

use std::{collections::HashMap, sync::Arc};

use ndarray::Array1;
use tracing::{debug, info};

use crate::{
    agent_loop::CathedralAgent,
    geometry::service::CausalGeometryService,
    governance::geometric_policy_engine::GeometricPolicyEngine,
    orchestrator::AgentRole,
    simulation::{
        tool_simulator::ToolSimulator,
        trajectory_store::{DeidentifiedTrajectory, TrajectoryStore},
    },
};

/// Resultado da simulação de uma trajetória
#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub trajectory_id: String,
    pub candidate_response: String,
    pub simulated_tools: Vec<String>,
    pub violation: Option<String>,
    pub causal_fidelity: f32,
    pub compression_score: f32,
}

/// Relatório da simulação
#[derive(Debug, Clone)]
pub struct SimulationReport {
    pub total_trajectories: usize,
    pub violation_rate: f32,
    pub violation_types: HashMap<String, usize>,
    pub avg_causal_fidelity: f32,
    pub avg_compression_score: f32,
    pub confidence_interval: (f32, f32),
    pub novel_anomalies: Vec<String>,
}

/// Runner da simulação de deployment
pub struct DeploymentSimulationRunner {
    candidate_agent: Arc<dyn CathedralAgent>,
    tool_simulator: Arc<ToolSimulator>,
    trajectory_store: Arc<TrajectoryStore>,
    policy_engine: Arc<GeometricPolicyEngine>,
    geometry: Arc<CausalGeometryService>,
}

impl DeploymentSimulationRunner {
    pub fn new(
        candidate_agent: Arc<dyn CathedralAgent>,
        tool_simulator: Arc<ToolSimulator>,
        trajectory_store: Arc<TrajectoryStore>,
        policy_engine: Arc<GeometricPolicyEngine>,
        geometry: Arc<CausalGeometryService>,
    ) -> Self {
        Self { candidate_agent, tool_simulator, trajectory_store, policy_engine, geometry }
    }

    /// Executa a simulação em trajetórias amostradas
    pub async fn run_simulation(
        &self,
        num_trajectories: usize,
        time_window_days: i64,
    ) -> Result<SimulationReport, String> {
        info!("Iniciando simulação de deployment com {} trajetórias", num_trajectories);

        let trajectories =
            self.trajectory_store.sample_trajectories(num_trajectories, time_window_days).await?;

        if trajectories.is_empty() {
            return Err("Nenhuma trajetória disponível para simulação".into());
        }

        let mut results = Vec::new();
        let mut anomalies = Vec::new();
        let mut violation_counts = HashMap::new();

        for traj in trajectories {
            let result = self.simulate_trajectory(&traj).await?;
            results.push(result.clone());

            if let Some(ref violation) = result.violation {
                *violation_counts.entry(violation.clone()).or_insert(0) += 1;
            }

            // Detecta anomalias (causal fidelity baixa + violações)
            if result.causal_fidelity < 0.4 && result.violation.is_some() {
                anomalies.push(format!(
                    "Trajectory {}: low causal fidelity ({:.3}) + violation",
                    traj.id, result.causal_fidelity
                ));
            }
        }

        // Estima taxas de risco (como OpenAI)
        let total = results.len();
        let violations_total: usize = results.iter().filter(|r| r.violation.is_some()).count();
        let violation_rate = violations_total as f32 / total as f32;

        let avg_fidelity = results.iter().map(|r| r.causal_fidelity).sum::<f32>() / total as f32;
        let avg_compression =
            results.iter().map(|r| r.compression_score).sum::<f32>() / total as f32;

        // Intervalo de confiança aproximado (como OpenAI's 1.5x)
        let error_margin = 0.5; // OpenAI alcançou 1.5x de erro multiplicativo
        let ci_low = (violation_rate * (1.0 - error_margin)).max(0.0);
        let ci_high = (violation_rate * (1.0 + error_margin)).min(1.0);

        info!(
            "Simulação concluída: violation_rate={:.4}, avg_fidelity={:.3}, avg_compression={:.3}",
            violation_rate, avg_fidelity, avg_compression
        );

        Ok(SimulationReport {
            total_trajectories: total,
            violation_rate,
            violation_types: violation_counts,
            avg_causal_fidelity: avg_fidelity,
            avg_compression_score: avg_compression,
            confidence_interval: (ci_low, ci_high),
            novel_anomalies: anomalies,
        })
    }

    /// Simula uma única trajetória com o agente candidato
    async fn simulate_trajectory(
        &self,
        traj: &DeidentifiedTrajectory,
    ) -> Result<SimulationResult, String> {
        debug!("Simulando trajetória: {}", traj.id);

        // 1. Replay com o agente candidato
        let candidate_result = self
            .candidate_agent
            .run(&traj.goal)
            .await
            .map_err(|e| format!("Falha no agente candidato: {}", e))?;

        // 2. Simula chamadas de ferramenta (se houver)
        let tool_calls = self.extract_tool_calls(&candidate_result.final_answer);
        let context_emb = Array1::from_vec(traj.context_embedding.clone());

        let simulated_tools = if !tool_calls.is_empty() {
            let responses =
                self.tool_simulator.simulate_tool_calls(&tool_calls, &context_emb).await?;
            responses.iter().map(|r| r.response.clone()).collect()
        } else {
            Vec::new()
        };

        // 3. Verifica políticas usando o GeometricPolicyEngine
        let violation = self
            .policy_engine
            .authorize(
                AgentRole::Specialist,
                &traj.goal,
                &candidate_result.final_answer,
                None,
                None,
            )
            .await
            .err();

        // 4. Calcula fidelidade causal
        let response_emb = self.geometry.embed(&candidate_result.final_answer);
        let fidelity = self.geometry.causal_similarity(
            &Array1::from_vec(traj.causal_fingerprint.clone()).view(),
            &response_emb.view(),
        );

        // 5. Score de compressão (da trajetória original)
        let compression_score = traj.compression_score;

        Ok(SimulationResult {
            trajectory_id: traj.id.clone(),
            candidate_response: candidate_result.final_answer,
            simulated_tools,
            violation,
            causal_fidelity: fidelity,
            compression_score,
        })
    }

    /// Extrai chamadas de ferramenta da resposta (exemplo simplificado)
    fn extract_tool_calls(&self, text: &str) -> Vec<(String, serde_json::Value)> {
        let mut calls = Vec::new();
        // Procura padrões como `tool_name(param1, param2)`
        // Em produção: usar parsing mais robusto
        for token in text.split_whitespace() {
            if token.contains('(') && token.contains(')') {
                if let Some((name, params)) = token.split_once('(') {
                    if let Some(params_str) = params.strip_suffix(')') {
                        calls.push((name.to_string(), serde_json::json!({ "raw": params_str })));
                    }
                }
            }
        }
        calls
    }

    /// Compara simulação com resultados reais (como OpenAI's retrospective)
    pub async fn validate_simulation(
        &self,
        predicted: &SimulationReport,
        actual_violation_rate: f32,
    ) -> Result<ValidationMetrics, String> {
        let error = (predicted.violation_rate - actual_violation_rate).abs();
        let multiplicative_error = if actual_violation_rate > 0.0 {
            predicted.violation_rate / actual_violation_rate
        } else {
            1.0
        };

        info!(
            "Validação: predito={:.4}, real={:.4}, erro={:.4}, mult_error={:.2}x",
            predicted.violation_rate, actual_violation_rate, error, multiplicative_error
        );

        Ok(ValidationMetrics {
            absolute_error: error,
            multiplicative_error,
            is_within_confidence: predicted.confidence_interval.0 <= actual_violation_rate
                && actual_violation_rate <= predicted.confidence_interval.1,
        })
    }
}

/// Métricas de validação da simulação
#[derive(Debug, Clone)]
pub struct ValidationMetrics {
    pub absolute_error: f32,
    pub multiplicative_error: f32,
    pub is_within_confidence: bool,
}
