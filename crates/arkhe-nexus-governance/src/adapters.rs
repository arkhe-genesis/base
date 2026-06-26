use arkhe_governance::{
    AdministrativeAction, GovernanceGuard, GovernanceProposal, GovernanceError,
    ExecutionResult, GovernanceViolation,
};
use chrono::Duration;

pub struct NexusGovernanceAdapter {
    guard: std::sync::Arc<GovernanceGuard>,
}

impl NexusGovernanceAdapter {
    pub fn new() -> Self {
        Self {
            guard: std::sync::Arc::new(GovernanceGuard::new()),
        }
    }

    pub fn with_guard(guard: std::sync::Arc<GovernanceGuard>) -> Self {
        Self { guard }
    }

    pub fn execute_admin_action<F>(
        &self,
        proposal: GovernanceProposal,
        action: F,
    ) -> Result<ExecutionResult, NexusGovernanceError>
    where
        F: FnOnce(&GovernanceProposal) -> Result<(), Box<dyn std::error::Error + Send + Sync>>,
    {
        self.guard.submit(proposal.clone())
            .map_err(NexusGovernanceError::GovernanceViolation)?;

        self.guard.execute(&proposal.id, action)
            .map_err(NexusGovernanceError::GovernanceError)
    }

    pub fn cancel_admin_action(
        &self,
        proposal_id: &str,
        cancellation_proposal: &GovernanceProposal,
    ) -> Result<(), NexusGovernanceError> {
        self.guard.cancel(proposal_id, cancellation_proposal)
            .map_err(NexusGovernanceError::GovernanceError)
    }

    pub fn pending_actions(&self) -> Vec<GovernanceProposal> {
        self.guard.pending_proposals()
    }

    pub fn executed_actions(&self) -> Vec<arkhe_governance::ExecutedProposal> {
        self.guard.executed_proposals()
    }

    pub fn audit_hash(&self) -> [u8; 32] {
        self.guard.audit_hash()
    }

    pub fn is_action_audited(&self, proposal_id: &str) -> bool {
        self.guard.executed_proposals()
            .iter()
            .any(|ep| ep.proposal.id == proposal_id)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NexusGovernanceError {
    #[error("Governance invariant violated: {0}")]
    GovernanceViolation(#[from] GovernanceViolation),

    #[error("Governance operation failed: {0}")]
    GovernanceError(#[from] GovernanceError),

    #[error("NEXUS action failed: {0}")]
    NexusActionFailed(String),

    #[error("Action not found in audit trail: {0}")]
    ActionNotAudited(String),
}
