use actix_web::{
  body::EitherBody,
  dev::{Service, ServiceRequest, ServiceResponse, Transform},
  Error, HttpResponse,
};
use std::future::{ready, Ready};
use std::pin::Pin;

#[derive(Clone)]
pub struct ApiKeyAuth {
  expected: String,
  header_name: &'static str,
}

impl ApiKeyAuth {
  pub fn new(expected: String) -> Self {
    Self {
      expected,
      header_name: "x-api-key",
    }
  }
}

impl<S, B> Transform<S, ServiceRequest> for ApiKeyAuth
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
  B: 'static,
{
  type Response = ServiceResponse<EitherBody<B>>;
  type Error = Error;
  type InitError = ();
  type Transform = ApiKeyAuthMiddleware<S>;
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ready(Ok(ApiKeyAuthMiddleware {
      service,
      expected: self.expected.clone(),
      header_name: self.header_name,
    }))
  }
}

pub struct ApiKeyAuthMiddleware<S> {
  service: S,
  expected: String,
  header_name: &'static str,
}

impl<S, B> Service<ServiceRequest> for ApiKeyAuthMiddleware<S>
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
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
    let expected = self.expected.clone();
    let header_name = self.header_name;
    let req = req;
    let mut authorized = false;

    if let Some(v) = req.headers().get(header_name) {
      if let Ok(s) = v.to_str() {
        authorized = s == expected;
      }
    }

    if !authorized {
      let res = req.into_response(HttpResponse::Unauthorized().finish().map_into_right_body());
      return Box::pin(async move { Ok(res) });
    }

    let fut = self.service.call(req);
    Box::pin(async move {
      let res = fut.await?.map_into_left_body();
      Ok(res)
    })
  }
}

