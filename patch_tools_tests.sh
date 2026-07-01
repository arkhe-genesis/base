sed -i 's/async fn clear_violations()/async fn clear_violations(_state: \&Arc<BridgeState>)/' crates/safe-core-bridge/src/tools.rs
sed -i 's/async fn health_check()/async fn health_check(_state: \&Arc<BridgeState>) -> HealthResponse/' crates/safe-core-bridge/src/tools.rs
