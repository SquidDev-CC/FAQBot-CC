//! Provides a brief bit of information about the bot and its current state.

use crate::{InteractionError, State};
use serenity::{
  all::{CreateEmbed, prelude::Context},
  builder::{CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage},
  model::application::{CommandInteraction, CommandType},
};

pub const ID: &str = "ccfaq";
pub fn register() -> CreateCommand {
  CreateCommand::new(ID)
    .kind(CommandType::ChatInput)
    .description("Shows information about the bot as well as the relevant version numbers, uptime and useful links.")
}

pub async fn run(
  state: &State,
  ctx: Context,
  interaction: CommandInteraction,
) -> Result<(), InteractionError> {
  let mut reply = CreateEmbed::new().title("ComputerCraft FAQ Bot")
    .color(0x00e6e6)
    .url("https://github.com/SquidDev-CC/FAQBot-CC")
    .description("A Discord bot for answering frequently asked questions regarding CC. Please contribute and expand the list of answers on [GitHub](https://github.com/SquidDev-CC/FAQBot-CC)!")
    .field(":information_source: **Commands**", "Available commands: `/faq`, `/docs`, `/source`, `/eval`.", true)
    .field(":asterisk: **FAQs**", format!("Currently there are {} FAQs available.", crate::commands::faq::faqs().len()), true)
    .field(
      ":up: **Uptime information**",
      format!(
        "Bot started <t:{}>\nBot uptime: {}",
        state.start_time_since_epoch(),
        humantime::format_duration(state.start_time().elapsed())
      ), true
    );

  if let Some(avatar) = ctx.cache.current_user().avatar_url() {
    reply = reply.thumbnail(avatar)
  }

  interaction
    .create_response(
      &ctx,
      CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(reply)),
    )
    .await?;

  Ok(())
}
