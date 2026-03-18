use actix_web::{get, web, HttpResponse};
use prometheus::{Encoder, TextEncoder};

pub fn configure(cfg: &mut web::ServiceConfig) {
  cfg.service(metrics);
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

