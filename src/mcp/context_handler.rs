pub fn context_tool_definitions() -> Vec<serde_json::Value> { vec![] }
pub async fn handle_create_context(
    args: serde_json::Value,
    state: &std::sync::Arc<crate::mcp::server::MCPServerState>,
) -> Result<serde_json::Value, crate::mcp::server::MCPError> {
    Ok(serde_json::json!({}))
}
pub async fn handle_add_to_context(
    args: serde_json::Value,
    state: &std::sync::Arc<crate::mcp::server::MCPServerState>,
) -> Result<serde_json::Value, crate::mcp::server::MCPError> {
    Ok(serde_json::json!({}))
}
pub async fn handle_get_context(
    args: serde_json::Value,
    state: &std::sync::Arc<crate::mcp::server::MCPServerState>,
) -> Result<serde_json::Value, crate::mcp::server::MCPError> {
    Ok(serde_json::json!({}))
}
pub async fn handle_list_contexts(
    args: serde_json::Value,
    state: &std::sync::Arc<crate::mcp::server::MCPServerState>,
) -> Result<serde_json::Value, crate::mcp::server::MCPError> {
    Ok(serde_json::json!({}))
}
pub async fn handle_clear_context(
    args: serde_json::Value,
    state: &std::sync::Arc<crate::mcp::server::MCPServerState>,
) -> Result<serde_json::Value, crate::mcp::server::MCPError> {
    Ok(serde_json::json!({}))
}
