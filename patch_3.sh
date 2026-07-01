sed -i 's/use rmcp::{tool, tool/use rmcp::{tool/g' crates/safe-core-governance/src/mcp.rs
sed -i 's/use crate::ethics::{EthicsEngine, EthicsVerdict};/\/\/\/ GovernanceEngine/' crates/safe-core-governance/src/governance.rs
sed -i '1i use safe_core_ethics::{EthicsEngine, EthicsRule, EthicsVerdict};' crates/safe-core-governance/src/governance.rs
sed -i 's/use crate::verifier::Constraint;/use safe_core_verifier::Constraint;/' crates/safe-core-governance/src/mcp.rs
sed -i 's/use crate::ethics::{EthicsRule, Severity};/use safe_core_ethics::{EthicsRule, Severity};/' crates/safe-core-governance/src/mcp.rs
sed -i 's/use async_trait::async_trait;//' crates/safe-core-bridge/src/state.rs
