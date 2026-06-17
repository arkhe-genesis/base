pub fn subagent_tool_definitions() -> Vec<serde_json::Value> { vec![] }
pub async fn handle_spawn_subagent(
    args: serde_json::Value,
    state: &std::sync::Arc<crate::mcp::server::MCPServerState>,
) -> Result<serde_json::Value, crate::mcp::server::MCPError> {
    Ok(serde_json::json!({}))
}
pub async fn handle_list_subagents(
    args: serde_json::Value,
    state: &std::sync::Arc<crate::mcp::server::MCPServerState>,
) -> Result<serde_json::Value, crate::mcp::server::MCPError> {
    Ok(serde_json::json!({}))
}
pub async fn handle_terminate_subagent(
    args: serde_json::Value,
    state: &std::sync::Arc<crate::mcp::server::MCPServerState>,
) -> Result<serde_json::Value, crate::mcp::server::MCPError> {
    Ok(serde_json::json!({}))
}
pub async fn handle_execute_subagent(
    args: serde_json::Value,
    state: &std::sync::Arc<crate::mcp::server::MCPServerState>,
) -> Result<serde_json::Value, crate::mcp::server::MCPError> {
    Ok(serde_json::json!({}))
}
