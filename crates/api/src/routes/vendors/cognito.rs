use actix_web::{post, web, HttpResponse, Responder};

pub fn configure(cfg: &mut web::ServiceConfig) {
  cfg.service(web::scope("/").service(signup));
}

#[post("/signup")]
async fn signup(body: web::Json<serde_json::Value>) -> impl Responder {
  HttpResponse::Created().json(serde_json::json!({"status":"created","user":body.into_inner()}))
}

