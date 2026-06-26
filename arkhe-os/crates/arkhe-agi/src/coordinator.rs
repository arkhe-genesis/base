use arkhe_agents::{agent::AgentManager, intent::IntentScheduler};
use arkhe_llm::inference::InferenceEngine;
use arkhe_identity::mldsa::MldsaSigner;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AgiCoordinator {
    agents: Arc<Mutex<AgentManager>>,
    scheduler: Arc<Mutex<IntentScheduler>>,
    llm: Arc<dyn InferenceEngine>,
    signer: MldsaSigner,
}

impl AgiCoordinator {
    pub fn new(llm: Arc<dyn InferenceEngine>) -> Self {
        Self {
            agents: Arc::new(Mutex::new(AgentManager::new())),
            scheduler: Arc::new(Mutex::new(IntentScheduler::new())),
            llm,
            signer: MldsaSigner::generate(),
        }
    }

    pub async fn run(&self) -> ! {
        loop {
            if let Some(intent) = self.scheduler.lock().await.schedule_next() {
                // Processa intenção
                println!("Executing intent: {:?}", intent);
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
}
