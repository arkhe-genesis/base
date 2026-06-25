use std::sync::Arc;

use cathedral_agi::{AGICore, OllamaClient};

#[tokio::test]
async fn test_agi_core_init() {
    let llm = Arc::new(OllamaClient::new("dummy"));
    let mut core = AGICore::new(llm, None);
    // Since Ollama won't be running, we can't fully process, but we test init.
    assert!(core.world_state().is_none());
}
