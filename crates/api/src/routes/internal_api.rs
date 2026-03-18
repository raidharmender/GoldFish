use actix_web::{get, post, web, HttpResponse, Responder};

pub fn configure(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::scope("/internal/api/v1")
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

