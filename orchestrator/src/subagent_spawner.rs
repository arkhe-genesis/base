//! Cathedral ARKHE v28.3.2 — Subagent Spawner (Versão Completa)
//! Cria dinamicamente subagentes com herança de identidade, políticas e memória.
//! Selo: CATHEDRAL-ARKHE-v28.3.2-SUBAGENT-SPAWNER-2026-06-17

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};
use chrono::{DateTime, Utc};

use crate::attestation::{
    AttestationManager, AttestationProvider, AttestationSigner,
    ExecutionAttestation, IdentityAttestation, PolicyDescriptor,
};
use crate::governance::geometric_policy_engine::GeometricPolicyEngine;
use crate::simulation::trajectory_store::TrajectoryStore;
use crate::context::Context;
use crate::identity_derivation::IdentityDerivationService;

// ============================================================================
// 1. Identidade do Subagente (Derivada do Pai)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentIdentity {
    pub id: String,
    pub parent_id: String,
    pub derivation_path: String,
    pub public_key: String,
    pub created_at: DateTime<Utc>,
    pub purpose: String,
    pub version: u32,
}

impl SubagentIdentity {
    /// Deriva uma nova identidade usando um caminho de derivação hierárquico.
    /// Em produção: usar BIP32/Ed25519 para derivação de chaves.
    pub fn derive(parent: &IdentityAttestation, purpose: &str, index: u32) -> Self {
        let derivation_path = format!("{}/{}/{}", parent.id, purpose, index);
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            parent_id: parent.id.clone(),
            derivation_path,
            public_key: format!("derived:{}:{}", parent.signer_key_id, index),
            created_at: Utc::now(),
            purpose: purpose.to_string(),
            version: 1,
        }
    }

    /// Verifica se esta identidade é filha legítima de um pai.
    pub fn is_child_of(&self, parent_id: &str) -> bool {
        self.parent_id == parent_id
    }
}

// ============================================================================
// 2. Subagente — Entidade Ativa com Contexto Próprio
// ============================================================================

#[derive(Clone)]
pub struct Subagent {
    pub identity: SubagentIdentity,
    pub policies: Vec<PolicyDescriptor>,
    pub context: Arc<Context>,
    pub tools: Vec<String>,
    pub parent_signer: Arc<dyn AttestationSigner + Send + Sync>,
    pub trajectory_store: Arc<TrajectoryStore>,
    pub is_active: bool,
    pub last_activity: DateTime<Utc>,
}

impl Subagent {
    /// Cria um novo subagente (chamado pelo Spawner).
    pub fn new(
        identity: SubagentIdentity,
        policies: Vec<PolicyDescriptor>,
        tools: Vec<String>,
        parent_signer: Arc<dyn AttestationSigner + Send + Sync>,
        trajectory_store: Arc<TrajectoryStore>,
    ) -> Self {
        Self {
            identity: identity.clone(),
            policies,
            context: Arc::new(Context::new(&identity.id)),
            tools,
            parent_signer,
            trajectory_store,
            is_active: true,
            last_activity: Utc::now(),
        }
    }

    /// Executa uma tarefa e gera ExecutionAttestation (assinado pelo pai).
    pub async fn execute(
        &self,
        task: &str,
        provider: &dyn AttestationProvider,
        cost_cap: Option<f64>,
    ) -> Result<ExecutionAttestation, String> {
        if !self.is_active {
            return Err("Subagente inativo".to_string());
        }

        info!("🔐 Subagente {} executando: {}", self.identity.id, task);

        // 1. Verifica políticas (simulação)
        for policy in &self.policies {
            if policy.blocking {
                // Em produção: usar policy_engine.authorize()
                // Por enquanto, apenas log
                info!("   Política: {} (blocking={})", policy.name, policy.blocking);
            }
        }

        // 2. Executa o workload
        let result = provider.run(task, cost_cap).await?;

        // 3. Cria atestado (assinado pelo pai)
        let mut attestation = ExecutionAttestation::new(
            task,
            &result,
            &self.identity.id,
            cost_cap.unwrap_or(0.01),
            vec!["subagent".to_string()],
            0.95,
            &self.identity.public_key,
        );

        // Assinatura do pai (delegação)
        attestation.sign(self.parent_signer.as_ref())?;

        // 4. Registra no TrajectoryStore
        let _ = self.trajectory_store.record_trajectory(
            &self.identity.id,
            &format!("execute: {}", task),
            vec![],
            &result,
            vec![],
            vec![],
        ).await;

        Ok(attestation)
    }

    /// Registra um evento no TrajectoryStore do subagente.
    pub async fn record_event(&self, event_type: &str, details: &str) -> Result<(), String> {
        let _ = self.trajectory_store.record_trajectory(
            &self.identity.id,
            event_type,
            vec![],
            details,
            vec![],
            vec![],
        ).await;
        Ok(())
    }

    /// Desativa o subagente (encerra operações).
    pub async fn deactivate(&mut self) {
        self.is_active = false;
        let _ = self.record_event("deactivated", "Subagente desativado").await;
        info!("🗑️ Subagente {} desativado", self.identity.id);
    }
}

// ============================================================================
// 4. SubagentSpawner — Fábrica Soberana
// ============================================================================

pub struct SubagentSpawner {
    pub parent_identity: Arc<RwLock<IdentityAttestation>>,
    pub parent_signer: Arc<dyn AttestationSigner + Send + Sync>,
    pub policy_engine: Arc<GeometricPolicyEngine>,
    pub attestation_manager: Arc<AttestationManager>,
    pub trajectory_store: Arc<TrajectoryStore>,
    pub derivation_service: Arc<IdentityDerivationService>,
    pub active_subagents: Arc<RwLock<Vec<Subagent>>>,
    pub max_subagents: usize,
    pub next_index: Arc<RwLock<u32>>,
}

