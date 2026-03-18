use anyhow::Context;
use jsonwebtoken::{decode, decode_header, jwk::Jwk, DecodingKey, Validation};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize)]
struct PlaidClaims {
  iat: i64,
  request_body_sha256: String,
}

pub async fn verify_plaid_webhook(
  raw_body: &[u8],
  jwt: &str,
  jwk_fetch_url: &str,
  client_id: &str,
  secret: &str,
) -> anyhow::Result<()> {
  let header = decode_header(jwt).context("decode jwt header")?;
  let kid = header.kid.context("missing kid")?;

  // Plaid uses ES256; enforce what the reference plug expects.
  if header.alg != jsonwebtoken::Algorithm::ES256 {
    anyhow::bail!("unexpected alg");
  }

  #[derive(Deserialize)]
  struct JwkResp {
    key: Jwk,
  }

  let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(5))
    .build()?;

  let resp: JwkResp = client
    .post(jwk_fetch_url)
    .json(&serde_json::json!({
      "client_id": client_id,
      "secret": secret,
      "key_id": kid
    }))
    .send()
    .await?
    .error_for_status()?
    .json()
    .await?;

  let decoding_key = DecodingKey::from_jwk(&resp.key)?;
  let mut validation = Validation::new(header.alg);
  validation.validate_exp = false; // Plaid uses iat freshness check in reference.

  let data = decode::<PlaidClaims>(jwt, &decoding_key, &validation).context("verify jwt")?;

  // Freshness: iat within last 5 minutes.
  let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
  if data.claims.iat <= now - 5 * 60 {
    anyhow::bail!("stale iat");
  }

  let hash = Sha256::digest(raw_body);
  let got = format!("{:x}", hash);
  if got != data.claims.request_body_sha256.to_lowercase() {
    anyhow::bail!("invalid body hash");
  }

  Ok(())
}

