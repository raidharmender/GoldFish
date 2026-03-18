use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct HealthResponse {
  pub status: &'static str,
  pub now_utc: DateTime<Utc>,
}

impl HealthResponse {
  pub fn ok() -> Self {
    Self {
      status: "ok",
      now_utc: Utc::now(),
    }
  }
}
