use crate::guard::{GovernanceGuard, GuardError};
use crate::invariants::GovernanceAction;

pub struct AsyncGovernanceGuard {
    inner: tokio::sync::Mutex<GovernanceGuard>,
}

impl AsyncGovernanceGuard {
    pub fn new() -> Self {
        Self { inner: tokio::sync::Mutex::new(GovernanceGuard::new()) }
    }

    pub async fn submit(&self, action: GovernanceAction) -> Result<String, GuardError> {
        let guard = self.inner.lock().await;
        guard.submit_action(action)
    }

    pub async fn execute<F, R>(&self, proposal_id: &str, action: F) -> Result<R, GuardError>
    where
        F: FnOnce(&GovernanceAction) -> Result<R, String>,
    {
        let guard = self.inner.lock().await;
        guard.execute_action(proposal_id, action)
    }
}
