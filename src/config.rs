use reqwest::Url;
use serenity::all::GuildId;

#[derive(serde::Deserialize)]
pub struct Config {
  /// Token to connect to Discord with.
  pub token: String,

  /// Restricted guild id this bot registers commands under.
  pub guild_id: Option<GuildId>,

  /// The server to use for evaling code. By default eval.tweaked.cc, but may be a custom server.
  #[serde(default = "default_eval_server")]
  pub eval_server: Url,
}

fn default_eval_server() -> Url {
  Url::parse("https://eval.tweaked.cc").expect("Can deserialise constant URL")
}
