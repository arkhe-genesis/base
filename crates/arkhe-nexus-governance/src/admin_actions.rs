use arkhe_governance::AdministrativeAction;
use chrono::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NexusAdminAction {
    KernelUpdate,
    SecurityPolicyChange,
    CapsuleModification,
    ComplianceRulesUpdate,
    FlockConfigUpdate,
    WormGraphOperation,
    BundleHashtreeUpdate,
    Other,
}

impl NexusAdminAction {
    pub fn to_generic(&self) -> AdministrativeAction {
        match self {
            Self::KernelUpdate => AdministrativeAction::SafeCoreUpdate,
            Self::SecurityPolicyChange => AdministrativeAction::SafeCoreUpdate,
            Self::CapsuleModification => AdministrativeAction::CapsulePrivilegeChange,
            Self::ComplianceRulesUpdate => AdministrativeAction::ComplianceRuleChange,
            Self::FlockConfigUpdate => AdministrativeAction::FlockParameterChange,
            Self::WormGraphOperation => AdministrativeAction::WormGraphOperation,
            Self::BundleHashtreeUpdate => AdministrativeAction::BundleHashtreeChange,
            Self::Other => AdministrativeAction::Other,
        }
    }

    pub fn to_proposal(
        &self,
        id: String,
        description: String,
        total_voters: u64,
        delay_hours: i64,
    ) -> arkhe_governance::GovernanceProposal {
        arkhe_governance::GovernanceProposal::new(
            id,
            description,
            self.to_generic(),
            total_voters,
            Duration::hours(delay_hours),
        )
    }
}
