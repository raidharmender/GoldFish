use actix_web::{App, http::StatusCode, test};
use base64::Engine;

#[actix_rt::test]
async fn health_route_returns_ok_json() {
  let app = test::init_service(
    App::new()
      .configure(goldfish_api::routes::public::configure)
      .configure(goldfish_api::openapi::configure),
  )
  .await;

  let req = test::TestRequest::get().uri("/api/v1/health").to_request();
  let resp = test::call_service(&app, req).await;
  // Public API is now JWT-protected; without Authorization it should reject.
  assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_rt::test]
async fn openapi_spec_is_served() {
  let app = test::init_service(App::new().configure(goldfish_api::routes::openapi::configure)).await;

  let req = test::TestRequest::get().uri("/api/spec").to_request();
  let resp = test::call_service(&app, req).await;
  assert_eq!(resp.status(), StatusCode::OK);

  let body: serde_json::Value = test::read_body_json(resp).await;
  assert_eq!(body["openapi"], "3.1.0");
}

#[actix_rt::test]
async fn swaggerui_root_redirects_to_index() {
  let app = test::init_service(
    App::new()
      .wrap(actix_web::middleware::NormalizePath::trim())
      .configure(goldfish_api::routes::public::configure)
      .configure(goldfish_api::openapi::configure),
  )
  .await;

  let req = test::TestRequest::get().uri("/swaggerui/").to_request();
  let resp = test::call_service(&app, req).await;
  // swagger route is still mounted in this in-process app (back-compat), so it redirects.
  assert_eq!(resp.status(), StatusCode::TEMPORARY_REDIRECT);
}

#[actix_rt::test]
async fn metrics_endpoint_exposes_prometheus_text() {
  // Ensure metrics are registered before gather/encode.
  goldfish_api::metrics::HTTP_REQUESTS_TOTAL
    .with_label_values(&["GET", "/metrics", "200"])
    .inc();

  let app = test::init_service(App::new().configure(goldfish_api::routes::metrics::configure)).await;

  let req = test::TestRequest::get().uri("/metrics").to_request();
  let resp = test::call_service(&app, req).await;
  assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_rt::test]
async fn metrics_with_basic_auth_succeeds() {
  goldfish_api::metrics::HTTP_REQUESTS_TOTAL
    .with_label_values(&["GET", "/metrics", "200"])
    .inc();

  let app = test::init_service(App::new().configure(goldfish_api::routes::metrics::configure)).await;
  let creds = base64::engine::general_purpose::STANDARD.encode("metrics:metrics");
  let req = test::TestRequest::get()
    .uri("/metrics")
    .insert_header(("authorization", format!("Basic {creds}")))
    .to_request();
  let resp = test::call_service(&app, req).await;
  assert_eq!(resp.status(), StatusCode::OK);

  let body = test::read_body(resp).await;
  let body = String::from_utf8(body.to_vec()).expect("utf8 metrics");
  assert!(body.contains("# HELP "));
}

