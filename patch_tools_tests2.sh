sed -i 's/async fn clear_violations(_state: \&Arc<BridgeState>)/async fn clear_violations(_state: \&Arc<BridgeState>) -> serde_json::Value/' crates/safe-core-bridge/src/tools.rs
sed -i 's/async fn health_check(_state: \&Arc<BridgeState>) -> HealthResponse/async fn health_check(_state: \&Arc<BridgeState>) -> HealthResponse/' crates/safe-core-bridge/src/tools.rs
sed -i 's/let resp = health_check(\&state());/let resp = health_check(\&state()).await;/' crates/safe-core-bridge/src/tools.rs
