pub mod cca_v2;
pub use cca_v2::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[async_trait::async_trait]
pub trait LlmClient: Send + Sync {
    async fn chat_completion(
        &self,
        messages: &[AgentMessage],
        tools: Option<serde_json::Value>,
    ) -> Result<String, String>;
    async fn chat_completion_stream(
        &self,
        messages: &[AgentMessage],
        tools: Option<serde_json::Value>,
    ) -> Result<Box<dyn futures::Stream<Item = Result<String, String>> + Send + Unpin>, String>;

    fn clone_arc(&self) -> std::sync::Arc<dyn LlmClient + Send + Sync>;
}
