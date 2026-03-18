use chrono::Utc;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn job_queue_idempotency_smoke() -> anyhow::Result<()> {
  // This test requires a running Postgres. If it's not available, skip without failing CI/local runs.
  let url = std::env::var("GOLDFISH__SQL__DATABASE_URL")
    .ok()
    .unwrap_or_else(|| "postgres://goldfish:goldfish@localhost:5432/goldfish".to_string());

  let pool = match goldfish_storage::sql::SqlStore::connect(&url).await {
    Ok(s) => s.pool,
    Err(_) => return Ok(()),
  };

  goldfish_storage::jobs::migrate(&pool).await?;

  let first = goldfish_storage::jobs::claim_idempotency(&pool, "test", "abc").await?;
  let second = goldfish_storage::jobs::claim_idempotency(&pool, "test", "abc").await?;
  assert!(first);
  assert!(!second);

  let id = goldfish_storage::jobs::enqueue(&pool, "test.job", serde_json::json!({"x":1}), Utc::now()).await?;
  let job = goldfish_storage::jobs::fetch_and_lock(&pool, "test-worker").await?;
  assert!(job.is_some());
  assert_eq!(job.as_ref().unwrap().id, id);

  goldfish_storage::jobs::complete(&pool, id).await?;
  Ok(())
}

