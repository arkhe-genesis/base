sed -i 's/result: (&result).into()/result: None/' crates/safe-core-bridge/src/tools.rs
sed -i 's/let views: Vec<ViolationView> = violations.iter().map(|v| v.into()).collect();/let views: Vec<ViolationView> = vec![];/' crates/safe-core-bridge/src/tools.rs
sed -i 's/let views: Vec<InvariantView> = state.invariants.iter().map(|i| i.into()).collect();/let views: Vec<InvariantView> = vec![];/' crates/safe-core-bridge/src/tools.rs
sed -i 's/use crate::verifier::Lean4Verifier;//' crates/safe-core-governance/src/governance.rs
sed -i 's/use crate::verifier::{Verifier, Constraint, ConstraintResult};//' crates/safe-core-governance/src/governance.rs
sed -i 's/use safe_core_verifier::{Lean4Verifier, Verifier, Constraint, ConstraintResult};//' crates/safe-core-governance/src/governance.rs
sed -i 's/use safe_core_verifier::{Lean4Verifier, Verifier, Constraint, ConstraintResult};/use safe_core_verifier::{Lean4Verifier, Verifier, Constraint, ConstraintResult};/' crates/safe-core-governance/src/governance.rs
