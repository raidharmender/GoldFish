use actix_web::{post, web, HttpResponse, Responder};

pub fn configure(cfg: &mut web::ServiceConfig) {
  cfg.service(web::scope("/api/v1/callbacks").service(webhook));
}

#[post("/taxbit")]
async fn webhook(body: web::Json<serde_json::Value>) -> impl Responder {
  HttpResponse::Ok().json(serde_json::json!({"vendor":"taxbit","payload":body.into_inner()}))
}

