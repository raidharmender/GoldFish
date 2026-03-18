use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use chrono::Utc;
use goldfish_storage::jobs;
use sha2::{Digest, Sha256};

pub fn configure(cfg: &mut web::ServiceConfig) {
  cfg.service(web::scope("/api/v1/callbacks").service(handle));
}

#[post("/docusign")]
async fn handle(req: HttpRequest, body: String) -> impl Responder {
  enqueue(&req, &body).await;
  // DocuSign sends XML; we accept raw string for now.
  HttpResponse::Ok().json(serde_json::json!({"vendor":"docusign","received_bytes":body.len()}))
}

async fn enqueue(req: &HttpRequest, body: &str) {
  let pool = match req.app_data::<web::Data<sqlx::PgPool>>() {
    Some(p) => p.get_ref().clone(),
    None => return,
  };
  let key = format!("{:x}", Sha256::digest(body.as_bytes()));
  if let Ok(true) = jobs::claim_idempotency(&pool, "docusign", &key).await {
    let _ = jobs::enqueue(&pool, "webhook.docusign", serde_json::json!({"body_sha256": key}), Utc::now()).await;
  }
}

