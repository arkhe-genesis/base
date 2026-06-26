use cathedral_scheduler::{
    HybridScheduler, TaskType, TeeType, WorkerProfile, WorkerRegistry, WorkerTier,
};
use std::sync::Arc;

#[tokio::test]
async fn test_register_and_schedule() {
    let registry = Arc::new(WorkerRegistry::new());
    let scheduler = HybridScheduler::new(registry.clone(), 15000, 0.7, 100000);

    let profile = WorkerProfile {
        worker_id: "worker1".to_string(),
        tier: WorkerTier::DePIN_GPU,
        endpoint: "http://localhost:8080".to_string(),
        cost_per_hour: 0.10,
        latency_p50_ms: 200,
        latency_p95_ms: 500,
        reputation: 0.9,
        stake_sats: 200000,
        last_attestation: 0,
        tasks_completed: 0,
        tasks_failed: 0,
        available: true,
        tee_type: TeeType::IoNet,
        capabilities: vec!["cuda".to_string()],
    };

    registry.register(profile).await.unwrap();

    let decision = scheduler.schedule(TaskType::Inference).await;
    assert_eq!(decision.selected_worker, "worker1");
}

#[tokio::test]
async fn test_no_workers_fallback() {
    let registry = Arc::new(WorkerRegistry::new());
    let scheduler = HybridScheduler::new(registry, 15000, 0.7, 100000);

    let decision = scheduler.schedule(TaskType::Inference).await;
    assert_eq!(decision.selected_worker, "datacenter-local");
}
