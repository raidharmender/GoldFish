use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
  pub id: String,
  pub profile: serde_json::Value,
  pub frozen: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
  sqlx::query(
    r#"
    create table if not exists goldfish_customers (
      id text primary key,
      profile jsonb not null default '{}'::jsonb,
      frozen boolean not null default false,
      created_at timestamptz not null default now(),
      updated_at timestamptz not null default now()
    );
    "#,
  )
  .execute(pool)
  .await
  .context("create goldfish_customers")?;
  Ok(())
}

pub async fn upsert_profile(pool: &PgPool, id: &str, patch: serde_json::Value) -> anyhow::Result<Customer> {
  // Merge patch into existing profile (shallow object merge).
  let rec = sqlx::query(
    r#"
    insert into goldfish_customers (id, profile)
    values ($1, $2::jsonb)
    on conflict (id) do update
      set profile = coalesce(goldfish_customers.profile, '{}'::jsonb) || excluded.profile,
          updated_at = now()
    returning id, profile, frozen, created_at, updated_at
    "#,
  )
  .bind(id)
  .bind(patch)
  .fetch_one(pool)
  .await?;

  Ok(Customer {
    id: rec.get("id"),
    profile: rec.get("profile"),
    frozen: rec.get("frozen"),
    created_at: rec.get("created_at"),
    updated_at: rec.get("updated_at"),
  })
}

pub async fn get(pool: &PgPool, id: &str) -> anyhow::Result<Option<Customer>> {
  let rec = sqlx::query(
    r#"
    select id, profile, frozen, created_at, updated_at
    from goldfish_customers
    where id = $1
    "#,
  )
  .bind(id)
  .fetch_optional(pool)
  .await?;

  Ok(rec.map(|r| Customer {
    id: r.get("id"),
    profile: r.get("profile"),
    frozen: r.get("frozen"),
    created_at: r.get("created_at"),
    updated_at: r.get("updated_at"),
  }))
}

