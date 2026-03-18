use anyhow::Context;
use reqwest::Client;
use serde_json::Value;

mod given_when_then {
  use super::*;
  use actix_web::{App, HttpServer, middleware as actix_mw};
  use std::net::TcpListener;

  pub struct RunningServer {
    pub base_url: String,
    pub handle: actix_web::dev::ServerHandle,
  }

  pub async fn given_public_server_running() -> anyhow::Result<RunningServer> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let port = listener.local_addr()?.port();
    let base_url = format!("http://127.0.0.1:{port}");

    let server = HttpServer::new(move || {
      App::new()
        .wrap(actix_mw::NormalizePath::trim())
        .configure(goldfish_api::routes::public::configure)
        .configure(goldfish_api::openapi::configure)
    })
    .listen(listener)?
    .workers(1)
    .shutdown_timeout(1)
    .run();

    let handle = server.handle();
    actix_rt::spawn(server);
    Ok(RunningServer { base_url, handle })
  }

  pub async fn when_get_json(client: &Client, url: &str) -> anyhow::Result<Value> {
    client
      .get(url)
      .send()
      .await
      .context("send request")?
      .error_for_status()
      .context("status not success")?
      .json::<Value>()
      .await
      .context("decode json")
  }

  pub async fn when_get_expect_unauthorized(client: &Client, url: &str) -> anyhow::Result<()> {
    let resp = client.get(url).send().await.context("send request")?;
    if resp.status() != reqwest::StatusCode::UNAUTHORIZED {
      anyhow::bail!("expected 401, got {}", resp.status());
    }
    Ok(())
  }

  pub fn then_health_status_is_ok(v: &Value) {
    assert_eq!(v["status"], "ok");
    assert!(v.get("now_utc").is_some());
  }

  pub fn then_openapi_is_3_1(v: &Value) {
    assert_eq!(v["openapi"], "3.1.0");
  }
}

#[actix_rt::test]
async fn behaviour_public_api_health_and_openapi() -> anyhow::Result<()> {
  let server = given_when_then::given_public_server_running().await?;
  let client = Client::new();

  // Public API is JWT-protected; without a valid token we should get 401.
  given_when_then::when_get_expect_unauthorized(&client, &format!("{}/api/v1/health", server.base_url)).await?;

  let spec = given_when_then::when_get_json(&client, &format!("{}/api/spec", server.base_url)).await?;
  given_when_then::then_openapi_is_3_1(&spec);

  server.handle.stop(true).await;
  Ok(())
}

