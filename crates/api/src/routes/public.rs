use actix_web::{get, web, HttpResponse, Responder};
use goldfish_core::health::HealthResponse;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(health), components(schemas(HealthResponse)), tags((name = "public")))]
pub struct PublicApiDoc;

pub fn configure(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::scope("/api")
      .service(web::scope("/v1").service(health))
      .service(web::scope("/v1/webhooks")),
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

