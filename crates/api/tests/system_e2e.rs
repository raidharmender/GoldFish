use actix_web::{App, HttpServer, middleware as actix_mw};
use std::net::TcpListener;
use base64::Engine;

async fn start_public_server() -> anyhow::Result<(u16, actix_web::dev::ServerHandle)> {
  let listener = TcpListener::bind(("127.0.0.1", 0))?;
  let port = listener.local_addr()?.port();

  let server = HttpServer::new(move || {
    App::new()
      .wrap(actix_mw::Compress::default())
      .wrap(actix_mw::NormalizePath::trim())
      .wrap(goldfish_api::middleware::metrics::Metrics)
      .configure(goldfish_api::routes::public::configure)
      .configure(goldfish_api::openapi::configure)
  })
  .listen(listener)?
  .workers(1)
  .shutdown_timeout(1)
  .run();

  let handle = server.handle();
  actix_rt::spawn(server);
  Ok((port, handle))
}

async fn start_metrics_server() -> anyhow::Result<(u16, actix_web::dev::ServerHandle)> {
  let listener = TcpListener::bind(("127.0.0.1", 0))?;
  let port = listener.local_addr()?.port();

  let server = HttpServer::new(move || App::new().configure(goldfish_api::routes::metrics::configure))
    .listen(listener)?
    .workers(1)
    .shutdown_timeout(1)
    .run();

  let handle = server.handle();
  actix_rt::spawn(server);
  Ok((port, handle))
}

async fn start_openapi_server() -> anyhow::Result<(u16, actix_web::dev::ServerHandle)> {
  let listener = TcpListener::bind(("127.0.0.1", 0))?;
  let port = listener.local_addr()?.port();

  let server = HttpServer::new(move || {
    App::new()
      .wrap(actix_mw::NormalizePath::trim())
      .configure(goldfish_api::routes::openapi::configure)
  })
  .listen(listener)?
  .workers(1)
  .shutdown_timeout(1)
  .run();

  let handle = server.handle();
  actix_rt::spawn(server);
  Ok((port, handle))
}

async fn start_storm_server(api_key: String) -> anyhow::Result<(u16, actix_web::dev::ServerHandle)> {
  let listener = TcpListener::bind(("127.0.0.1", 0))?;
  let port = listener.local_addr()?.port();

  let server = HttpServer::new(move || {
    App::new().configure(|cfg| goldfish_api::routes::storm::configure_with_key(cfg, api_key.clone()))
  })
    .listen(listener)?
    .workers(1)
    .shutdown_timeout(1)
    .run();

  let handle = server.handle();
  actix_rt::spawn(server);
  Ok((port, handle))
}

#[actix_rt::test]
async fn system_health_and_openapi_and_metrics_work() -> anyhow::Result<()> {
  let (public_port, public_handle) = start_public_server().await?;
  let (openapi_port, openapi_handle) = start_openapi_server().await?;
  let (metrics_port, metrics_handle) = start_metrics_server().await?;
  let (storm_port, storm_handle) = start_storm_server("test-key".to_string()).await?;

  let client = reqwest::Client::builder()
    .redirect(reqwest::redirect::Policy::none())
    .build()?;

  let health_url = format!("http://127.0.0.1:{public_port}/api/v1/health");
  let resp = client.get(health_url).send().await?;
  assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);

  let spec_url = format!("http://127.0.0.1:{public_port}/api/spec");
  let resp = client.get(spec_url).send().await?;
  assert_eq!(resp.status(), reqwest::StatusCode::OK);

  let spec_url = format!("http://127.0.0.1:{openapi_port}/api/spec");
  let spec: serde_json::Value = client.get(spec_url).send().await?.json().await?;
  assert_eq!(spec["openapi"], "3.1.0");

  let swagger_url = format!("http://127.0.0.1:{public_port}/swaggerui/");
  let resp = client.get(swagger_url).send().await?;
  assert_eq!(resp.status(), reqwest::StatusCode::TEMPORARY_REDIRECT);

  let storm_url = format!("http://127.0.0.1:{storm_port}/storm/api/v1/status");
  let resp = client.get(storm_url).send().await?;
  assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);

  let storm_url = format!("http://127.0.0.1:{storm_port}/storm/api/v1/status");
  let resp = client.get(storm_url).header("x-api-key", "test-key").send().await?;
  assert_eq!(resp.status(), reqwest::StatusCode::OK);

  let metrics_url = format!("http://127.0.0.1:{metrics_port}/metrics");
  let resp = client.get(&metrics_url).send().await?;
  assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);

  let creds = base64::engine::general_purpose::STANDARD.encode("metrics:metrics");
  let resp = client
    .get(&metrics_url)
    .header("authorization", format!("Basic {creds}"))
    .send()
    .await?;
  assert_eq!(resp.status(), reqwest::StatusCode::OK);
  let text = resp.text().await?;
  assert!(text.contains("http_requests_total"));

  public_handle.stop(true).await;
  openapi_handle.stop(true).await;
  metrics_handle.stop(true).await;
  storm_handle.stop(true).await;
  Ok(())
}

// Job queue/idempotency are exercised in `goldfish-storage` tests (requires Postgres).

