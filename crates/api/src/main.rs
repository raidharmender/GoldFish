mod openapi;
mod routes;
mod settings;

use actix_web::{App, HttpServer, middleware};
use settings::Settings;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  settings::init_tracing();
  let settings = Settings::load()?;

  let public_addr = (settings.public.host.as_str(), settings.public.port);
  let metrics_addr = (settings.metrics.host.as_str(), settings.metrics.port);

  info!(?public_addr, "starting public api");
  let public = HttpServer::new(move || {
    App::new()
      .wrap(middleware::Compress::default())
      .wrap(middleware::NormalizePath::trim())
      .configure(routes::public::configure)
      .configure(openapi::configure)
  })
  .bind(public_addr)?
  .workers(settings.public.workers)
  .shutdown_timeout(5)
  .run();

  info!(?metrics_addr, "starting metrics endpoint");
  let metrics = HttpServer::new(move || App::new().configure(routes::metrics::configure))
    .bind(metrics_addr)?
    .workers(1)
    .shutdown_timeout(2)
    .run();

  tokio::select! {
    res = public => { res?; }
    res = metrics => { res?; }
    _ = tokio::signal::ctrl_c() => { info!("shutdown requested"); }
  };

  Ok(())
}
