use actix_web::{get, web, HttpResponse, Responder};

pub fn configure(cfg: &mut web::ServiceConfig) {
  cfg.service(web::scope("/storm").service(web::scope("/api/v1").service(status)));
  cfg.route("/storm/healthcheck", web::get().to(healthcheck));
}

#[get("/status")]
async fn status() -> impl Responder {
  HttpResponse::Ok().json(serde_json::json!({"status":"ok"}))
}

async fn healthcheck() -> impl Responder {
  HttpResponse::Ok().body("ok")
}

