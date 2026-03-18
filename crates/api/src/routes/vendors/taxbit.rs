use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use bytes::Bytes;
use chrono::Utc;
use goldfish_storage::jobs;
use sha2::{Digest, Sha256};

pub fn configure(cfg: &mut web::ServiceConfig) {
  cfg.service(web::scope("/api/v1/callbacks").service(webhook));
}

#[post("/taxbit")]
async fn webhook(req: HttpRequest, body: Bytes) -> impl Responder {
  enqueue(&req, "taxbit", &body).await;
  match serde_json::from_slice::<serde_json::Value>(&body) {
    Ok(v) => HttpResponse::Ok().json(serde_json::json!({"vendor":"taxbit","payload":v})),
    Err(_) => HttpResponse::BadRequest().finish(),
  }
}

async fn enqueue(req: &HttpRequest, scope: &str, body: &[u8]) {
  let pool = match req.app_data::<web::Data<sqlx::PgPool>>() {
    Some(p) => p.get_ref().clone(),
    None => return,
  };
  let key = format!("{:x}", Sha256::digest(body));
  if let Ok(true) = jobs::claim_idempotency(&pool, scope, &key).await {
    let _ = jobs::enqueue(&pool, "webhook.taxbit", serde_json::json!({"body_sha256": key}), Utc::now()).await;
  }
}

