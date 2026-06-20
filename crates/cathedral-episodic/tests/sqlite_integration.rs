use cathedral_episodic::EpisodicSync;
use tempfile::tempdir;

#[tokio::test]
async fn test_sqlite_persistence() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let url = format!("sqlite:{}?mode=rwc", db_path.to_string_lossy());

    let sync = EpisodicSync::new("worker1".to_string(), &url).await.unwrap();

    let id = sync.upsert("Hello", "Hi", 0.9).await.unwrap();
    let results = sync.retrieve("Hello", 10).await;
    assert_eq!(results.len(), 1);

    // Recria a conexão para testar persistência
    let sync2 = EpisodicSync::new("worker1".to_string(), &url).await.unwrap();
    let results2 = sync2.retrieve("Hello", 10).await;
    assert_eq!(results2.len(), 1);
    assert_eq!(results2[0].id, id);
}
