use anyhow::Context;
use deadpool_redis::{Config, Pool, Runtime};

#[derive(Debug, Clone)]
pub struct RedisStore {
  pub pool: Pool,
}

impl RedisStore {
  pub fn connect(redis_url: &str) -> anyhow::Result<Self> {
    let cfg = Config::from_url(redis_url);
    let pool = cfg
      .create_pool(Some(Runtime::Tokio1))
      .context("create redis pool")?;
    Ok(Self { pool })
  }
}

