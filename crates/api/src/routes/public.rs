use actix_web::{get, web, HttpResponse, Responder};
use goldfish_core::health::HealthResponse;
use utoipa::OpenApi;
use crate::auth::jwt::JwtAuth;
use crate::auth::actor::ActorType;
use crate::routes::customers;

#[derive(OpenApi)]
#[openapi(paths(health), components(schemas(HealthResponse)), tags((name = "public")))]
pub struct PublicApiDoc;

pub fn configure(cfg: &mut web::ServiceConfig) {
  // Match reference behavior: configure allowed audiences per surface (Auth0-style).
  let jwks_url = std::env::var("GOLDFISH__JWKS_URL")
    .unwrap_or_else(|_| "http://localhost:9999/.well-known/jwks.json".to_string());
  let iss = std::env::var("GOLDFISH__AUTH0_ISS").ok();
  let allowed_auds = std::env::var("GOLDFISH__ALLOWED_AUDS")
    .ok()
    .map(|s| s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect::<Vec<_>>());

  let mut jwt = JwtAuth::new(jwks_url, ActorType::Customer).with_require_exp(true);
  if let Some(iss) = iss {
    jwt = jwt.with_issuer(iss);
  }
  if let Some(auds) = allowed_auds {
    jwt = jwt.with_allowed_audiences(auds);
  }

  cfg.service(
    // Mount versioned scopes directly so `/api/spec` isn't shadowed by a broad `/api` scope.
    web::scope("/api/v1")
      // In the reference app, most /api/v1 routes use customer_api pipeline. We model it here.
      .wrap(jwt)
      .service(health)
      .configure(customers::configure)
      .service(web::scope("/webhooks")),
  );
}

#[utoipa::path(
  get,
  path = "/api/v1/health",
  tag = "public",
  responses(
    (status = 200, description = "Service health", body = HealthResponse)
  )
)]
#[get("/health")]
async fn health() -> impl Responder {
  HttpResponse::Ok().json(HealthResponse::ok())
}

