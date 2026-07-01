sed -i 's/pub struct CreateRuleParams/pub struct CreateRuleParamsData/g' crates/safe-core-governance/src/mcp.rs
sed -i 's/pub struct VerifyParams/pub struct VerifyParamsData/g' crates/safe-core-governance/src/mcp.rs
sed -i 's/async fn enforce_action(&self, params: EnforceParamsData)/async fn enforce_action(&self, #[tool(description = "Action parameter")] action: String, #[tool(description = "Context parameter")] context: serde_json::Value, #[tool(description = "Agent ID")] agent_id: Option<String>)/g' crates/safe-core-governance/src/mcp.rs
sed -i 's/params.action/action/g' crates/safe-core-governance/src/mcp.rs
sed -i 's/params.context/context/g' crates/safe-core-governance/src/mcp.rs
sed -i 's/params.agent_id/agent_id/g' crates/safe-core-governance/src/mcp.rs
sed -i 's/async fn create_rule(&self, params: CreateRuleParamsData)/async fn create_rule(&self, action: String, constraint: String, severity: String, enabled: bool)/g' crates/safe-core-governance/src/mcp.rs
sed -i 's/params.severity/severity/g' crates/safe-core-governance/src/mcp.rs
sed -i 's/params.constraint/constraint/g' crates/safe-core-governance/src/mcp.rs
sed -i 's/params.enabled/enabled/g' crates/safe-core-governance/src/mcp.rs
sed -i 's/async fn verify_constraint(&self, params: VerifyParamsData)/async fn verify_constraint(&self, constraint: String, context: serde_json::Value)/g' crates/safe-core-governance/src/mcp.rs
