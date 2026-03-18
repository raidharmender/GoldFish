use actix_web::{App, HttpServer, middleware as actix_mw};
use goldfish_api::{routes, settings, middleware, settings::Settings};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  settings::init_tracing();
  let settings = Settings::load()?;
  let _scheduler = goldfish_api::jobs::scheduler::start().await?;

  let public_addr = (settings.public.host.as_str(), settings.public.port);
  let metrics_addr = (settings.metrics.host.as_str(), settings.metrics.port);
  let openapi_addr = (settings.openapi.host.as_str(), settings.openapi.port);
  let storm_addr = (settings.storm.host.as_str(), settings.storm.port);
  let internal_addr = (settings.internal_api.host.as_str(), settings.internal_api.port);
  let docusign_addr = (
    settings.vendors.docusign.host.as_str(),
    settings.vendors.docusign.port,
  );
  let bitgo_addr = (settings.vendors.bitgo.host.as_str(), settings.vendors.bitgo.port);
  let cognito_addr = (
    settings.vendors.cognito.host.as_str(),
    settings.vendors.cognito.port,
  );
  let plaid_addr = (settings.vendors.plaid.host.as_str(), settings.vendors.plaid.port);
  let mt_addr = (
    settings.vendors.modern_treasury.host.as_str(),
    settings.vendors.modern_treasury.port,
  );
  let taxbit_addr = (settings.vendors.taxbit.host.as_str(), settings.vendors.taxbit.port);

  info!(?public_addr, "starting public api");
  let public = HttpServer::new(move || {
    App::new()
      .wrap(actix_mw::Compress::default())
      .wrap(actix_mw::NormalizePath::trim())
      .wrap(middleware::metrics::Metrics)
      .configure(routes::public::configure)
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

  info!(?openapi_addr, "starting openapi/swagger endpoint");
  let openapi_server = HttpServer::new(move || {
    App::new()
      .wrap(actix_mw::Compress::default())
      .wrap(actix_mw::NormalizePath::trim())
      .configure(routes::openapi::configure)
  })
  .bind(openapi_addr)?
  .workers(1)
  .shutdown_timeout(2)
  .run();

  info!(?storm_addr, "starting storm endpoint");
  let storm = HttpServer::new(move || {
    App::new()
      .wrap(actix_mw::Compress::default())
      .wrap(actix_mw::NormalizePath::trim())
      .configure(routes::storm::configure)
  })
  .bind(storm_addr)?
  .workers(1)
  .shutdown_timeout(2)
  .run();

  info!(?internal_addr, "starting internal api endpoint");
  let internal_api = HttpServer::new(move || {
    App::new()
      .wrap(actix_mw::Compress::default())
      .wrap(actix_mw::NormalizePath::trim())
      .configure(routes::internal_api::configure)
  })
  .bind(internal_addr)?
  .workers(1)
  .shutdown_timeout(2)
  .run();

  info!(?docusign_addr, "starting docusign callbacks endpoint");
  let docusign = HttpServer::new(move || {
    App::new()
      .wrap(actix_mw::NormalizePath::trim())
      .configure(routes::vendors::docusign::configure)
  })
  .bind(docusign_addr)?
  .workers(1)
  .shutdown_timeout(2)
  .run();

  info!(?bitgo_addr, "starting bitgo callbacks endpoint");
  let bitgo = HttpServer::new(move || {
    App::new()
      .wrap(actix_mw::NormalizePath::trim())
      .configure(routes::vendors::bitgo::configure)
  })
  .bind(bitgo_addr)?
  .workers(1)
  .shutdown_timeout(2)
  .run();

  info!(?cognito_addr, "starting cognito endpoint");
  let cognito = HttpServer::new(move || {
    App::new()
      .wrap(actix_mw::NormalizePath::trim())
      .configure(routes::vendors::cognito::configure)
  })
  .bind(cognito_addr)?
  .workers(1)
  .shutdown_timeout(2)
  .run();

  info!(?plaid_addr, "starting plaid callbacks endpoint");
  let plaid = HttpServer::new(move || {
    App::new()
      .wrap(actix_mw::NormalizePath::trim())
      .configure(routes::vendors::plaid::configure)
  })
  .bind(plaid_addr)?
  .workers(1)
  .shutdown_timeout(2)
  .run();

  info!(?mt_addr, "starting modern treasury callbacks endpoint");
  let modern_treasury = HttpServer::new(move || {
    App::new()
      .wrap(actix_mw::NormalizePath::trim())
      .configure(routes::vendors::modern_treasury::configure)
  })
  .bind(mt_addr)?
  .workers(1)
  .shutdown_timeout(2)
  .run();

  info!(?taxbit_addr, "starting taxbit callbacks endpoint");
  let taxbit = HttpServer::new(move || {
    App::new()
      .wrap(actix_mw::NormalizePath::trim())
      .configure(routes::vendors::taxbit::configure)
  })
  .bind(taxbit_addr)?
  .workers(1)
  .shutdown_timeout(2)
  .run();

  tokio::select! {
    res = public => { res?; }
    res = metrics => { res?; }
    res = openapi_server => { res?; }
    res = storm => { res?; }
    res = internal_api => { res?; }
    res = docusign => { res?; }
    res = bitgo => { res?; }
    res = cognito => { res?; }
    res = plaid => { res?; }
    res = modern_treasury => { res?; }
    res = taxbit => { res?; }
    _ = tokio::signal::ctrl_c() => { info!("shutdown requested"); }
  };

  Ok(())
}
