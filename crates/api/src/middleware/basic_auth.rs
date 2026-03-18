use actix_web::{
  body::EitherBody,
  dev::{Service, ServiceRequest, ServiceResponse, Transform},
  Error, HttpResponse,
};
use base64::Engine;
use std::future::{ready, Ready};
use std::pin::Pin;

#[derive(Clone)]
pub struct BasicAuth {
  username: String,
  password: String,
}

impl BasicAuth {
  pub fn new(username: String, password: String) -> Self {
    Self { username, password }
  }
}

impl<S, B> Transform<S, ServiceRequest> for BasicAuth
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<EitherBody<B>>;
  type Error = Error;
  type InitError = ();
  type Transform = BasicAuthMw<S>;
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ready(Ok(BasicAuthMw {
      service,
      username: self.username.clone(),
      password: self.password.clone(),
    }))
  }
}

pub struct BasicAuthMw<S> {
  service: S,
  username: String,
  password: String,
}

impl<S, B> Service<ServiceRequest> for BasicAuthMw<S>
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
    let expected = format!("{}:{}", self.username, self.password);
    let ok = req
      .headers()
      .get("authorization")
      .and_then(|v| v.to_str().ok())
      .and_then(|s| s.strip_prefix("Basic "))
      .and_then(|b64| base64::engine::general_purpose::STANDARD.decode(b64).ok())
      .and_then(|bytes| String::from_utf8(bytes).ok())
      .is_some_and(|creds| creds == expected);

    if !ok {
      let res = req.into_response(
        HttpResponse::Unauthorized()
          .append_header(("WWW-Authenticate", "Basic realm=\"metrics\""))
          .finish()
          .map_into_right_body(),
      );
      return Box::pin(async move { Ok(res) });
    }

    let fut = self.service.call(req);
    Box::pin(async move {
      Ok(fut.await?.map_into_left_body())
    })
  }
}

