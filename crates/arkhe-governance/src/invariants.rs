use chrono::{DateTime, Utc, Duration};
use std::collections::HashSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionClass {
    Operational,
    Standard,
    Critical,
}

impl ActionClass {
    pub fn name(&self) -> &'static str {
        match self {
            ActionClass::Operational => "operational",
            ActionClass::Standard => "standard",
            ActionClass::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdministrativeAction {
    SafeCoreUpdate,
    CapsulePrivilegeChange,
    ComplianceRuleChange,
    FlockParameterChange,
    WormGraphOperation,
    BundleHashtreeChange,
    Other,
}

#[derive(Debug, Clone)]
pub struct GovernanceProposal {
    pub id: String,
    pub description: String,
    pub action: AdministrativeAction,
    pub total_voters: u64,
    pub requested_delay: Duration,
    pub votes: HashSet<String>,
}

impl GovernanceProposal {
    pub fn new(id: String, description: String, action: AdministrativeAction, total_voters: u64, requested_delay: Duration) -> Self {
        Self {
            id,
            description,
            action,
            total_voters,
            requested_delay,
            votes: HashSet::new(),
        }
    }

    pub fn vote_for(&mut self, did: String) {
        self.votes.insert(did);
    }
}

#[derive(Debug, Clone)]
pub struct ExecutedProposal {
    pub proposal: GovernanceProposal,
    pub executed_at: DateTime<Utc>,
    pub result: ExecutionResult,
}

#[derive(Debug, Clone)]
pub enum ExecutionResult {
    Success,
    Rejected(String),
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct ExecutedAction {
    pub id: [u8; 32],
    pub class: ActionClass,
    pub executed_at: DateTime<Utc>,
    pub action_hash: [u8; 32],
    pub result: ExecutionResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceAction {
    pub id: [u8; 32],
    pub class: ActionClass,
    pub description: String,
    pub proposer_did: String,
    pub created_at: DateTime<Utc>,
    pub requested_delay: std::time::Duration,
    pub votes_for: HashSet<String>,
    pub votes_against: HashSet<String>,
    pub action_hash: [u8; 32],
    pub revokes: Option<[u8; 32]>,
}

static ACTION_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl GovernanceAction {
    pub fn new(
        class: ActionClass,
        description: String,
        proposer_did: String,
        requested_delay: std::time::Duration,
        action_hash: [u8; 32],
    ) -> Self {
        let nonce = ACTION_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut hasher = blake3::Hasher::new();
        hasher.update(&nonce.to_le_bytes());
        hasher.update(class.name().as_bytes());
        hasher.update(proposer_did.as_bytes());
        hasher.update(&action_hash);

        Self {
            id: *hasher.finalize().as_bytes(),
            class,
            description,
            proposer_did,
            created_at: Utc::now(),
            requested_delay,
            votes_for: HashSet::new(),
            votes_against: HashSet::new(),
            action_hash,
            revokes: None,
        }
    }

    pub fn canonical_hash(&self) -> [u8; 32] {
        let bytes = serde_json::to_vec(self).expect("serialize");
        blake3::hash(&bytes).into()
    }

    pub fn earliest_execution(&self) -> DateTime<Utc> {
        let chrono_delay = Duration::from_std(self.requested_delay)
            .expect("Delay must fit in i64 nanoseconds");
        self.created_at + chrono_delay
    }
}

pub struct CheckResult {
    pub satisfied: bool,
    pub summary: String,
}

impl CheckResult {
    pub fn summary(&self) -> String {
        self.summary.clone()
    }
}

pub struct GovernanceInvariantChecker {
    pub revocation_window: std::time::Duration,
}

impl Default for GovernanceInvariantChecker {
    fn default() -> Self {
        Self {
            revocation_window: std::time::Duration::from_secs(24 * 3600),
        }
    }
}

impl GovernanceInvariantChecker {
    pub fn check(&self, _action: &GovernanceAction) -> CheckResult {
        CheckResult {
            satisfied: true,
            summary: "Satisfied".to_string(),
        }
    }

    pub fn check_revocation(&self, target: &GovernanceAction) -> Result<(), String> {
        let elapsed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() - target.created_at.timestamp() as u64;
        if elapsed > self.revocation_window.as_secs() {
            return Err("Revocation window expired".to_string());
        }
        Ok(())
    }

    pub fn record_execution(&mut self, _proposal: &GovernanceAction, _result: ExecutionResult) {}
}

#[derive(Debug, thiserror::Error)]
pub enum GovernanceViolation {
    #[error("Violated: {0}")]
    Violated(String),
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum GovernanceError {
    #[error("Error: {0}")]
    Error(String),
}
