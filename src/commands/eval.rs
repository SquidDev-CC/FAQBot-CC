use std::borrow::Cow;
use std::time::Duration;

use bytes::Bytes;
use serenity::{
  all::Context,
  builder::{
    CreateActionRow, CreateAllowedMentions, CreateAttachment, CreateButton, CreateCommand,
    CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage, CreateMessage, EditInteractionResponse,
  },
  model::{
    Permissions,
    application::{
      ButtonStyle, CommandInteraction, CommandOptionType, CommandType, ComponentInteraction,
      MessageInteractionMetadata, ResolvedOption, ResolvedTarget, ResolvedValue,
    },
    channel::Message,
  },
};

use crate::commands::eval::code_block::{CodeBlockResult, get_code_block};
use crate::commands::strip_bot_mention;
use crate::discord::MessageExt;
use crate::{InteractionError, State};

pub const ON_RERUN: &str = "on_rerun";
pub const ON_DELETE: &str = "on_delete";

/// Support for extracting code block(s) from a message.
mod code_block {
  use std::sync::LazyLock;

  use regex::Regex;

  #[derive(Debug, Clone, PartialEq, Eq)]
  pub enum CodeBlockResult<T> {
    /// No code blocks were found.
    None,
    /// Exactly one code block was found.
    One(T),
    /// Multiple code blocks were found.
    Amgibuous(T),
  }

  impl<T: ToOwned + ?Sized> CodeBlockResult<&T> {
    pub fn owned(&self) -> CodeBlockResult<T::Owned> {
      match self {
        CodeBlockResult::None => CodeBlockResult::None,
        CodeBlockResult::One(x) => CodeBlockResult::One((*x).to_owned()),
        CodeBlockResult::Amgibuous(x) => CodeBlockResult::Amgibuous((*x).to_owned()),
      }
    }
  }

  /// Extract all code blocks from the message.
  fn get_code_blocks(code: &str) -> Vec<&str> {
    static REGEX: LazyLock<Regex> = LazyLock::new(|| {
      regex::RegexBuilder::new(r"```(?:lua)?\n(.*?)```|`([^`]+)`")
        .dot_matches_new_line(true)
        .case_insensitive(true)
        .build()
        .unwrap()
    });

    REGEX
      .captures_iter(code)
      .filter_map(|capture| {
        capture
          .get(1)
          .or_else(|| capture.get(2))
          .map(|x| x.as_str())
      })
      .collect()
  }

  /// Extract a single code block from a message.
  pub fn get_code_block(content: &str) -> CodeBlockResult<&str> {
    match *get_code_blocks(content).as_slice() {
      [] => {
        let content = content.trim();
        if content.is_empty() {
          CodeBlockResult::None
        } else {
          CodeBlockResult::One(content)
        }
      }
      [code] => CodeBlockResult::One(code),
      [code, ..] => CodeBlockResult::Amgibuous(code),
    }
  }
}

