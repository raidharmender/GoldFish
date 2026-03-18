use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use std::future::{ready, Ready};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

pub struct Metrics;

impl<S, B> Transform<S, ServiceRequest> for Metrics
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<B>;
  type Error = Error;
  type InitError = ();
  type Transform = MetricsMiddleware<S>;
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ready(Ok(MetricsMiddleware { service }))
  }
}

pub struct MetricsMiddleware<S> {
  service: S,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddleware<S>
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<B>;
  type Error = Error;
  type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

  fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    self.service.poll_ready(cx)
  }

  fn call(&self, req: ServiceRequest) -> Self::Future {
    let method = req.method().to_string();
    let route = req
      .match_pattern()
      .map(|s| s.to_string())
      .unwrap_or_else(|| req.path().to_string());
    let start = Instant::now();

    let fut = self.service.call(req);
    Box::pin(async move {
      let res = fut.await?;
      let status = res.status().as_u16().to_string();
      let elapsed = start.elapsed().as_secs_f64();

      crate::metrics::HTTP_REQUESTS_TOTAL
        .with_label_values(&[&method, &route, &status])
        .inc();
      crate::metrics::HTTP_REQUEST_DURATION_SECONDS
        .with_label_values(&[&method, &route])
        .observe(elapsed);

      Ok(res)
    })
  }
}

