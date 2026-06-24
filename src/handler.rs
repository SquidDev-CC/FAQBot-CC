use serenity::{
  all::{
    Guild,
    prelude::{Context, EventHandler},
  },
  async_trait,
  builder::{CreateAllowedMentions, CreateInteractionResponse, CreateMessage},
  model::{
    application::{Command, CommandDataOptionValue, Interaction},
    channel::Message,
    gateway::Ready,
    id::GuildId,
  },
};
use tracing::Instrument;

use crate::{
  State,
  commands::{self, strip_bot_mention},
  config::Config,
};

pub struct Handler {
  state: State,
  guild_id: Option<GuildId>,
}

impl Handler {
  pub fn new(config: Config) -> Self {
    Self {
      state: State::new(config.eval_server),
      guild_id: config.guild_id,
    }
  }
}

#[async_trait]
impl EventHandler for Handler {
  async fn ready(&self, ctx: Context, ready: Ready) {
    tracing::info!("{} is connected!", ready.user.name);

    let commands = vec![
      commands::about::register(),
      commands::docs::register_docs(),
      commands::docs::register_source(),
      commands::eval::register_slash(),
      commands::eval::register_message(),
      commands::faq::register(),
    ];
    if let Some(guild_id) = self.guild_id {
      guild_id.set_commands(&ctx, commands).await
    } else {
      Command::set_global_commands(&ctx, commands).await
    }
    .expect("Failed to register commands");

    for guild in ready.guilds {
      if let Some(guild) = ctx.cache.guild(guild.id) {
        tracing::info!(guild = guild.name, "Connected to guild");
      } else if let Ok(guild) = ctx.http.get_guild(guild.id).await {
        tracing::info!(guild = guild.name, "Connected to guild");
      };
    }
  }

  async fn guild_create(&self, _ctx: Context, guild: Guild, _is_new: Option<bool>) {
    tracing::info!(guild = guild.name, "Connected to guild");
  }

  async fn message(&self, ctx: Context, msg: Message) {
    if msg.content.starts_with('%') {
      handle_legacy_command(&ctx, &msg).await;
    } else if let Some(content) = strip_bot_mention(&ctx, &msg.content) {
      let span = tracing::info_span!("Eval", user = msg.author.name);
      async {
        if let Err(err) = commands::eval::eval_from_ping(&self.state, &ctx, &msg, content).await {
          tracing::error!(%err, "Failed to eval code");
        }
      }
      .instrument(span)
      .await;
    }
  }

  async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    match interaction {
      Interaction::Command(command) => {
        let command_name = command.data.name.clone();
        let span = tracing::info_span!("Command", command = command_name, user = command.user.name);
        async move {
          tracing::info!("Received command");

          use commands;
          let result = match command_name.as_str() {
            commands::about::ID => commands::about::run(&self.state, ctx, command).await,
            commands::docs::DOCS_ID => commands::docs::run_docs(&self.state, ctx, command).await,
            commands::docs::SOURCE_ID => {
              commands::docs::run_source(&self.state, ctx, command).await
            }
            commands::eval::ID_SLASH => commands::eval::run_slash(&self.state, ctx, command).await,
            commands::eval::ID_MESSAGE => {
              commands::eval::run_message(&self.state, ctx, command).await
            }
            commands::faq::ID => commands::faq::run(ctx, command).await,
            _ => {
              return tracing::error!("Unknown command");
            }
          };

          if let Err(err) = result {
            tracing::error!(?err, "Failed to run command")
          }
        }
        .instrument(span)
        .await;
      }
      Interaction::Autocomplete(command) => {
        let command_name = command.data.name.as_str();
        let Some((arg, arg_value)) = command.data.options.iter().find_map(|x| match &x.value {
          CommandDataOptionValue::Autocomplete { kind: _, value } => {
            Some((x.name.as_str(), value.as_str()))
          }
          _ => None,
        }) else {
          return tracing::error!("Cannot find option to auto-complete");
        };

        let span = tracing::info_span!(
          "Autocomplete",
          command = command_name,
          arg,
          user = command.user.name
        );
        async {
          let result = match (command_name, arg) {
            (commands::docs::DOCS_ID | commands::docs::SOURCE_ID, commands::docs::METHOD_ARG) => {
              commands::docs::autocomplete(&self.state, arg_value).await
            }
            _ => return tracing::error!("Unknown option to auto-complete"),
          };

          if let Err(err) = command
            .create_response(&ctx, CreateInteractionResponse::Autocomplete(result))
            .await
          {
            tracing::error!(?err, "Failed to send autocomplete data")
          }
        }
        .instrument(span)
        .await;
      }
      Interaction::Component(component) => {
        let span = tracing::info_span!(
          "Component interaction",
          id = component.data.custom_id,
          user = component.user.name,
        );
        async {
          tracing::info!("Received component interaction");

          let result = match component.data.custom_id.as_str() {
            commands::eval::ON_RERUN => commands::eval::rerun(&self.state, &ctx, &component).await,
            commands::eval::ON_DELETE => commands::eval::delete(&ctx, &component).await,
            _ => {
              return tracing::error!("Unknown command");
            }
          };

          if let Err(err) = result {
            tracing::error!(%err, "Component interaction failed")
          }
        }
        .instrument(span)
        .await;
      }
      _ => (),
    }
  }
}

/// Handle a legacy `%` command, warning the user this is no longer supported.
async fn handle_legacy_command(ctx: &Context, msg: &Message) -> () {
  async fn report_command(ctx: &Context, msg: &Message, alternative: &str) {
    let reply  = CreateMessage::new()
        .content(format!(
            ":warning: % commands have been removed due to changes in bot verification. Please {alternative} instead. \
            See [this issue](<https://github.com/SquidDev-CC/FAQBot-CC/issues/65>) for more details."
        ))
        .allowed_mentions(CreateAllowedMentions::new().replied_user(false))
        .reference_message(msg);

    if let Err(err) = msg.channel_id.send_message(ctx, reply).await {
      tracing::error!(?err, "Error sending message")
    }
  }
  let first_word = match msg.content.split_once(' ') {
    None => &msg.content,
    Some((x, _)) => x,
  };

  match first_word {
    "%about" => report_command(ctx, msg, "use `/ccfaq`").await,
    "%docs" | "%d" => report_command(ctx, msg, "use `/docs`").await,
    "%source" | "%s" => report_command(ctx, msg, "use `/source`").await,
    "%faq" | "%f" | "%info" | "%i" => report_command(ctx, msg, "use `/faq`").await,
    "%eval" | "%exec" | "%code" => report_command(ctx, msg, "use `/eval` or tag the bot").await,
    _ => (),
  };
}
