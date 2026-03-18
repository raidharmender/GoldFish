use actix_web::{
  body::EitherBody,
  dev::{Service, ServiceRequest, ServiceResponse, Transform},
  Error, HttpMessage, HttpResponse,
};
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation};
use jsonwebtoken::jwk::JwkSet;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::future::{ready, Ready};
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::auth::actor::{Actor, ActorType};

#[derive(Debug, Clone)]
pub struct JwtAuth {
  jwks_url: String,
  issuer: Option<String>,
  audience: Option<String>,
  allowed_audiences: Option<Vec<String>>,
  actor_type: ActorType,
  require_exp: bool,
}

impl JwtAuth {
  pub fn new(jwks_url: String, actor_type: ActorType) -> Self {
    Self {
      jwks_url,
      issuer: None,
      audience: None,
      allowed_audiences: None,
      actor_type,
      require_exp: true,
    }
  }

  pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self {
    self.issuer = Some(issuer.into());
    self
  }

  pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
    self.audience = Some(audience.into());
    self
  }

  /// Mimics the Auth0 token processor behavior: allow any of these audiences (aud can be string or array).
  pub fn with_allowed_audiences(mut self, allowed: Vec<String>) -> Self {
    self.allowed_audiences = Some(allowed);
    self
  }

  /// Some flows skip strict exp/aud checks; use carefully.
  pub fn with_require_exp(mut self, require: bool) -> Self {
    self.require_exp = require;
    self
  }
}

#[derive(Debug, Deserialize)]
struct Claims {
  sub: String,
  exp: usize,
  #[serde(default)]
  iss: Option<String>,
  #[serde(default)]
  aud: Option<serde_json::Value>,
}

struct CachedJwks {
  fetched_at: Instant,
  jwks: Arc<JwkSet>,
}

static JWKS_CACHE: Lazy<RwLock<Option<CachedJwks>>> = Lazy::new(|| RwLock::new(None));

async fn get_jwks(jwks_url: &str) -> anyhow::Result<Arc<JwkSet>> {
  const TTL: Duration = Duration::from_secs(300);

  {
    let g = JWKS_CACHE.read().await;
    if let Some(c) = &*g {
      if c.fetched_at.elapsed() < TTL {
        return Ok(c.jwks.clone());
      }
    }
  }

  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(3))
    .build()?;

  let jwks: JwkSet = client.get(jwks_url).send().await?.error_for_status()?.json().await?;
  let jwks = Arc::new(jwks);
  let mut w = JWKS_CACHE.write().await;
  *w = Some(CachedJwks {
    fetched_at: Instant::now(),
    jwks: jwks.clone(),
  });
  Ok(jwks)
}

fn extract_bearer(req: &ServiceRequest) -> Option<String> {
  let v = req.headers().get("authorization")?.to_str().ok()?;
  let v = v.strip_prefix("Bearer ")?;
  Some(v.to_string())
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<EitherBody<B>>;
  type Error = Error;
  type InitError = ();
  type Transform = JwtAuthMiddleware<S>;
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ready(Ok(JwtAuthMiddleware {
      service: Rc::new(service),
      jwks_url: self.jwks_url.clone(),
      issuer: self.issuer.clone(),
      audience: self.audience.clone(),
      allowed_audiences: self.allowed_audiences.clone(),
      actor_type: self.actor_type.clone(),
      require_exp: self.require_exp,
    }))
  }
}

pub struct JwtAuthMiddleware<S> {
  service: Rc<S>,
  jwks_url: String,
  issuer: Option<String>,
  audience: Option<String>,
  allowed_audiences: Option<Vec<String>>,
  actor_type: ActorType,
  require_exp: bool,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<EitherBody<B>>;
  type Error = Error;
  type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

  fn poll_ready(
    &self,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Result<(), Self::Error>> {
    self.service.poll_ready(cx)
  }

  fn call(&self, req: ServiceRequest) -> Self::Future {
    let jwks_url = self.jwks_url.clone();
    let issuer = self.issuer.clone();
    let audience = self.audience.clone();
    let allowed_audiences = self.allowed_audiences.clone();
    let actor_type = self.actor_type.clone();
    let require_exp = self.require_exp;
    let svc = self.service.clone();

    Box::pin(async move {
      let token = match extract_bearer(&req) {
        Some(t) => t,
        None => {
          let res = req.into_response(HttpResponse::Unauthorized().finish().map_into_right_body());
          return Ok(res);
        }
      };

      let header = match decode_header(&token) {
        Ok(h) => h,
        Err(_) => {
          let res = req.into_response(HttpResponse::Unauthorized().finish().map_into_right_body());
          return Ok(res);
        }
      };

      let kid = match header.kid {
        Some(k) => k,
        None => {
          let res = req.into_response(HttpResponse::Unauthorized().finish().map_into_right_body());
          return Ok(res);
        }
      };

      let jwks = match get_jwks(&jwks_url).await {
        Ok(j) => j,
        Err(_) => {
          let res = req.into_response(HttpResponse::ServiceUnavailable().finish().map_into_right_body());
          return Ok(res);
        }
      };

      let jwk = match jwks.keys.iter().find(|k| k.common.key_id.as_deref() == Some(kid.as_str())) {
        Some(k) => k,
        None => {
          let res = req.into_response(HttpResponse::Unauthorized().finish().map_into_right_body());
          return Ok(res);
        }
      };

      let decoding_key = match DecodingKey::from_jwk(jwk) {
        Ok(k) => k,
        Err(_) => {
          let res = req.into_response(HttpResponse::Unauthorized().finish().map_into_right_body());
          return Ok(res);
        }
      };

      let mut validation = Validation::new(header.alg);
      validation.validate_exp = require_exp;
      if let Some(iss) = &issuer {
        validation.set_issuer(&[iss]);
      }
      if let Some(aud) = &audience {
        validation.set_audience(&[aud]);
      }

      let data = match decode::<Claims>(&token, &decoding_key, &validation) {
        Ok(d) => d,
        Err(_) => {
          let res = req.into_response(HttpResponse::Unauthorized().finish().map_into_right_body());
          return Ok(res);
        }
      };

      // allowed_audiences check (supports aud as string or array), like the Elixir token processors.
      if let Some(allowed) = &allowed_audiences {
        let ok = match data.claims.aud.clone() {
          Some(serde_json::Value::String(a)) => allowed.iter().any(|x| x == &a),
          Some(serde_json::Value::Array(arr)) => arr.iter().any(|v| {
            v.as_str()
              .is_some_and(|s| allowed.iter().any(|x| x == s))
          }),
          _ => false,
        };
        if !ok {
          let res = req.into_response(HttpResponse::Forbidden().finish().map_into_right_body());
          return Ok(res);
        }
      }

      req.extensions_mut().insert(Actor {
        actor_type,
        subject: data.claims.sub,
      });

      let res = svc.call(req).await?.map_into_left_body();
      Ok(res)
    })
  }
}

