use std::{
  error::Error,
  fmt::Debug,
  sync::Arc,
  time::{Duration, Instant},
};

use bytes::Bytes;
use reqwest::{
  StatusCode, Url,
  header::{self, HeaderValue},
};
use tokio::sync::Mutex;

/// A remote HTTP resource. Access to this value is cached, using both a TTL and
/// ETag headers.
pub struct CachedRequest<T, E: Error> {
  inner: CachedRequestInner<T, E>,
  last_response: Mutex<Option<CachedResponse<T>>>,
}

#[derive(Debug, thiserror::Error)]
pub enum CachedRequestError<E> {
  #[error("Request failed: {0}")]
  Request(#[from] reqwest::Error),
  #[error("Parsing failed: {0}")]
  Parse(E),
}

/// Internal state of our [CachedRequest] whose access is not locked and
/// immutable.
struct CachedRequestInner<T, E> {
  client: reqwest::Client,
  url: Url,
  ttl: Duration,
  parse: fn(bytes::Bytes) -> Result<T, E>,
}

struct CachedResponse<T> {
  value: Arc<T>,
  expiry: Instant,
  etag: Option<HeaderValue>,
}

impl<T, E: Error> CachedRequest<T, E> {
  /// Create a new [CachedRequest], which fetches the given `url` and
  /// parses it `parse`.
  pub fn new(
    client: reqwest::Client,
    url: Url,
    ttl: Duration,
    parse: fn(Bytes) -> Result<T, E>,
  ) -> Self {
    Self {
      inner: CachedRequestInner {
        client,
        url,
        ttl,
        parse,
      },
      last_response: Mutex::new(None),
    }
  }

  /// Get the current value of the cache.
  ///
  /// If an error occurs while fetching or parsing the remote resource, the cache
  /// will return the old value if present.
  #[tracing::instrument(skip_all, err, fields(url = %self.inner.url))]
  pub async fn get(&self) -> Result<Arc<T>, CachedRequestError<E>> {
    let last_response = &mut *self.last_response.lock().await;
    match self.inner.get(&mut *last_response).await {
      Ok(x) => Ok(x),
      Err(err) => {
        tracing::error!(?err, "Failed to fetch resource");
        match &*last_response {
          Some(x) => Ok(x.value.clone()),
          None => Err(err),
        }
      }
    }
  }
}

impl<T, E> CachedRequestInner<T, E> {
  async fn get(
    &self,
    last_response: &mut Option<CachedResponse<T>>,
  ) -> Result<Arc<T>, CachedRequestError<E>> {
    match &*last_response {
      Some(x) if x.expiry > Instant::now() => return Ok(x.value.clone()),
      _ => {}
    };

    let mut request = self.client.get(self.url.clone());
    if let Some(etag) = last_response.as_ref().and_then(|x| x.etag.as_ref()) {
      request = request.header(header::IF_NONE_MATCH, etag);
    }

    let response = request.send().await?;
    let etag = response.headers().get(header::ETAG).cloned();
    let value = match (response.status(), &*last_response) {
      (StatusCode::NOT_MODIFIED, Some(old)) => old.value.clone(),
      _ => {
        let body = response.error_for_status()?.bytes().await?;
        Arc::new((self.parse)(body).map_err(CachedRequestError::Parse)?)
      }
    };

    *last_response = Some(CachedResponse {
      value: value.clone(),
      expiry: Instant::now() + self.ttl,
      etag,
    });

    Ok(value)
  }
}
