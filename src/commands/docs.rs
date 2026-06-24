//! Provides `/docs` and `/source` commands to search the CC:T and Lua docs fo
//! a particular method.

use std::{borrow::Borrow, collections::HashMap, hash::Hash};

use fuzzy_matcher::FuzzyMatcher;
use serenity::{
  all::prelude::Context,
  builder::{
    CreateAutocompleteResponse, CreateCommand, CreateCommandOption, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage,
  },
  model::application::{
    CommandInteraction, CommandOptionType, CommandType, ResolvedOption, ResolvedValue,
  },
};

use crate::{InteractionError, State, state::DocumentedEntry};

enum EmbedMode {
  Docs,
  Source,
}

async fn lookup(
  state: &State,
  name: &str,
  mode: EmbedMode,
) -> Result<CreateInteractionResponseMessage, InteractionError> {
  let name = name.trim_end_matches(['(', ')', ' ']);

  let methods = match state.documentation_entries().await {
    Ok(x) => x,
    Err(err) => {
      tracing::error!(?err, "Cannot get CC:T definitions");
      return Ok(
        CreateInteractionResponseMessage::new()
          .content("Failed to fetch CC:T definitions")
          .ephemeral(true),
      );
    }
  };

  match fuzzy_lookup(&methods, name) {
    FuzzyLookup::Exact(d) => Ok(CreateInteractionResponseMessage::new().embed(make_embed(d, mode))),
    FuzzyLookup::Fuzzy(d) => Ok(CreateInteractionResponseMessage::new()
        .content(format!("Cannot find '{}', using {}' instead", name, &d.name))
        .embed(make_embed(d, mode))
      ),
    FuzzyLookup::Missing => {
      Ok(CreateInteractionResponseMessage::new()
          .content(format!(
            "Cannot find '{name}'. Please check your spelling, or contribute to the documentation at https://github.com/cc-tweaked/CC-Tweaked."
          ))
        .ephemeral(true)
      )
    }
  }
}

async fn run(
  state: &State,
  ctx: Context,
  interaction: CommandInteraction,
  mode: EmbedMode,
) -> Result<(), InteractionError> {
  let &[
    ResolvedOption {
      name: "name",
      value: ResolvedValue::String(search),
      ..
    },
  ] = interaction.data.options().as_slice()
  else {
    return Err(InteractionError::InvalidOptions);
  };

  interaction
    .create_response(
      &ctx,
      CreateInteractionResponse::Message(lookup(state, search, mode).await?),
    )
    .await?;
  Ok(())
}

/// A basic auto-complete handler for a method name.
///
/// This just finds all strings prefixed with the current text, then scores
/// based on length.
pub async fn autocomplete(state: &State, search: &str) -> CreateAutocompleteResponse {
  let search = search.trim();
  let methods = match state.documentation_entries().await {
    Ok(x) => x,
    Err(err) => {
      tracing::error!(?err, "Cannot get CC:T definitions");
      return CreateAutocompleteResponse::new();
    }
  };

  let mut choices: Vec<_> = methods
    .keys()
    .filter(|x| x.len() >= search.len() && x[..search.len()].eq_ignore_ascii_case(search))
    .collect();
  choices.sort_by_key(|x| x.len());

  choices
    .into_iter()
    .take(25)
    .fold(CreateAutocompleteResponse::new(), |r, x| {
      r.add_string_choice(x, x)
    })
}

pub const DOCS_ID: &str = "docs";
pub const SOURCE_ID: &str = "source";
pub const METHOD_ARG: &str = "name";

pub fn register_docs() -> CreateCommand {
  CreateCommand::new(DOCS_ID)
    .kind(CommandType::ChatInput)
    .description("Searches for a function with the current name and returns its documentation.")
    .add_option(
      CreateCommandOption::new(CommandOptionType::String, METHOD_ARG, "The function's name")
        .required(true)
        .set_autocomplete(true),
    )
}

/// Searches for a function with the current name and returns its documentation.
pub async fn run_docs(
  state: &State,
  ctx: Context,
  interaction: CommandInteraction,
) -> Result<(), InteractionError> {
  run(state, ctx, interaction, EmbedMode::Docs).await
}

pub fn register_source() -> CreateCommand {
  CreateCommand::new(SOURCE_ID)
    .kind(CommandType::ChatInput)
    .description(
      "earches for a function with the current name and returns a link to the source code.",
    )
    .add_option(
      CreateCommandOption::new(CommandOptionType::String, METHOD_ARG, "The function's name")
        .required(true)
        .set_autocomplete(true),
    )
}

/// Searches for a function with the current name and returns a link to the source code.
pub async fn run_source(
  state: &State,
  ctx: Context,
  interaction: CommandInteraction,
) -> Result<(), InteractionError> {
  run(state, ctx, interaction, EmbedMode::Source).await
}

enum FuzzyLookup<T> {
  Missing,
  Fuzzy(T),
  Exact(T),
}

/// Look up an entry in a map, either returning an exact match ([FuzzyLookup::Exact]) or a
/// partial/fuzzy one ([FuzzyLookup::Fuzzy]).
fn fuzzy_lookup<'a, K, V>(map: &'a HashMap<K, V>, pattern: &'a str) -> FuzzyLookup<&'a V>
where
  K: Eq + Hash + Borrow<str>,
{
  if let Some(x) = map.get(pattern) {
    return FuzzyLookup::Exact(x);
  }

  let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
  let mut matches: Vec<_> = map
    .iter()
    .filter_map(|(k, v)| matcher.fuzzy_match(k.borrow(), pattern).map(|s| (v, s)))
    .collect();
  matches.sort_by_key(|(_, x)| *x);

  match *matches.as_slice() {
    [(x, _)] => FuzzyLookup::Fuzzy(x),
    [.., (x, x_score), (_, y_score)] if x_score > y_score + 5 => FuzzyLookup::Fuzzy(x),
    _ => FuzzyLookup::Missing,
  }
}

fn make_embed(entry: &DocumentedEntry, mode: EmbedMode) -> CreateEmbed {
  let mut embed = CreateEmbed::new()
    .title(entry.name.as_str())
    .url(match mode {
      EmbedMode::Docs => &entry.docs,
      EmbedMode::Source => entry.source.as_ref().unwrap_or(&entry.docs),
    });

  if let Some(summary) = entry.summary.as_ref() {
    embed = embed.description(summary);
  }

  embed
}
