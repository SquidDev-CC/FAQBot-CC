use anyhow::Context as _;
use serenity::{Client, all::GatewayIntents};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::handler::Handler;
pub use crate::state::State;

mod cached_request;
mod commands;
mod config;
mod handler;
mod lua_definitions;
mod state;

#[derive(Debug, thiserror::Error)]
pub enum InteractionError {
  #[error("Error in Discord API: {0}")]
  Serenity(#[from] serenity::Error),
  #[error("Invalid command options")]
  InvalidOptions,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Configure and enable tracing
  tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer())
    .with(
      EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy(),
    )
    .init();

  let config: config::Config =
    serde_json::from_reader(std::fs::File::open("config.json").context("Opening config file")?)
      .context("Parsing config file")?;

  let intents = GatewayIntents::GUILD_MESSAGES
    | GatewayIntents::DIRECT_MESSAGES
    | GatewayIntents::MESSAGE_CONTENT;

  // Start and run our Discord bot.
  Client::builder(&config.token, intents)
    .event_handler(Handler::new(config))
    .await
    .context("Creating Discord client")?
    .start()
    .await?;

  Ok(())
}
