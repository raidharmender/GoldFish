use actix_web::{get, post, web, HttpResponse, Responder};
use crate::auth::{actor::ActorType, jwt::JwtAuth};

pub fn configure(cfg: &mut web::ServiceConfig) {
  let jwks_url = std::env::var("GOLDFISH__JWKS_URL").unwrap_or_else(|_| "http://localhost:9999/.well-known/jwks.json".to_string());
  cfg.service(
    web::scope("/internal/api/v1")
      .wrap(JwtAuth::new(jwks_url, ActorType::Internal))
      .service(health)
      .service(create_audit_log),
  );
}

#[get("/health")]
async fn health() -> impl Responder {
  HttpResponse::Ok().json(serde_json::json!({"status":"ok"}))
}

#[post("/audit_logs")]
async fn create_audit_log(body: web::Json<serde_json::Value>) -> impl Responder {
  HttpResponse::Accepted().json(serde_json::json!({
    "status": "accepted",
    "audit_log": body.into_inner()
  }))
}

