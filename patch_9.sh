sed -i 's/ServerInfo::new("Safe-Core Governance MCP Server".to_string(), env!("CARGO_PKG_VERSION").to_string())/ServerInfo { name: "Safe-Core Governance MCP Server".to_string(), version: env!("CARGO_PKG_VERSION").to_string() }/' crates/safe-core-governance/src/mcp.rs
sed -i 's/use rmcp::{/use rmcp::{model::ServerInfo, /' crates/safe-core-governance/src/mcp.rs
