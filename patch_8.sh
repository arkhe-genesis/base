sed -i 's/ServerInfo {/ServerInfo::new("Safe-Core Governance MCP Server".to_string(), env!("CARGO_PKG_VERSION").to_string())/' crates/safe-core-governance/src/mcp.rs
sed -i 's/            name: "Safe-Core Governance MCP Server".to_string(),//' crates/safe-core-governance/src/mcp.rs
sed -i 's/            version: env!("CARGO_PKG_VERSION").to_string(),//' crates/safe-core-governance/src/mcp.rs
sed -i 's/            instructions:/;/' crates/safe-core-governance/src/mcp.rs
sed -i 's/            capabilities: ServerCapabilities::builder()/        ServerInfo::new("Safe-Core Governance MCP Server".to_string(), env!("CARGO_PKG_VERSION").to_string())/' crates/safe-core-governance/src/mcp.rs
