use cathedral_fallback::{FallbackChain, WorkerExecutor, WorkerTier};

#[tokio::test]
async fn test_fallback_execution() {
    let mut chain = FallbackChain::new(5000);

    chain.add_worker(WorkerExecutor {
        id: "worker1".to_string(),
        tier: WorkerTier::DePIN_GPU,
        endpoint: "http://localhost:8080".to_string(),
        tee_attested: true,
    });

    let result = chain.execute("test task").await;
    assert!(result.is_ok());
}