enum EvalResult {
  Failure(&'static str),
  Success(String, Bytes),
}

/// Submit our code to be evaluated.
#[tracing::instrument(skip_all)]
async fn submit_code(state: &State, code: CodeBlockResult<String>, is_reply: bool) -> EvalResult {
  let (ambiguous, code) = match code {
    CodeBlockResult::None => {
      let err = if is_reply {
        "No code found in message! If trying to evaluate the original message, right click and select 'App' → 'CC: Aide → 'Run code'."
      } else {
        "No code found in message!"
      };
      return EvalResult::Failure(err);
    }
    CodeBlockResult::One(x) => (false, x),
    CodeBlockResult::Amgibuous(x) => (true, x),
  };

  let result = tokio::time::timeout(Duration::from_secs(20), async {
    let response = state
      .http_client()
      .post(state.eval_server().clone())
      .body(code)
      .send()
      .await?
      .error_for_status()?;

    let clean_exit = response
      .headers()
      .get("X-Clean-Exit")
      .is_some_and(|x| x == "True");

    let body = response.bytes().await?;
    Ok::<_, reqwest::Error>((clean_exit, body))
  })
  .await;

  match result {
    Err(err) => {
      tracing::error!(%err, "Timeout waiting for response");
      EvalResult::Failure("Timeout running code")
    }
    Ok(Err(err)) => {
      tracing::error!(%err, "HTTP error");
      EvalResult::Failure("Error running code")
    }
    Ok(Ok((clean_exit, screenshot))) => {
      let mut warnings = String::new();
      if ambiguous {
        warnings.push_str(":warning: Multiple code blocks, choosing the first.");
      }

      if !clean_exit {
        if !warnings.is_empty() {
          warnings.push('\n');
        }
        warnings.push_str(":warning: Computer ran for too long.")
      }

      EvalResult::Success(warnings, screenshot)
    }
  }
}

pub const ID_SLASH: &str = "eval";
pub fn register_slash() -> CreateCommand {
  CreateCommand::new(ID_SLASH)
    .kind(CommandType::ChatInput)
    .description("Evaluate a snippet of code.")
    .add_option(
      CreateCommandOption::new(CommandOptionType::String, "code", "The code to evaluate")
        .required(true),
    )
}

/// Evaluate a snippet of code.
pub async fn run_slash(
  state: &State,
  ctx: Context,
  interaction: CommandInteraction,
) -> Result<(), InteractionError> {
  let &[
    ResolvedOption {
      name: "code",
      value: ResolvedValue::String(code),
      ..
    },
  ] = interaction.data.options().as_slice()
  else {
    return Err(InteractionError::InvalidOptions);
  };

  interaction.defer(&ctx).await?;
  let response = match submit_code(state, CodeBlockResult::One(code.to_owned()), false).await {
    EvalResult::Failure(err) => {
      // Ideally we'd use ephemeral() here, but that doesn't work. See
      // https://github.com/SquidDev-CC/FAQBot-CC/issues/66
      CreateInteractionResponseFollowup::new().content(format!(":bangbang: {err}"))
    }
    EvalResult::Success(text, screenshot) => {
      // It's not possible to Rerun this code, as (AFAICT) there's no way to look
      // up the original code, so we just drop the buttons.
      CreateInteractionResponseFollowup::new()
        .content(text)
        .add_file(CreateAttachment::bytes(screenshot, "image.png"))
    }
  };
  interaction.create_followup(&ctx, response).await?;
  Ok(())
}

pub const ID_MESSAGE: &str = "Run in CC:T";

pub fn register_message() -> CreateCommand {
  CreateCommand::new(ID_MESSAGE).kind(CommandType::Message)
}

/// Evaluate the targeted message.
pub async fn run_message(
  state: &State,
  ctx: Context,
  interaction: CommandInteraction,
) -> Result<(), InteractionError> {
  let Some(ResolvedTarget::Message(msg)) = interaction.data.target() else {
    return Err(InteractionError::InvalidOptions);
  };

  interaction.defer(&ctx).await?;

  let code = code_block::get_code_block(&msg.content).owned();
  let response = match submit_code(state, code, false).await {
    EvalResult::Failure(err) => {
      // Ideally we'd use ephemeral() here, but that doesn't work. See
      // https://github.com/SquidDev-CC/FAQBot-CC/issues/66
      CreateInteractionResponseFollowup::new().content(format!(":bangbang: {err}"))
    }
    EvalResult::Success(text, screenshot) => CreateInteractionResponseFollowup::new()
      .content(text)
      .add_file(CreateAttachment::bytes(screenshot, "image.png"))
      .components(vec![CreateActionRow::Buttons(vec![
        CreateButton::new(ON_RERUN)
          .label("Rerun")
          .style(ButtonStyle::Primary),
        CreateButton::new(ON_DELETE)
          .label("Delete")
          .style(ButtonStyle::Danger)
          .emoji('🗑'),
      ])]),
  };
  interaction.create_followup(&ctx, response).await?;

  Ok(())
}

/// Evaluate the code within the message.
pub async fn eval_from_ping(
  state: &State,
  ctx: &Context,
  message: &Message,
  content: &str,
) -> Result<(), InteractionError> {
  let code = get_code_block(content).owned();
  match submit_code(state, code, message.message_reference.is_some()).await {
    EvalResult::Failure(err) => {
      message.reply(&ctx, format!(":bangbang: {err}")).await?;
      Ok(())
    }
    EvalResult::Success(text, screenshot) => {
      message
        .channel_id
        .send_files(
          &ctx,
          [CreateAttachment::bytes(screenshot, "image.png")],
          CreateMessage::new()
            .content(text)
            .reference_message(message)
            .allowed_mentions(CreateAllowedMentions::new().replied_user(false))
            .components(vec![CreateActionRow::Buttons(vec![
              CreateButton::new(ON_RERUN)
                .label("Rerun")
                .style(ButtonStyle::Primary),
              CreateButton::new(ON_DELETE)
                .label("Delete")
                .style(ButtonStyle::Danger)
                .emoji('🗑'),
            ])]),
        )
        .await?;
      Ok(())
    }
  }
}

/// Determine whether a user can interact with this evaluation result.
///
/// Moderators and the original author (either of the interaction or the target
/// message) can use these controls.
fn can_interact(interaction: &ComponentInteraction, original_message: Option<&Message>) -> bool {
  if interaction
    .member
    .as_ref()
    .and_then(|x| x.permissions)
    .is_some_and(|x| x.contains(Permissions::MANAGE_MESSAGES))
  {
    tracing::debug!("User has permissions to manage messages");
    return true;
  }

  // The user who triggered the original eval can rerun this.
  if let Some(kind) = &interaction.message.interaction_metadata {
    match kind.as_ref() {
      MessageInteractionMetadata::Command(x) if x.user == interaction.user => {
        tracing::debug!("User trigged the original command");
        return true;
      }
      _ => (),
    }
  }

  // If this is a message command, and then the author of the original message
  // can also run this.
  if original_message.is_some_and(|x| x.author == interaction.user) {
    tracing::debug!("User wrote the original message ");
    return true;
  }

  tracing::warn!(
    author = original_message.map(|x| &x.author.name),
    "User does not have permission"
  );
  false
}

/// Get the original message containing the code for this component interaction,
/// then check the user can actually perform the interaction.
async fn check_interaction<'a>(
  ctx: &'a Context,
  interaction: &'a ComponentInteraction,
) -> Result<Option<Cow<'a, Message>>, Result<(), InteractionError>> {
  let original_message = match interaction.message.find_reply(ctx).await {
    Ok(x) => x,
    Err(err) => {
      tracing::error!(?err, "Failed to find original message");
      None
    }
  };

  if !can_interact(interaction, original_message.as_deref()) {
    Err(
      interaction
        .create_response(
          &ctx,
          CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
              .content("Only the original commenter can do this. Sorry!")
              .ephemeral(true),
          ),
        )
        .await
        .map_err(Into::into),
    )
  } else {
    Ok(original_message)
  }
}

