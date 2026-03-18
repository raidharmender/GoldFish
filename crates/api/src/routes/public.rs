use actix_web::{get, web, HttpResponse, Responder};
use goldfish_core::health::HealthResponse;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(health), components(schemas(HealthResponse)), tags((name = "public")))]
pub struct PublicApiDoc;

pub fn configure(cfg: &mut web::ServiceConfig) {
  cfg.service(
    // Mount versioned scopes directly so `/api/spec` isn't shadowed by a broad `/api` scope.
    web::scope("/api/v1")
      .service(health)
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