impl SubagentSpawner {
    pub fn new_with_derivation(
        parent_identity: Arc<RwLock<IdentityAttestation>>,
        parent_signer: Arc<dyn AttestationSigner + Send + Sync>,
        policy_engine: Arc<GeometricPolicyEngine>,
        attestation_manager: Arc<AttestationManager>,
        trajectory_store: Arc<TrajectoryStore>,
        max_subagents: usize,
        seed: [u8; 32],
    ) -> Self {
        let parent_id = parent_identity.blocking_read().id.clone();
        let derivation_service = IdentityDerivationService::from_seed(
            &parent_id,
            &seed,
        );
        Self {
            parent_identity,
            parent_signer,
            policy_engine,
            attestation_manager,
            trajectory_store,
            derivation_service: Arc::new(derivation_service),
            active_subagents: Arc::new(RwLock::new(Vec::new())),
            max_subagents,
            next_index: Arc::new(RwLock::new(0)),
        }
    }

    pub fn new(
        parent_identity: Arc<RwLock<IdentityAttestation>>,
        parent_signer: Arc<dyn AttestationSigner + Send + Sync>,
        policy_engine: Arc<GeometricPolicyEngine>,
        attestation_manager: Arc<AttestationManager>,
        trajectory_store: Arc<TrajectoryStore>,
        max_subagents: usize,
    ) -> Self {
        Self::new_with_derivation(
            parent_identity,
            parent_signer,
            policy_engine,
            attestation_manager,
            trajectory_store,
            max_subagents,
            [0u8; 32]
        )
    }

    /// Cria um novo subagente com propósito específico.
    pub async fn spawn(
        &self,
        purpose: &str,
        tools: Vec<String>,
    ) -> Result<Subagent, String> {
        // 1. Verifica limite
        {
            let active = self.active_subagents.read().await;
            if active.len() >= self.max_subagents {
                return Err(format!("Limite de subagentes excedido: {}", self.max_subagents));
            }
        }

        // 2. Obtém identidade do pai
        let parent = self.parent_identity.read().await;
        let index = {
            let mut idx = self.next_index.write().await;
            let current = *idx;
            *idx += 1;
            current
        };

        // 3. Deriva identidade do subagente
        let sub_identity_attestation = self.derivation_service.derive_attestation(purpose, index);

        let sub_identity = SubagentIdentity {
            id: sub_identity_attestation.id.clone(),
            parent_id: parent.id.clone(),
            derivation_path: format!("{}/{}", purpose, index),
            public_key: sub_identity_attestation.hardware_fingerprint.unwrap_or_default(),
            created_at: Utc::now(),
            purpose: purpose.to_string(),
            version: 1,
        };

        // 4. Herda políticas do pai
        let parent_policies = self.policy_engine.list_active_policies().await?;
        let sub_policies = parent_policies;

        // 5. Cria o subagente
        let subagent = Subagent::new(
            sub_identity,
            sub_policies,
            tools,
            self.parent_signer.clone(),
            self.trajectory_store.clone(),
        );

        // 6. Registra a criação
        subagent.record_event("created", &format!("Propósito: {}", purpose)).await?;

        // 7. Adiciona à lista de ativos
        {
            let mut active = self.active_subagents.write().await;
            active.push(subagent.clone());
        }

        info!("✅ Subagente criado: {} (propósito: {})", subagent.identity.id, purpose);
        Ok(subagent)
    }

    /// Lista todos os subagentes ativos.
    pub async fn list_active(&self) -> Vec<Subagent> {
        self.active_subagents.read().await.clone()
    }

    /// Encontra um subagente por ID.
    pub async fn get(&self, id: &str) -> Option<Subagent> {
        let active = self.active_subagents.read().await;
        active.iter().find(|s| s.identity.id == id).cloned()
    }

    /// Encontra subagentes por propósito.
    pub async fn get_by_purpose(&self, purpose: &str) -> Vec<Subagent> {
        let active = self.active_subagents.read().await;
        active.iter()
            .filter(|s| s.identity.purpose == purpose)
            .cloned()
            .collect()
    }

    /// Termina um subagente (remove e arquiva).
    pub async fn terminate(&self, id: &str) -> Result<Subagent, String> {
        let mut active = self.active_subagents.write().await;
        let pos = active.iter().position(|s| s.identity.id == id);
        if let Some(idx) = pos {
            let mut subagent = active.remove(idx);
            subagent.deactivate().await;
            subagent.record_event("terminated", "Subagente terminado pelo spawner").await?;
            info!("🗑️ Subagente {} terminado", id);
            Ok(subagent)
        } else {
            Err("Subagente não encontrado".to_string())
        }
    }

    /// Obtém estatísticas dos subagentes.
    pub async fn stats(&self) -> SubagentStats {
        let active = self.active_subagents.read().await;
        let total = active.len();
        let active_count = active.iter().filter(|s| s.is_active).count();
        let purposes: Vec<_> = active.iter()
            .map(|s| s.identity.purpose.clone())
            .collect();

        SubagentStats {
            total,
            active: active_count,
            purposes,
        }
    }

    /// Encerra todos os subagentes (shutdown).
    pub async fn terminate_all(&self) -> Result<usize, String> {
        let mut active = self.active_subagents.write().await;
        let count = active.len();
        for sub in active.iter_mut() {
            sub.deactivate().await;
        }
        active.clear();
        info!("🗑️ Todos os {} subagentes terminados", count);
        Ok(count)
    }
}

// ============================================================================
// 5. Estatísticas
// ============================================================================

#[derive(Debug, Clone)]
pub struct SubagentStats {
    pub total: usize,
    pub active: usize,
    pub purposes: Vec<String>,
}
