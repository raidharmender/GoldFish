use actix_web::{get, web, HttpResponse, Responder};
use crate::middleware::api_key::ApiKeyAuth;

pub fn configure(cfg: &mut web::ServiceConfig) {
  let key = std::env::var("GOLDFISH__STORM_API_KEY").unwrap_or_else(|_| "dev".to_string());
  configure_with_key(cfg, key);
}

pub fn configure_with_key(cfg: &mut web::ServiceConfig, key: String) {
  cfg.service(
    web::scope("/storm")
      .service(web::scope("/api/v1").wrap(ApiKeyAuth::new(key)).service(status)),
  );
  cfg.route("/storm/healthcheck", web::get().to(healthcheck));
}

#[get("/status")]
async fn status() -> impl Responder {
  HttpResponse::Ok().json(serde_json::json!({"status":"ok"}))
}

async fn healthcheck() -> impl Responder {
  HttpResponse::Ok().body("ok")
}

