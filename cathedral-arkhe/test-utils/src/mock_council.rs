//! test-utils/src/mock_council.rs
//! Agentes do conselho: honestos, maliciosos, lentos e com falhas.
//! Selo: CATHEDRAL-ARKHE-TEST-MOCK-COUNCIL-v1.0.0

use anyhow::Result;
use async_trait::async_trait;
use arkhe_observer_5d::{RemoteAgentClient, MetaGovernanceRequest};
use rand::Rng;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

// ============================================================
// TIPOS DE AGENTES
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentBehavior {
    Honest,
    AlwaysOppose,
    AlwaysApprove,
    Random,
    MaliciousChaos,
    Slow { latency_ms: u64 },
    Flaky { failure_rate: f64 },
    ObserverSpammer,
    ReputationManipulator,
    SybilCollaborator { fellow_ids: Vec<String> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SybilPhase {
    BuildingTrust,
    CoordinatedAttack,
    Contamination,
}

#[derive(Debug, Clone)]
pub struct SybilState {
    pub phase: SybilPhase,
    pub contamination_attempts: usize,
    pub target_agents: Vec<String>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl SybilState {
    pub fn new() -> Self {
        Self {
            phase: SybilPhase::BuildingTrust,
            contamination_attempts: 0,
            target_agents: Vec::new(),
            start_time: Some(chrono::Utc::now()),
        }
    }

    pub fn update_phase(&mut self) {
        let elapsed = self.start_time
            .map(|t| (chrono::Utc::now() - t).num_seconds())
            .unwrap_or(0);

        if elapsed < 10 {
            self.phase = SybilPhase::BuildingTrust;
        } else if elapsed < 25 {
            self.phase = SybilPhase::CoordinatedAttack;
        } else {
            self.phase = SybilPhase::Contamination;
        }
    }
}

// ============================================================
// AGENTE MOCK CONFIGURÁVEL
// ============================================================

#[derive(Clone)]
pub struct ConfigurableMockAgent {
    pub agent_id: String,
    pub reputation: f64,
    pub behavior: AgentBehavior,
    pub approved_votes: Arc<tokio::sync::RwLock<u64>>,
    pub rejected_votes: Arc<tokio::sync::RwLock<u64>>,
    pub total_votes: Arc<tokio::sync::RwLock<u64>>,
    pub sybil_state: Option<Arc<tokio::sync::Mutex<SybilState>>>,
}

impl ConfigurableMockAgent {
    pub fn new(agent_id: &str, reputation: f64, behavior: AgentBehavior) -> Self {
        let sybil_state = match &behavior {
            AgentBehavior::SybilCollaborator { .. } => Some(Arc::new(tokio::sync::Mutex::new(SybilState::new()))),
            _ => None,
        };
        Self {
            agent_id: agent_id.to_string(),
            reputation,
            behavior,
            approved_votes: Arc::new(tokio::sync::RwLock::new(0)),
            rejected_votes: Arc::new(tokio::sync::RwLock::new(0)),
            total_votes: Arc::new(tokio::sync::RwLock::new(0)),
            sybil_state,
        }
    }

    pub async fn decide(&self, request: &MetaGovernanceRequest) -> bool {
        if let Some(state_arc) = &self.sybil_state {
            let mut state = state_arc.lock().await;
            state.update_phase();
            return self.decide_with_sybil_state(&state, request);
        }
        self.decide_default(request).await
    }

    fn decide_with_sybil_state(&self, state: &SybilState, request: &MetaGovernanceRequest) -> bool {
        let risk = request.risk_score;
        match state.phase {
            SybilPhase::BuildingTrust => {
                risk < 0.6 && self.reputation > 0.5
            }
            SybilPhase::CoordinatedAttack => {
                if risk < 0.3 { true }
                else if risk > 0.8 { true }
                else { false }
            }
            SybilPhase::Contamination => {
                risk > 0.4 && risk < 0.7 && rand::thread_rng().gen_bool(0.7)
            }
        }
    }

    async fn decide_default(&self, request: &MetaGovernanceRequest) -> bool {
        let risk = request.risk_score;
        let mut rng = rand::thread_rng();

        match self.behavior {
            AgentBehavior::Honest => {
                risk < 0.5 && self.reputation > 0.7
                    || risk < 0.3 && self.reputation > 0.5
                    || (risk < 0.7 && self.reputation > 0.9)
            }
            AgentBehavior::AlwaysOppose => false,
            AgentBehavior::AlwaysApprove => true,
            AgentBehavior::Random => rng.gen_bool(0.5),
            AgentBehavior::MaliciousChaos => {
                if risk > 0.8 { true }
                else if risk < 0.5 { false }
                else { rng.gen_bool(0.5) }
            }
            AgentBehavior::Slow { latency_ms } => {
                sleep(Duration::from_millis(latency_ms)).await;
                risk < 0.5
            }
            AgentBehavior::Flaky { failure_rate } => {
                if rng.gen_bool(failure_rate) {
                    return false;
                }
                risk < 0.5
            }
            AgentBehavior::ObserverSpammer => {
                if risk > 0.3 && risk < 0.6 { return true; }
                if risk > 0.8 { return true; }
                if risk < 0.2 { return false; }
                rng.gen_bool(0.5)
            }
            AgentBehavior::ReputationManipulator => {
                let proposer_rep = request
                    .metadata
                    .get("proposer_reputation")
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(0.5);
                if proposer_rep > 0.8 { false }
                else if proposer_rep < 0.3 { true }
                else { rng.gen_bool(0.5) }
            }
            AgentBehavior::SybilCollaborator { .. } => {
                false
            }
        }
    }

    pub async fn record_vote(&self, approved: bool) {
        let mut total = self.total_votes.write().await;
        *total += 1;
        if approved {
            let mut a = self.approved_votes.write().await;
            *a += 1;
        } else {
            let mut r = self.rejected_votes.write().await;
            *r += 1;
        }
    }

    pub async fn stats(&self) -> (u64, u64, u64) {
        let total = *self.total_votes.read().await;
        let approved = *self.approved_votes.read().await;
        let rejected = *self.rejected_votes.read().await;
        (total, approved, rejected)
    }
}

#[async_trait]
impl RemoteAgentClient for ConfigurableMockAgent {
    async fn query_reputation(&mut self) -> Result<f64> {
        Ok(self.reputation)
    }

    async fn request_vote(&mut self, request: &MetaGovernanceRequest) -> Result<bool> {
        let approve = self.decide(request).await;
        self.record_vote(approve).await;
        if matches!(self.behavior, AgentBehavior::Flaky { .. }) && rand::thread_rng().gen_bool(0.3) {
            return Err(anyhow::anyhow!("Agente flaky falhou propositalmente"));
        }
        Ok(approve)
    }
}

// ============================================================
// FÁBRICA DE AGENTES PARA TESTES
// ============================================================

pub struct CouncilTestFactory;

impl CouncilTestFactory {
    pub fn create_mixed_council(
        honest_count: usize,
        malicious_count: usize,
    ) -> Vec<ConfigurableMockAgent> {
        let mut agents = Vec::new();

        for i in 0..honest_count {
            let rep = 0.6 + rand::thread_rng().gen_range(0.0..0.35);
            agents.push(ConfigurableMockAgent::new(
                &format!("honest-{}", i),
                rep.clamp(0.5, 0.95),
                AgentBehavior::Honest,
            ));
        }

        for i in 0..malicious_count {
            let behavior = if i % 2 == 0 {
                AgentBehavior::AlwaysOppose
            } else {
                AgentBehavior::MaliciousChaos
            };
            agents.push(ConfigurableMockAgent::new(
                &format!("malicious-{}", i),
                0.9,
                behavior,
            ));
        }

        agents
    }

    pub fn create_slow_council(count: usize, latency_ms: u64) -> Vec<ConfigurableMockAgent> {
        (0..count)
            .map(|i| {
                ConfigurableMockAgent::new(
                    &format!("slow-{}", i),
                    0.7,
                    AgentBehavior::Slow { latency_ms },
                )
            })
            .collect()
    }

    pub fn create_flaky_council(count: usize, failure_rate: f64) -> Vec<ConfigurableMockAgent> {
        (0..count)
            .map(|i| {
                ConfigurableMockAgent::new(
                    &format!("flaky-{}", i),
                    0.7,
                    AgentBehavior::Flaky { failure_rate },
                )
            })
            .collect()
    }

    pub fn create_sybil_group(count: usize, group_id: &str) -> Vec<ConfigurableMockAgent> {
        let mut agents = Vec::new();
        let fellow_ids: Vec<String> = (0..count)
            .map(|i| format!("sybil-{}-{}", group_id, i))
            .collect();

        for i in 0..count {
            let agent_id = format!("sybil-{}-{}", group_id, i);
            agents.push(ConfigurableMockAgent::new(
                &agent_id,
                0.7,
                AgentBehavior::SybilCollaborator {
                    fellow_ids: fellow_ids.clone(),
                },
            ));
        }
        agents
    }

    pub fn create_reputation_manipulator_council(
        honest_count: usize,
        manipulator_count: usize,
    ) -> Vec<ConfigurableMockAgent> {
        let mut agents = Vec::new();

        for i in 0..honest_count {
            agents.push(ConfigurableMockAgent::new(
                &format!("honest-{}", i),
                0.7 + rand::thread_rng().gen_range(0.0..0.2),
                AgentBehavior::Honest,
            ));
        }

        for i in 0..manipulator_count {
            agents.push(ConfigurableMockAgent::new(
                &format!("manipulator-{}", i),
                0.9,
                AgentBehavior::ReputationManipulator,
            ));
        }

        agents
    }

    pub fn create_observer_spammer_council(
        honest_count: usize,
        spammer_count: usize,
    ) -> Vec<ConfigurableMockAgent> {
        let mut agents = Vec::new();

        for i in 0..honest_count {
            agents.push(ConfigurableMockAgent::new(
                &format!("honest-{}", i),
                0.8,
                AgentBehavior::Honest,
            ));
        }

        for i in 0..spammer_count {
            agents.push(ConfigurableMockAgent::new(
                &format!("spammer-{}", i),
                0.85,
                AgentBehavior::ObserverSpammer,
            ));
        }

        agents
    }
}
