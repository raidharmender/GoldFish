use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::info;

/// Minimal scheduler scaffold to mirror Oban+Cron/Quantum responsibilities.
/// This will evolve into a durable job system (DB-backed) later.
pub async fn start() -> anyhow::Result<JobScheduler> {
  let sched = JobScheduler::new().await?;

  // Example: heartbeat job every minute.
  sched
    .add(
      Job::new_async("0 * * * * *", |_uuid, _l| {
        Box::pin(async move {
          info!("scheduler heartbeat");
        })
      })?,
    )
    .await?;

  sched.start().await?;
  Ok(sched)
}

