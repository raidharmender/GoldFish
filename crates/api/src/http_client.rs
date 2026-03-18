use std::time::Duration;
use tracing::Instrument;

#[derive(Clone)]
pub struct HttpClient {
  inner: reqwest::Client,
}

impl HttpClient {
  pub fn new(timeout: Duration) -> anyhow::Result<Self> {
    let inner = reqwest::Client::builder().timeout(timeout).build()?;
    Ok(Self { inner })
  }

  pub async fn get_json<T: serde::de::DeserializeOwned>(
    &self,
    url: &str,
  ) -> anyhow::Result<T> {
    let span = tracing::info_span!("http.get", url = url);
    async move {
      let res = self.inner.get(url).send().await?.error_for_status()?;
      Ok(res.json::<T>().await?)
    }
    .instrument(span)
    .await
  }

  pub async fn post_json<Req: serde::Serialize, Resp: serde::de::DeserializeOwned>(
    &self,
    url: &str,
    body: &Req,
  ) -> anyhow::Result<Resp> {
    let span = tracing::info_span!("http.post", url = url);
    async move {
      let res = self
        .inner
        .post(url)
        .json(body)
        .send()
        .await?
        .error_for_status()?;
      Ok(res.json::<Resp>().await?)
    }
    .instrument(span)
    .await
  }
}

