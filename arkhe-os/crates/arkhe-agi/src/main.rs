use arkhe_agi::coordinator::AgiCoordinator;
use arkhe_llm::inference::LlamaCppEngine;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("Starting Arkhe AGI...");
    let engine = LlamaCppEngine::new("model.bin");
    let coordinator = AgiCoordinator::new(Arc::new(engine));
    // Since run is an infinite loop and requires ! return type, we mock it
    // or run in a spawned task for graceful exit in tests.
    // For this demonstration, we'll spawn and sleep.
    tokio::spawn(async move {
        coordinator.run().await;
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    println!("Arkhe AGI initialized successfully.");
}
