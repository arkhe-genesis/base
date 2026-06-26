use std::sync::{Mutex, MutexGuard};
use crate::invariants::{
    GovernanceAction, GovernanceProposal, GovernanceInvariantChecker, GovernanceError,
    GovernanceViolation, ExecutionResult, ExecutedAction, ExecutedProposal
};
use crate::safe_core::SafeCoreHook;

#[derive(Debug, thiserror::Error)]
pub enum GuardError {
    #[error("Cancellation denied: {0}")]
    CancellationDenied(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(Box<dyn std::error::Error + Send + Sync>),
}

pub struct GovernanceGuard {
    checker: Mutex<GovernanceInvariantChecker>,
    pending: Mutex<Vec<GovernanceAction>>,
    executed: Mutex<Vec<ExecutedAction>>,
    executed_proposals: Mutex<Vec<ExecutedProposal>>,
    pending_proposals: Mutex<Vec<GovernanceProposal>>,
    hooks: Mutex<Vec<Box<dyn SafeCoreHook>>>,
}

impl GovernanceGuard {
    pub fn new() -> Self {
        Self {
            checker: Mutex::new(GovernanceInvariantChecker::default()),
            pending: Mutex::new(Vec::new()),
            executed: Mutex::new(Vec::new()),
            executed_proposals: Mutex::new(Vec::new()),
            pending_proposals: Mutex::new(Vec::new()),
            hooks: Mutex::new(Vec::new()),
        }
    }

    pub fn with_checker(checker: GovernanceInvariantChecker) -> Self {
        Self {
            checker: Mutex::new(checker),
            pending: Mutex::new(Vec::new()),
            executed: Mutex::new(Vec::new()),
            executed_proposals: Mutex::new(Vec::new()),
            pending_proposals: Mutex::new(Vec::new()),
            hooks: Mutex::new(Vec::new()),
        }
    }

    pub fn checker(&self) -> MutexGuard<GovernanceInvariantChecker> {
        self.checker.lock().unwrap()
    }

    pub fn submit(&self, proposal: GovernanceProposal) -> Result<(), GovernanceViolation> {
        self.pending_proposals.lock().unwrap().push(proposal);
        Ok(())
    }

    pub fn submit_action(&self, action: GovernanceAction) -> Result<String, GuardError> {
        self.run_pre_submit_hooks(&action)?;
        self.pending.lock().unwrap().push(action.clone());
        Ok(hex::encode(action.id))
    }

    pub fn execute<F>(&self, proposal_id: &str, action: F) -> Result<ExecutionResult, GovernanceError>
    where
        F: FnOnce(&GovernanceProposal) -> Result<(), Box<dyn std::error::Error + Send + Sync>>,
    {
        let pending = self.pending_proposals.lock().unwrap();
        let proposal = pending.iter().find(|p| p.id == proposal_id)
            .cloned()
            .ok_or_else(|| GovernanceError::Error("Proposal not found".to_string()))?;
        drop(pending);

        let action_result = action(&proposal);

        let execution_result = if let Err(e) = &action_result {
            ExecutionResult::Rejected(e.to_string())
        } else {
            ExecutionResult::Success
        };

        self.executed_proposals.lock().unwrap().push(ExecutedProposal {
            proposal,
            executed_at: chrono::Utc::now(),
            result: execution_result.clone(),
        });

        Ok(execution_result)
    }

    pub fn execute_action<F, R>(&self, proposal_id: &str, action: F) -> Result<R, GuardError>
    where
        F: FnOnce(&GovernanceAction) -> Result<R, String>,
    {
        let mut pending = self.pending.lock().unwrap();
        let pos = pending.iter().position(|p| hex::encode(p.id) == proposal_id)
            .ok_or_else(|| GuardError::NotFound(proposal_id.to_string()))?;

        let proposal = pending.remove(pos);
        drop(pending);

        let action_result = action(&proposal);

        let execution_result = if let Err(e) = &action_result {
            ExecutionResult::Rejected(e.clone())
        } else {
            ExecutionResult::Success
        };

        {
            let mut checker = self.checker.lock().unwrap();
            checker.record_execution(&proposal, execution_result.clone());
        }

        self.executed.lock().unwrap().push(ExecutedAction {
            id: proposal.id,
            class: proposal.class.clone(),
            executed_at: chrono::Utc::now(),
            action_hash: proposal.action_hash,
            result: execution_result,
        });

        action_result.map_err(|e| GuardError::ExecutionFailed(e.into()))
    }

    pub fn cancel(&self, proposal_id: &str, _cancellation: &GovernanceProposal) -> Result<(), GovernanceError> {
        let mut pending = self.pending_proposals.lock().unwrap();
        let pos = pending.iter().position(|p| p.id == proposal_id)
            .ok_or_else(|| GovernanceError::Error("Proposal not found".to_string()))?;
        pending.remove(pos);
        Ok(())
    }

    pub fn cancel_action(&self, proposal_id: &str, cancellation: &GovernanceAction) -> Result<(), GuardError> {
        let check = self.checker.lock().unwrap().check(cancellation);
        if !check.satisfied {
            return Err(GuardError::CancellationDenied(check.summary()));
        }

        let mut pending = self.pending.lock().unwrap();
        let pos = pending.iter().position(|p| hex::encode(p.id) == proposal_id)
            .ok_or_else(|| GuardError::NotFound(proposal_id.to_string()))?;

        let target = &pending[pos];

        if let Err(e) = self.checker.lock().unwrap().check_revocation(target) {
            return Err(GuardError::CancellationDenied(e.to_string()));
        }

        pending.remove(pos);
        Ok(())
    }

    pub fn pending_proposals(&self) -> Vec<GovernanceProposal> {
        self.pending_proposals.lock().unwrap().clone()
    }

    pub fn executed_proposals(&self) -> Vec<ExecutedProposal> {
        self.executed_proposals.lock().unwrap().clone()
    }

    pub fn audit_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        for p in self.executed_proposals.lock().unwrap().iter() {
            hasher.update(p.proposal.id.as_bytes());
        }
        *hasher.finalize().as_bytes()
    }

    fn run_pre_submit_hooks(&self, action: &GovernanceAction) -> Result<(), GuardError> {
        let hooks = self.hooks.lock().unwrap();
        for hook in hooks.iter() {
            if let Err(e) = hook.pre_submit(action) {
                return Err(GuardError::CancellationDenied(e.to_string()));
            }
        }
        Ok(())
    }
}
