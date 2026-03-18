use once_cell::sync::Lazy;
use prometheus::{HistogramVec, IntCounterVec, register_histogram_vec, register_int_counter_vec};

pub static HTTP_REQUESTS_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
  register_int_counter_vec!(
    "http_requests_total",
    "Total number of HTTP requests",
    &["method", "route", "status"]
  )
  .expect("register http_requests_total")
});

pub static HTTP_REQUEST_DURATION_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
  register_histogram_vec!(
    "http_request_duration_seconds",
    "HTTP request duration in seconds",
    &["method", "route"],
    vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]
  )
  .expect("register http_request_duration_seconds")
});

