sed -i 's/    model::{ServerInfo, ServerCapabilities, CallToolResult, Content},/    model::{ServerInfo, ServerCapabilities},/g' crates/safe-core-governance/src/mcp.rs
sed -i 's/    CallToolResult,/    /g' crates/safe-core-governance/src/mcp.rs
sed -i 's/    Content,/    /g' crates/safe-core-governance/src/mcp.rs
sed -i 's/CallToolResult/serde_json::Value/g' crates/safe-core-governance/src/mcp.rs
sed -i 's/Content::text(serde_json::to_string_pretty(&output).unwrap_or_default())/serde_json::to_value(&output).unwrap_or_default()/g' crates/safe-core-governance/src/mcp.rs
sed -i 's/Ok(serde_json::Value::success(vec!\[/Ok(/g' crates/safe-core-governance/src/mcp.rs
sed -i 's/Ok(serde_json::Value::success(vec!\[//g' crates/safe-core-governance/src/mcp.rs
sed -i 's/\]))//g' crates/safe-core-governance/src/mcp.rs
sed -i 's/pub struct EnforceParams/pub struct EnforceParamsData/g' crates/safe-core-governance/src/mcp.rs
sed -i 's/#[derive(Debug, Clone, serde::Deserialize)]//g' crates/safe-core-governance/src/mcp.rs
sed -i 's/#[derive(Debug, Clone, Deserialize, JsonSchema)]//g' crates/safe-core-governance/src/mcp.rs
