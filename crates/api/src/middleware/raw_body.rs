/// Stores raw request bytes for webhook verification.
///
/// We'll attach these to per-handler verification to avoid fighting actix payload ownership.
#[derive(Debug, Clone)]
pub struct RawBodyBytes(pub actix_web::web::Bytes);

