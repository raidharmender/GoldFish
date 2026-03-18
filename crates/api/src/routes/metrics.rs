use actix_web::{get, web, HttpResponse};
use prometheus::{Encoder, TextEncoder};
use crate::middleware::basic_auth::BasicAuth;

pub fn configure(cfg: &mut web::ServiceConfig) {
  let user = std::env::var("GOLDFISH__METRICS_USER").unwrap_or_else(|_| "metrics".to_string());
  let pass = std::env::var("GOLDFISH__METRICS_PASS").unwrap_or_else(|_| "metrics".to_string());
  cfg.service(web::scope("").wrap(BasicAuth::new(user, pass)).service(metrics));
}

#[get("/metrics")]
async fn metrics() -> HttpResponse {
  let encoder = TextEncoder::new();
  let metric_families = prometheus::gather();

  let mut buf = Vec::with_capacity(16 * 1024);
  if let Err(e) = encoder.encode(&metric_families, &mut buf) {
    return HttpResponse::InternalServerError().body(format!("encode metrics failed: {e}"));
  }

  HttpResponse::Ok()
    .content_type(encoder.format_type())
    .body(buf)
}

