use actix_web::{get, patch, put, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use crate::auth::actor::Actor;

pub fn configure(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::scope("")
      .service(show)
      .service(update)
      .service(patch_update),
  );
}

fn resolve_customer_id(req: &HttpRequest, customer_id: &str) -> Option<String> {
  if customer_id == "self" {
    let exts = req.extensions();
    let actor = exts.get::<Actor>()?;
    return Some(actor.subject.clone());
  }
  Some(customer_id.to_string())
}

#[get("/customers/{customer_id}")]
async fn show(req: HttpRequest, path: web::Path<String>) -> impl Responder {
  let customer_id = path.into_inner();
  let customer_id = match resolve_customer_id(&req, &customer_id) {
    Some(id) => id,
    None => return HttpResponse::Unauthorized().finish(),
  };

  let pool = match req.app_data::<web::Data<sqlx::PgPool>>() {
    Some(p) => p.get_ref().clone(),
    None => return HttpResponse::ServiceUnavailable().finish(),
  };

  match goldfish_storage::customers::get(&pool, &customer_id).await {
    Ok(Some(c)) => HttpResponse::Ok().json(serde_json::json!({"customer": {
      "id": c.id,
      "frozen": c.frozen,
      "created_at": c.created_at,
      "updated_at": c.updated_at,
      "profile": c.profile
    }})),
    Ok(None) => HttpResponse::NotFound().finish(),
    Err(_) => HttpResponse::InternalServerError().finish(),
  }
}

#[put("/customers/{customer_id}")]
async fn update(req: HttpRequest, path: web::Path<String>, body: web::Json<serde_json::Value>) -> impl Responder {
  update_impl(req, path, body).await
}

#[patch("/customers/{customer_id}")]
async fn patch_update(req: HttpRequest, path: web::Path<String>, body: web::Json<serde_json::Value>) -> impl Responder {
  update_impl(req, path, body).await
}

async fn update_impl(req: HttpRequest, path: web::Path<String>, body: web::Json<serde_json::Value>) -> HttpResponse {
  let customer_id = path.into_inner();
  let customer_id = match resolve_customer_id(&req, &customer_id) {
    Some(id) => id,
    None => return HttpResponse::Unauthorized().finish(),
  };

  let payload = body.into_inner();
  let patch = payload.get("customer").cloned().unwrap_or(payload);
  if !patch.is_object() {
    return HttpResponse::BadRequest().finish();
  }

  let pool = match req.app_data::<web::Data<sqlx::PgPool>>() {
    Some(p) => p.get_ref().clone(),
    None => return HttpResponse::ServiceUnavailable().finish(),
  };

  match goldfish_storage::customers::upsert_profile(&pool, &customer_id, patch).await {
    Ok(c) => HttpResponse::Ok().json(serde_json::json!({"customer": {
      "id": c.id,
      "frozen": c.frozen,
      "created_at": c.created_at,
      "updated_at": c.updated_at,
      "profile": c.profile
    }})),
    Err(_) => HttpResponse::InternalServerError().finish(),
  }
}

