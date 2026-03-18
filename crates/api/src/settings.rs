use serde::Deserialize;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
  pub public: ServerSettings,
  pub metrics: ServerSettings,
  pub openapi: ServerSettings,
  pub storm: ServerSettings,
  pub internal_api: ServerSettings,
  pub vendors: VendorPorts,
  pub sql: SqlSettings,
  pub redis: RedisSettings,
  pub storm_api_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerSettings {
  #[serde(default = "default_host")]
  pub host: String,
  pub port: u16,
  #[serde(default = "default_workers")]
  pub workers: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SqlSettings {
  pub database_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisSettings {
  pub redis_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VendorPorts {
  pub docusign: ServerSettings,
  pub bitgo: ServerSettings,
  pub cognito: ServerSettings,
  pub plaid: ServerSettings,
  pub modern_treasury: ServerSettings,
  pub taxbit: ServerSettings,
}

fn default_host() -> String {
  "0.0.0.0".to_string()
}

fn default_workers() -> usize {
  std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2)
}

impl Settings {
  pub fn load() -> anyhow::Result<Self> {
    let cfg = config::Config::builder()
      .set_default("public.port", 4000)?
      .set_default("metrics.port", 4001)?
      .set_default("storm.port", 4002)?
      .set_default("internal_api.port", 4004)?
      .set_default("openapi.port", 4107)?
      .set_default("vendors.docusign.port", 4101)?
      .set_default("vendors.bitgo.port", 4102)?
      .set_default("vendors.cognito.port", 4104)?
      .set_default("vendors.plaid.port", 4105)?
      .set_default("vendors.modern_treasury.port", 4106)?
      .set_default("vendors.taxbit.port", 4108)?
      .set_default("sql.database_url", "postgres://goldfish:goldfish@localhost:5432/goldfish")?
      .set_default("redis.redis_url", "redis://127.0.0.1:6379")?
      .add_source(config::Environment::with_prefix("GOLDFISH").separator("__"))
      .build()?;

    Ok(cfg.try_deserialize()?)
  }
}

pub fn init_tracing() {
  let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
  tracing_subscriber::fmt().with_env_filter(filter).json().init();
}

