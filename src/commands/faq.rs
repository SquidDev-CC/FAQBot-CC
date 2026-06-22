//! Provides commands to read FAQs

use std::{collections::HashMap, sync::LazyLock};

use serenity::{
  all::{CreateEmbed, prelude::Context},
  builder::{
    CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage,
  },
  model::application::{
    CommandInteraction, CommandOptionType, CommandType, ResolvedOption, ResolvedValue,
  },
};

use crate::InteractionError;

pub struct Faq<'a> {
  /// The unique name/id of the FAQ. Equal to the filename minus extension.
  id: &'a str,
  /// The title of the FAQ.
  title: &'a str,
  /// The actual contents of the FAQ.
  contents: &'a str,
}

impl<'a> Faq<'a> {
  fn as_embed(&self) -> CreateEmbed {
    CreateEmbed::new()
      .title(self.title)
      .description(self.contents)
      .color(0x00e6e6)
  }
}

/// Get a list of all available FAQs.
pub fn faqs() -> &'static HashMap<&'static str, Faq<'static>> {
  static FAQS: LazyLock<HashMap<&str, Faq>> = LazyLock::new(|| {
    let files = include_dir::include_dir!("$CARGO_MANIFEST_DIR/faqs");
    files
      .files()
      .filter_map(|x| {
        let id = x.path().file_name()?.to_str()?.strip_suffix(".md")?;
        let (first, contents) = x
          .contents_utf8()
          .expect("Invalid UTF-8")
          .split_once('\n')
          .expect("Markdown file has no title");

        Some((
          id,
          Faq {
            id,
            title: first
              .strip_prefix('#')
              .expect("Failed to find title")
              .trim(),
            contents: contents.trim(),
          },
        ))
      })
      .collect()
  });

  &FAQS
}

pub const ID: &str = "faq";

pub fn register() -> CreateCommand {
  CreateCommand::new(ID)
    .kind(CommandType::ChatInput)
    .description("Find an FAQ.")
    .set_options(
      faqs()
        .values()
        .map(|faq| {
          CreateCommandOption::new(
            CommandOptionType::SubCommand,
            faq.id.replace('.', ""),
            faq.title,
          )
        })
        .collect(),
    )
}

pub async fn run(ctx: Context, interaction: CommandInteraction) -> Result<(), InteractionError> {
  let &[
    ResolvedOption {
      name: faq,
      value: ResolvedValue::SubCommand(_),
      ..
    },
  ] = interaction.data.options().as_slice()
  else {
    return Err(InteractionError::InvalidOptions);
  };

  let Some(faq) = faqs().get(faq) else {
    tracing::error!("No such FAQ");
    return Ok(());
  };

  interaction
    .create_response(
      &ctx,
      CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new().embed(faq.as_embed()),
      ),
    )
    .await?;
  Ok(())
}

#[cfg(test)]
mod test {
  use super::faqs;

  #[test]
  fn test_few_faqs() {
    assert!(faqs().len() <= 25, "Can only have 25 FAQs");
  }
}
