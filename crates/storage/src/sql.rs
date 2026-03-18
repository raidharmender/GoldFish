use anyhow::Context;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SqlStore {
  pub pool: PgPool,
}

impl SqlStore {
  pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
    let pool = PgPoolOptions::new()
      .max_connections(32)
      .acquire_timeout(Duration::from_secs(3))
      .connect(database_url)
      .await
      .context("connect postgres")?;

    Ok(Self { pool })
  }
}
