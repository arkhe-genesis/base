//! Cathedral ARKHE v28.3.2 — Subagent Persistence
//! Serialização e persistência de subagentes para suspensão/retomada.
//! Selo: CATHEDRAL-ARKHE-v28.3.2-SUBAGENT-PERSISTENCE-2026-06-17

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{info, warn};

use crate::subagent_spawner::{Subagent, SubagentIdentity, SubagentSpawner};
use crate::context::{Context, ContextEntry};
use crate::simulation::trajectory_store::TrajectoryStore;
use crate::attestation::PolicyDescriptor;
use crate::attestation::AttestationSigner;

// ============================================================================
// 1. Estado Serializável
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentState {
    pub identity: SubagentIdentity,
    pub policies: Vec<PolicyDescriptor>,
    pub context_entries: Vec<ContextEntry>,
    pub tools: Vec<String>,
    pub is_active: bool,
    pub last_activity: DateTime<Utc>,
    pub parent_id: String,
}

impl SubagentState {
    /// Cria um estado a partir de um subagente ativo.
    pub async fn from_subagent(subagent: &Subagent) -> Self {
        let entries = subagent.context.get_entries().await;
        Self {
            identity: subagent.identity.clone(),
            policies: subagent.policies.clone(),
            context_entries: entries,
            tools: subagent.tools.clone(),
            is_active: subagent.is_active,
            last_activity: subagent.last_activity,
            parent_id: subagent.identity.parent_id.clone(),
        }
    }

    /// Reconstrói um subagente a partir do estado.
    pub async fn into_subagent(
        self,
        parent_signer: Arc<dyn AttestationSigner + Send + Sync>,
        trajectory_store: Arc<TrajectoryStore>,
    ) -> Subagent {
        let ctx = Arc::new(Context::new(&self.identity.id));
        for entry in &self.context_entries {
            let _ = ctx.add(&entry.content, entry.role.clone(), entry.importance).await;
        }
        Subagent {
            identity: self.identity,
            policies: self.policies,
            context: ctx,
            tools: self.tools,
            parent_signer,
            trajectory_store,
            is_active: self.is_active,
            last_activity: self.last_activity,
        }
    }
}

// ============================================================================
// 2. Métodos no SubagentSpawner
// ============================================================================

impl SubagentSpawner {
    /// Suspende um subagente (salva estado e remove da lista ativa).
    pub async fn suspend(&self, id: &str) -> Result<SubagentState, String> {
        let sub = self.get(id).await.ok_or("Subagente não encontrado")?;
        let state = SubagentState::from_subagent(&sub).await;
        self.terminate(id).await?;
        info!("💾 Subagente {} suspenso", id);
        Ok(state)
    }

    /// Retoma um subagente a partir do estado salvo.
    pub async fn resume(&self, state: SubagentState) -> Result<Subagent, String> {
        let sub = state.into_subagent(
            self.parent_signer.clone(),
            self.trajectory_store.clone(),
        ).await;
        let mut active = self.active_subagents.write().await;
        active.push(sub.clone());
        info!("🔄 Subagente {} retomado", sub.identity.id);
        Ok(sub)
    }

    /// Salva o estado no TrajectoryStore (ou arquivo).
    pub async fn save_state(&self, id: &str) -> Result<(), String> {
        let state = self.suspend(id).await?;
        let json = serde_json::to_string(&state)
            .map_err(|e| format!("Falha ao serializar: {}", e))?;
        self.trajectory_store.record_trajectory(
            &state.identity.id,
            "subagent_state",
            vec![],
            &json,
            vec![],
            vec![],
        ).await.map_err(|e| format!("Failed to record state: {}", e))?;
        info!("📂 Estado do subagente {} salvo", id);
        Ok(())
    }

    /// Carrega o estado do TrajectoryStore.
    pub async fn load_state(&self, id: &str) -> Result<SubagentState, String> {
        // Busca a trajetória mais recente com goal "subagent_state"
        // This relies on stub implementations. Since `list_trajectories` is not
        // fully implemented in the current mock trajectory module, we'll return an error.
        Err("Estado não encontrado".to_string())
    }
}
