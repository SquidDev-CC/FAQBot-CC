use std::{
  collections::HashMap,
  sync::Arc,
  time::{Instant, SystemTime},
};

use crate::cached_request::{CachedRequest, CachedRequestError};
pub use doc_entries::DocumentedEntry;
use reqwest::Url;

pub struct State {
  start_time: Instant,
  start_time_since_epoch: u64,
  http_client: reqwest::Client,
  eval_server: Url,
  documentation_entries: CachedRequest<HashMap<String, DocumentedEntry>, serde_json::Error>,
}

impl State {
  pub fn new(eval_server: Url) -> Self {
    let http_client = reqwest::Client::new();
    Self {
      start_time: Instant::now(),
      start_time_since_epoch: SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time should be after epoch")
        .as_secs(),
      eval_server,
      documentation_entries: doc_entries::definitions_cache(http_client.clone()),
      http_client,
    }
  }
  /// The time the bot was started.
  pub fn start_time(&self) -> Instant {
    self.start_time
  }

  /// The time the bot was started, relative to the UNIX epoch.
  pub fn start_time_since_epoch(&self) -> u64 {
    self.start_time_since_epoch
  }

  /// The shared HTTP client.
  pub fn http_client(&self) -> &reqwest::Client {
    &self.http_client
  }

  /// The server to use for evaling code.
  pub fn eval_server(&self) -> &Url {
    &self.eval_server
  }

  /// Get a map of all entires in the documentation index.
  pub async fn documentation_entries(
    &self,
  ) -> Result<Arc<HashMap<String, DocumentedEntry>>, CachedRequestError<serde_json::Error>> {
    self.documentation_entries.get().await
  }
}

mod doc_entries {
  use std::{collections::HashMap, time::Duration};

  use bytes::Buf;

  use crate::{cached_request::CachedRequest, lua_definitions::LUA_DEFINITIONS};

  pub fn definitions_cache(
    client: reqwest::Client,
  ) -> CachedRequest<HashMap<String, DocumentedEntry>, serde_json::Error> {
    CachedRequest::new(
      client,
      "https://tweaked.cc/index.json".parse().unwrap(),
      Duration::from_mins(1),
      parse_illuaminate_index,
    )
  }

  fn parse_illuaminate_index(
    index: bytes::Bytes,
  ) -> Result<HashMap<String, DocumentedEntry>, serde_json::Error> {
    let index: HashMap<String, IlluaminateIndexEntry> = serde_json::from_reader(index.reader())?;
    let merged = LUA_DEFINITIONS
      .iter()
      .map(|(name, url)| (name.to_string(), DocumentedEntry::from_lua(name, url)))
      .chain(
        index
          .into_iter()
          .map(|(k, v)| (k, DocumentedEntry::from_illuaminate(v))),
      )
      .collect();
    Ok(merged)
  }

  #[derive(serde::Deserialize)]
  struct IlluaminateIndexEntry {
    name: String,
    source: String,
    summary: Option<String>,
    url: String,
  }

  pub struct DocumentedEntry {
    pub name: String,
    pub docs: String,
    pub source: Option<String>,
    pub summary: Option<String>,
  }

  impl DocumentedEntry {
    fn from_illuaminate(entry: IlluaminateIndexEntry) -> Self {
      Self {
        name: entry.name,
        source: Some(entry.source),
        docs: format!("https://tweaked.cc/{}", entry.url),
        summary: entry.summary,
      }
    }

    fn from_lua(name: &'static str, url: &'static str) -> Self {
      Self {
        name: name.to_string(),
        source: None,
        docs: url.to_owned(),
        summary: None,
      }
    }
  }
}
