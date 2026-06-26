use arkhe_agents::agent::AgentManager;
use arkhe_agents::intent::IntentScheduler;
use arkhe_core::types::{Intent, AssetRef, Did};

#[tokio::test]
async fn test_agent_lifecycle() {
    let mut manager = AgentManager::new();
    let id = manager.create_agent("test-agent");
    let agent = manager.get_agent(id).unwrap();
    assert_eq!(agent.name, "test-agent");
}

#[tokio::test]
async fn test_intent_scheduler() {
    let mut scheduler = IntentScheduler::new();
    let intent = Intent::TransferAsset {
        asset: AssetRef { chain: "test".to_string(), id: "asset1".to_string() },
        amount: 100,
        recipient: Did::new("test", "user"),
        priority: 1,
    };
    scheduler.submit(intent);
    let next = scheduler.schedule_next();
    assert!(next.is_some());
}
