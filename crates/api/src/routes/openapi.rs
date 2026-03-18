use actix_web::{web, HttpResponse, Responder};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::routes::public::PublicApiDoc;
use utoipa::openapi::OpenApi as OpenApiModel;

#[derive(OpenApi)]
#[openapi(tags((name = "goldfish")))]
struct ApiDoc;

pub fn configure(cfg: &mut web::ServiceConfig) {
  // Merge docs from different surfaces as the app grows.
  let mut openapi = ApiDoc::openapi();
  openapi.paths.paths.extend(PublicApiDoc::openapi().paths.paths);
  openapi.components = PublicApiDoc::openapi().components;

  let openapi_data = web::Data::new(openapi);

  cfg.service(SwaggerUi::new("/swaggerui/{tail:.*}").url(
    "/api/spec",
    openapi_data.get_ref().clone(),
  ));

  cfg.route(
    "/swaggerui",
    web::get().to(|| async {
      HttpResponse::TemporaryRedirect()
        .append_header(("Location", "/swaggerui/index.html"))
        .finish()
    }),
  );

  cfg.app_data(openapi_data.clone());
  cfg.route("/api/spec", web::get().to(api_spec));
}

async fn api_spec(openapi: web::Data<OpenApiModel>) -> impl Responder {
  HttpResponse::Ok().json(openapi.get_ref())
}

