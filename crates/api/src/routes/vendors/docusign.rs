use actix_web::{post, web, HttpResponse, Responder};

pub fn configure(cfg: &mut web::ServiceConfig) {
  cfg.service(web::scope("/api/v1/callbacks").service(handle));
}

#[post("/docusign")]
async fn handle(body: String) -> impl Responder {
  // DocuSign sends XML; we accept raw string for now.
  HttpResponse::Ok().json(serde_json::json!({"vendor":"docusign","received_bytes":body.len()}))
}

