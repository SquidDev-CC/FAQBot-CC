use serenity::{
  all::prelude::{Context, EventHandler},
  async_trait,
  builder::CreateInteractionResponse,
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
    tracing::info!(
      guilds = ready.guilds.len(),
      username = ready.user.name,
      "Bot is connected!"
    );

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
  }

  async fn message(&self, ctx: Context, msg: Message) {
    if let Some(content) = strip_bot_mention(&ctx, &msg.content) {
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
