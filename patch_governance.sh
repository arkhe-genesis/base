sed -i 's/use rmcp::{/use rmcp::{tool,\n/g' crates/safe-core-governance/src/mcp.rs
sed -i 's/^\/\/\/ GovernanceEngine/use crate::ethics::{EthicsEngine, EthicsVerdict};\n\/\/\/ GovernanceEngine/' crates/safe-core-governance/src/governance.rs
sed -i 's/use crate::ethics::{EthicsEngine, EthicsRule, EthicsVerdict};//' crates/safe-core-governance/src/governance.rs
sed -i 's/use crate::persistence::{StateRepository, RepositoryError};/use crate::persistence::StateRepository;/' crates/safe-core-governance/src/governance.rs
