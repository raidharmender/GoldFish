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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn ok_has_status_ok() {
    let res = HealthResponse::ok();
    assert_eq!(res.status, "ok");
  }

  #[test]
  fn ok_sets_timestamp_close_to_now() {
    let before = Utc::now();
    let res = HealthResponse::ok();
    let after = Utc::now();

    assert!(res.now_utc >= before);
    assert!(res.now_utc <= after);
  }
}
