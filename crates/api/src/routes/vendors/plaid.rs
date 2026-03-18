use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use bytes::Bytes;
use crate::auth::plaid;
use sha2::{Digest, Sha256};
use chrono::Utc;
use goldfish_storage::jobs;

pub fn configure(cfg: &mut web::ServiceConfig) {
  cfg.service(web::scope("/api/v1/callbacks").service(webhook));
}

#[post("/plaid")]
async fn webhook(req: HttpRequest, body: Bytes) -> impl Responder {
  let omit = std::env::var("GOLDFISH__PLAID_OMIT_JWT_VERIFICATION")
    .ok()
    .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));

  if !omit {
    let jwt = match req.headers().get("plaid-verification").and_then(|v| v.to_str().ok()) {
      Some(v) => v,
      None => return HttpResponse::Unauthorized().finish(),
    };

    let jwk_fetch_url =
      std::env::var("GOLDFISH__PLAID_WEBHOOKS_JWK_ADDR").unwrap_or_else(|_| "http://localhost:9999/plaid/jwk".to_string());
    let client_id = std::env::var("GOLDFISH__PLAID_CLIENT_ID").unwrap_or_default();
    let secret = std::env::var("GOLDFISH__PLAID_SECRET").unwrap_or_default();

    if let Err(_e) = plaid::verify_plaid_webhook(
      &body,
      jwt,
      &jwk_fetch_url,
      &client_id,
      &secret,
    )
    .await
    {
      return HttpResponse::Unauthorized().finish();
    }
  }

  // Idempotency + enqueue job
  let key = format!("{:x}", Sha256::digest(&body));
  let pool = match req.app_data::<web::Data<sqlx::PgPool>>() {
    Some(p) => p.get_ref().clone(),
    None => return HttpResponse::ServiceUnavailable().finish(),
  };

  let first = match jobs::claim_idempotency(&pool, "plaid", &key).await {
    Ok(v) => v,
    Err(_) => return HttpResponse::ServiceUnavailable().finish(),
  };

  if first {
    let _ = jobs::enqueue(
      &pool,
      "webhook.plaid",
      serde_json::json!({"body_sha256": key}),
      Utc::now(),
    )
    .await;
  }

  // Body is JSON per Plaid. For now we just echo it back after verification.
  match serde_json::from_slice::<serde_json::Value>(&body) {
    Ok(v) => HttpResponse::Ok().json(serde_json::json!({"vendor":"plaid","payload":v})),
    Err(_) => HttpResponse::BadRequest().finish(),
  }
}

