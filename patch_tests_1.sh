sed -i 's/async fn clear_violations(_state: \&Arc<BridgeState>) -> serde_json::Value/pub async fn clear_violations(_state: \&Arc<BridgeState>) -> serde_json::Value/' crates/safe-core-bridge/src/tools.rs
sed -i 's/let result = clear_violations(\&state()).await;/let result = clear_violations(\&state()).await;/' crates/safe-core-bridge/src/tools.rs
sed -i 's/async fn health_check(_state: \&Arc<BridgeState>) -> HealthResponse/pub async fn health_check(_state: \&Arc<BridgeState>) -> HealthResponse/' crates/safe-core-bridge/src/tools.rs
sed -i 's/use chrono::{DateTime, Utc};/use chrono::{DateTime, Utc};/' crates/safe-core-bridge/src/api.rs
