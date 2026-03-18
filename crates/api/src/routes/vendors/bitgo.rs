use actix_web::{post, web, HttpResponse, Responder};

pub fn configure(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::scope("/api/v1/callbacks")
      .service(web::scope("/{secret}").service(transfer).service(address_confirmation))
      .service(transfer_no_secret)
      .service(address_confirmation_no_secret),
  );
}

#[post("/transfer")]
async fn transfer(path: web::Path<(String,)>, body: web::Json<serde_json::Value>) -> impl Responder {
  let (secret,) = path.into_inner();
  HttpResponse::Ok().json(serde_json::json!({"vendor":"bitgo","hook":"transfer","secret":secret,"payload":body.into_inner()}))
}

#[post("/address_confirmation")]
async fn address_confirmation(
  path: web::Path<(String,)>,
  body: web::Json<serde_json::Value>,
) -> impl Responder {
  let (secret,) = path.into_inner();
  HttpResponse::Ok().json(serde_json::json!({"vendor":"bitgo","hook":"address_confirmation","secret":secret,"payload":body.into_inner()}))
}

#[post("/transfer")]
async fn transfer_no_secret(body: web::Json<serde_json::Value>) -> impl Responder {
  HttpResponse::Ok().json(serde_json::json!({"vendor":"bitgo","hook":"transfer","payload":body.into_inner()}))
}

#[post("/address_confirmation")]
async fn address_confirmation_no_secret(body: web::Json<serde_json::Value>) -> impl Responder {
  HttpResponse::Ok().json(serde_json::json!({"vendor":"bitgo","hook":"address_confirmation","payload":body.into_inner()}))
}

