use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPayload {
  pub kind: String,
  pub args: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct JobRecord {
  pub id: i64,
  pub kind: String,
  pub args: serde_json::Value,
  pub attempts: i32,
  pub run_at: DateTime<Utc>,
}

pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
  // Minimal Oban-like tables (idempotent). We’ll migrate to proper migration files later.
  sqlx::query(
    r#"
    create table if not exists goldfish_jobs (
      id bigserial primary key,
      kind text not null,
      args jsonb not null default '{}'::jsonb,
      attempts int not null default 0,
      max_attempts int not null default 20,
      run_at timestamptz not null default now(),
      locked_at timestamptz,
      locked_by text,
      last_error text,
      created_at timestamptz not null default now(),
      updated_at timestamptz not null default now()
    );
    create index if not exists goldfish_jobs_run_at_idx on goldfish_jobs (run_at) where locked_at is null;
    "#,
  )
  .execute(pool)
  .await
  .context("create goldfish_jobs")?;

  sqlx::query(
    r#"
    create table if not exists goldfish_idempotency_keys (
      scope text not null,
      key text not null,
      first_seen_at timestamptz not null default now(),
      primary key (scope, key)
    );
    "#,
  )
  .execute(pool)
  .await
  .context("create goldfish_idempotency_keys")?;

  Ok(())
}

pub async fn claim_idempotency(pool: &PgPool, scope: &str, key: &str) -> anyhow::Result<bool> {
  let rows = sqlx::query(
    r#"
    insert into goldfish_idempotency_keys (scope, key)
    values ($1, $2)
    on conflict do nothing
    "#,
  )
  .bind(scope)
  .bind(key)
  .execute(pool)
  .await?;

  Ok(rows.rows_affected() == 1)
}

pub async fn enqueue(pool: &PgPool, kind: &str, args: serde_json::Value, run_at: DateTime<Utc>) -> anyhow::Result<i64> {
  let rec = sqlx::query(
    r#"
    insert into goldfish_jobs (kind, args, run_at)
    values ($1, $2, $3)
    returning id
    "#,
  )
  .bind(kind)
  .bind(args)
  .bind(run_at)
  .fetch_one(pool)
  .await?;

  Ok(rec.get::<i64, _>("id"))
}

pub async fn fetch_and_lock(pool: &PgPool, worker_id: &str) -> anyhow::Result<Option<JobRecord>> {
  let rec = sqlx::query(
    r#"
    with next_job as (
      select id
      from goldfish_jobs
      where locked_at is null
        and run_at <= now()
      order by run_at asc, id asc
      for update skip locked
      limit 1
    )
    update goldfish_jobs j
    set locked_at = now(), locked_by = $1, updated_at = now()
    from next_job
    where j.id = next_job.id
    returning j.id, j.kind, j.args, j.attempts, j.run_at
    "#,
  )
  .bind(worker_id)
  .fetch_optional(pool)
  .await?;

  Ok(rec.map(|r| JobRecord {
    id: r.get("id"),
    kind: r.get("kind"),
    args: r.get("args"),
    attempts: r.get("attempts"),
    run_at: r.get("run_at"),
  }))
}

pub async fn complete(pool: &PgPool, job_id: i64) -> anyhow::Result<()> {
  sqlx::query("delete from goldfish_jobs where id = $1")
    .bind(job_id)
    .execute(pool)
    .await?;
  Ok(())
}

pub async fn fail_and_reschedule(pool: &PgPool, job_id: i64, err: &str, delay_seconds: i64) -> anyhow::Result<()> {
  sqlx::query(
    r#"
    update goldfish_jobs
    set
      attempts = attempts + 1,
      locked_at = null,
      locked_by = null,
      last_error = $2,
      run_at = now() + ($3 || ' seconds')::interval,
      updated_at = now()
    where id = $1
    "#,
  )
  .bind(job_id)
  .bind(err)
  .bind(delay_seconds)
  .execute(pool)
  .await?;
  Ok(())
}