/// Rerun an evaluated bit of code. Triggered by the [ON_RERUN] action.
pub async fn rerun(
  state: &State,
  ctx: &Context,
  interaction: &ComponentInteraction,
) -> Result<(), InteractionError> {
  let message = match check_interaction(ctx, interaction).await {
    Ok(Some(x)) => x,
    Ok(None) => {
      return interaction
        .create_response(
          &ctx,
          CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
              .content(":bangbang: Cannot find the original message!")
              .ephemeral(true),
          ),
        )
        .await
        .map_err(Into::into);
    }
    Err(err) => return err,
  };

  interaction
    .create_response(&ctx, CreateInteractionResponse::Acknowledge)
    .await?;

  let message = strip_bot_mention(ctx, &message.content).unwrap_or(&message.content);
  let code = get_code_block(message).owned();

  match submit_code(state, code, false).await {
    EvalResult::Failure(err) => {
      interaction
        .create_followup(&ctx, CreateInteractionResponseFollowup::new().content(err))
        .await?;
      Ok(())
    }
    EvalResult::Success(text, screenshot) => {
      interaction
        .edit_response(
          &ctx,
          EditInteractionResponse::new()
            .content(text)
            .new_attachment(CreateAttachment::bytes(screenshot, "image.png")),
        )
        .await?;
      Ok(())
    }
  }
}

/// Delete the given message. Triggered by the [ON_DELETE] action.
pub async fn delete(
  ctx: &Context,
  interaction: &ComponentInteraction,
) -> Result<(), InteractionError> {
  if let Err(err) = check_interaction(ctx, interaction).await {
    return err;
  };

  interaction.message.delete(&ctx).await?;
  interaction
    .create_response(&ctx, CreateInteractionResponse::Acknowledge)
    .await?;
  Ok(())
}

#[cfg(test)]
mod test {
  #[test]
  fn test_get_code() {
    use super::code_block::{CodeBlockResult, get_code_block};

    assert_eq!(get_code_block(" "), CodeBlockResult::None);
    assert_eq!(get_code_block(" 1 + 2"), CodeBlockResult::One("1 + 2"));
    assert_eq!(
      get_code_block("```lua\nprint('hello')\n```"),
      CodeBlockResult::One("print('hello')\n")
    );
    assert_eq!(
      get_code_block("`print('hello')`"),
      CodeBlockResult::One("print('hello')")
    );
  }
}
