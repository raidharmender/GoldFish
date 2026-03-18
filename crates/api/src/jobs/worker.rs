use sqlx::PgPool;
use tracing::{error, info};

pub fn start(pool: PgPool) {
  let worker_id = format!("worker-{}", uuid::Uuid::new_v4());
  tokio::spawn(async move {
    loop {
      match goldfish_storage::jobs::fetch_and_lock(&pool, &worker_id).await {
        Ok(Some(job)) => {
          let job_id = job.id;
          info!(job_id, kind = %job.kind, attempts = job.attempts, "picked job");

          let result: anyhow::Result<()> = match job.kind.as_str() {
            "webhook.plaid" => {
              // placeholder: actual handlers will be added per domain
              Ok(())
            }
            "webhook.bitgo" => Ok(()),
            "webhook.docusign" => Ok(()),
            "webhook.modern_treasury" => Ok(()),
            "webhook.taxbit" => Ok(()),
            "webhook.cognito" => Ok(()),
            _ => Ok(()),
          };

          match result {
            Ok(()) => {
              let _ = goldfish_storage::jobs::complete(&pool, job_id).await;
            }
            Err(e) => {
              error!(job_id, error=%e, "job failed");
              let _ = goldfish_storage::jobs::fail_and_reschedule(&pool, job_id, &e.to_string(), 30).await;
            }
          }
        }
        Ok(None) => {
          tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
        Err(e) => {
          error!(error=%e, "job fetch failed");
          tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
      }
    }
  });
}

