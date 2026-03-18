use actix_web::{get, web, HttpResponse};

pub fn configure(cfg: &mut web::ServiceConfig) {
  cfg.service(metrics);
}

#[get("/metrics")]
async fn metrics() -> HttpResponse {
  // Placeholder until we wire real Prometheus registry/encoders.
  HttpResponse::Ok()
    .content_type("text/plain; version=0.0.4")
    .body("# metrics not yet wired\n")
}

