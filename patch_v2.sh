sed -i 's/pub use mcp_impl::SafeCoreMcpServer;//g' crates/safe-core-bridge/src/mcp.rs
sed -i 's/pub use mcp::SafeCoreMcpServer;//g' crates/safe-core-bridge/src/lib.rs
sed -i 's/use safe_core_bridge::{handlers, BridgeState, SafeCoreMcpServer};/use safe_core_bridge::{handlers, BridgeState};\nuse safe_core_bridge::mcp::mcp_impl::SafeCoreMcpServer;/g' crates/safe-core-bridge/src/main.rs
sed -i 's/allowed: result.is_allowed()/allowed: result.verdict != "Block"/' crates/safe-core-bridge/src/tools.rs
