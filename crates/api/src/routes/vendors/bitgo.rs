use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use bytes::Bytes;
use chrono::Utc;
use goldfish_storage::jobs;
use sha2::{Digest, Sha256};

pub fn configure(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::scope("/api/v1/callbacks")
      .service(web::scope("/{secret}").service(transfer).service(address_confirmation))
      .service(transfer_no_secret)
      .service(address_confirmation_no_secret),
  );
}

#[post("/transfer")]
async fn transfer(req: HttpRequest, path: web::Path<(String,)>, body: Bytes) -> impl Responder {
  let (secret,) = path.into_inner();
  enqueue(&req, "bitgo.transfer", &body).await;
  match serde_json::from_slice::<serde_json::Value>(&body) {
    Ok(v) => HttpResponse::Ok().json(serde_json::json!({"vendor":"bitgo","hook":"transfer","secret":secret,"payload":v})),
    Err(_) => HttpResponse::BadRequest().finish(),
  }
}

#[post("/address_confirmation")]
async fn address_confirmation(
  req: HttpRequest,
  path: web::Path<(String,)>,
  body: Bytes,
) -> impl Responder {
  let (secret,) = path.into_inner();
  enqueue(&req, "bitgo.address_confirmation", &body).await;
  match serde_json::from_slice::<serde_json::Value>(&body) {
    Ok(v) => HttpResponse::Ok().json(serde_json::json!({"vendor":"bitgo","hook":"address_confirmation","secret":secret,"payload":v})),
    Err(_) => HttpResponse::BadRequest().finish(),
  }
}

#[post("/transfer")]
async fn transfer_no_secret(req: HttpRequest, body: Bytes) -> impl Responder {
  enqueue(&req, "bitgo.transfer", &body).await;
  match serde_json::from_slice::<serde_json::Value>(&body) {
    Ok(v) => HttpResponse::Ok().json(serde_json::json!({"vendor":"bitgo","hook":"transfer","payload":v})),
    Err(_) => HttpResponse::BadRequest().finish(),
  }
}

#[post("/address_confirmation")]
async fn address_confirmation_no_secret(req: HttpRequest, body: Bytes) -> impl Responder {
  enqueue(&req, "bitgo.address_confirmation", &body).await;
  match serde_json::from_slice::<serde_json::Value>(&body) {
    Ok(v) => HttpResponse::Ok().json(serde_json::json!({"vendor":"bitgo","hook":"address_confirmation","payload":v})),
    Err(_) => HttpResponse::BadRequest().finish(),
  }
}

async fn enqueue(req: &HttpRequest, kind: &str, body: &[u8]) {
  let pool = match req.app_data::<web::Data<sqlx::PgPool>>() {
    Some(p) => p.get_ref().clone(),
    None => return,
  };
  let key = format!("{:x}", Sha256::digest(body));
  if let Ok(true) = jobs::claim_idempotency(&pool, "bitgo", &format!("{kind}:{key}")).await {
    let _ = jobs::enqueue(&pool, "webhook.bitgo", serde_json::json!({"kind": kind, "body_sha256": key}), Utc::now()).await;
  }
}

