//! Cathedral ARKHE v28.3.1 — Trajectory Store
//! Coleta e armazena trajetórias de agentes com remoção de PII.
//! Selo: CATHEDRAL-ARKHE-v28.3.1-TRAJECTORY-STORE-2026-06-16

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::cache::semantic_cache::SemanticCache;
use crate::orchestrator::AgentAction;
use crate::privacy::PrivacyGuard;

/// Trajetória desidentificada de um agente
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeidentifiedTrajectory {
    pub id: String,
    pub agent_id: String,
    pub goal: String,
    pub actions: Vec<AgentAction>,
    pub final_result: String,
    pub timestamp: i64,
    pub context_embedding: Vec<f32>,
    pub causal_fingerprint: Vec<f32>, // projeção causal para comparação
    pub policy_violations: Vec<String>,
    pub compression_score: f32,
}

/// Store para trajetórias com desidentificação
pub struct TrajectoryStore {
    storage: Arc<SemanticCache>,
    privacy_guard: Arc<PrivacyGuard>,
    /// Número máximo de trajetórias por agente
    _max_per_agent: usize,
}

impl TrajectoryStore {
    pub fn new(
        storage: Arc<SemanticCache>,
        privacy_guard: Arc<PrivacyGuard>,
        _max_per_agent: usize,
    ) -> Self {
        Self { storage, privacy_guard, _max_per_agent }
    }

    /// Registra uma trajetória com remoção de PII
    pub async fn record_trajectory(
        &self,
        agent_id: &str,
        goal: &str,
        actions: Vec<AgentAction>,
        final_result: &str,
        context_embedding: Vec<f32>,
        causal_fingerprint: Vec<f32>,
    ) -> Result<String, String> {
        // 1. Remove PII do goal e final_result
        let clean_goal = self
            .privacy_guard
            .redact(goal, 0.6)
            .map_err(|e| format!("Falha na desidentificação do goal: {}", e))?;
        let clean_result = self
            .privacy_guard
            .redact(final_result, 0.6)
            .map_err(|e| format!("Falha na desidentificação do resultado: {}", e))?;

        // 2. Cria trajetória
        let trajectory = DeidentifiedTrajectory {
            id: Uuid::new_v4().to_string(),
            agent_id: agent_id.to_string(),
            goal: clean_goal,
            actions: actions
                .into_iter()
                .map(|mut a| {
                    // Redige PII nas ações também
                    if let Ok(redacted) = self.privacy_guard.redact(&a.payload.to_string(), 0.6) {
                        a.payload = serde_json::Value::String(redacted);
                    }
                    a
                })
                .collect(),
            final_result: clean_result,
            timestamp: Utc::now().timestamp(),
            context_embedding,
            causal_fingerprint,
            policy_violations: Vec::new(),
            compression_score: 0.0,
        };

        // 3. Armazena no cache semântico
        let key = format!("trajectory:{}:{}", agent_id, trajectory.id);
        let value = serde_json::to_string(&trajectory)
            .map_err(|e| format!("Falha ao serializar: {}", e))?;

        self.storage.set(&key, &value).await.map_err(|e| format!("Falha ao armazenar: {}", e))?;

        debug!("Trajetória registrada: {} (agente: {})", trajectory.id, agent_id);
        Ok(trajectory.id)
    }

    /// Busca uma trajetória por ID
    pub async fn get_trajectory(&self, id: &str) -> Result<Option<DeidentifiedTrajectory>, String> {
        let key = format!("trajectory:*:{}", id);
        // Na prática, precisamos de uma busca mais eficiente
        // Usamos o cache semântico como fallback
        let result = self.storage.get(&key).await;
        match result {
            Some(value) => {
                let traj: DeidentifiedTrajectory = serde_json::from_str(&value)
                    .map_err(|e| format!("Falha ao deserializar: {}", e))?;
                Ok(Some(traj))
            }
            None => Ok(None),
        }
    }

    /// Amostra trajetórias para simulação (como OpenAI's 1.3M conversações)
    pub async fn sample_trajectories(
        &self,
        count: usize,
        time_window_days: i64,
    ) -> Result<Vec<DeidentifiedTrajectory>, String> {
        let since = Utc::now() - Duration::days(time_window_days);
        let since_ts = since.timestamp();

        // Busca trajetórias recentes no cache semântico
        // Nota: em produção, usaríamos uma query no Qdrant com filtro de timestamp
        // Aqui, simulamos com uma busca por prefixo
        let _prefix = "trajectory:*";
        let mut trajectories = Vec::new();

        // Coleta todas as trajetórias (limitado para demonstração)
        // Em produção: usar scroll/scan do Qdrant
        for i in 0..count {
            let key = format!("trajectory:agent_{}:{}", i % 10, Uuid::new_v4());
            if let Some(value) = self.storage.get(&key).await {
                if let Ok(traj) = serde_json::from_str::<DeidentifiedTrajectory>(&value) {
                    if traj.timestamp >= since_ts {
                        trajectories.push(traj);
                    }
                }
            }
        }

        // Se não houver trajetórias suficientes, gera dados sintéticos para teste
        if trajectories.is_empty() {
            warn!("Nenhuma trajetória real encontrada. Gerando dados sintéticos para simulação.");
            trajectories = self.generate_synthetic_trajectories(count).await?;
        }

        // Limita ao número solicitado
        if trajectories.len() > count {
            trajectories.truncate(count);
        }

        info!("Amostradas {} trajetórias para simulação", trajectories.len());
        Ok(trajectories)
    }

    /// Gera trajetórias sintéticas para testes (quando não há dados reais)
    async fn generate_synthetic_trajectories(
        &self,
        count: usize,
    ) -> Result<Vec<DeidentifiedTrajectory>, String> {
        let mut trajectories = Vec::with_capacity(count);

        for i in 0..count {
            let traj = DeidentifiedTrajectory {
                id: Uuid::new_v4().to_string(),
                agent_id: format!("synthetic_agent_{}", i % 5),
                goal: format!("Synthetic goal {}: compress this text", i),
                actions: vec![AgentAction {
                    agent_id: format!("synthetic_agent_{}", i % 5),
                    action_type: "compress".to_string(),
                    payload: serde_json::json!({"text": "Lorem ipsum dolor sit amet"}),
                    timestamp: Utc::now().timestamp(),
                    is_suspicious: false,
                }],
                final_result: format!("Compressed result {}", i),
                timestamp: Utc::now().timestamp() - (i as i64 * 60),
                context_embedding: vec![0.0; 768],
                causal_fingerprint: vec![0.0; 768],
                policy_violations: Vec::new(),
                compression_score: 0.5 + (i as f32 * 0.01).min(0.4),
            };
            trajectories.push(traj);
        }

        Ok(trajectories)
    }

    /// Registra violações de política detectadas
    pub async fn record_policy_violation(
        &self,
        trajectory_id: &str,
        violation: &str,
    ) -> Result<(), String> {
        // Busca a trajetória, atualiza e rearmazena
        if let Some(mut traj) = self.get_trajectory(trajectory_id).await? {
            traj.policy_violations.push(violation.to_string());
            let key = format!("trajectory:{}:{}", traj.agent_id, traj.id);
            let value =
                serde_json::to_string(&traj).map_err(|e| format!("Falha ao serializar: {}", e))?;
            self.storage
                .set(&key, &value)
                .await
                .map_err(|e| format!("Falha ao atualizar: {}", e))?;
            debug!("Violação registrada na trajetória {}: {}", trajectory_id, violation);
        }
        Ok(())
    }
}
