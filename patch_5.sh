sed -i 's/pub struct CreateRuleParamsData/pub struct CreateRuleParamsData/g' crates/safe-core-governance/src/mcp.rs
sed -i 's/pub struct VerifyParamsData/pub struct VerifyParamsData/g' crates/safe-core-governance/src/mcp.rs
sed -i 's/#[tool(description = "")]//g' crates/safe-core-governance/src/mcp.rs
sed -i 's/#[tool(description="")]//g' crates/safe-core-governance/src/mcp.rs
sed -i 's/use safe_core_verifier::{Lean4Verifier, Verifier, Constraint, ConstraintResult};//' crates/safe-core-governance/src/governance.rs
sed -i 's/use safe_core_verifier::Constraint;//' crates/safe-core-governance/src/mcp.rs
sed -i 's/use crate::verifier::Lean4Verifier;//' crates/safe-core-governance/src/governance.rs
sed -i 's/use crate::verifier::{Verifier, Constraint, ConstraintResult};//' crates/safe-core-governance/src/governance.rs
