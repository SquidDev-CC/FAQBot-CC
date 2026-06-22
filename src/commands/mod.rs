use serenity::all::prelude::Context;

pub mod about;
pub mod docs;
pub mod eval;
pub mod faq;

/// Strip a mention of the bot from the start of the message (`<@USER_ID>` or `<@!USER_ID>`)
pub fn strip_bot_mention<'a>(ctx: &Context, content: &'a str) -> Option<&'a str> {
  content
    .strip_prefix("<@")
    .map(|x| x.trim_start_matches('!'))
    .and_then(|x| x.strip_prefix(&ctx.cache.current_user().id.to_string()))
    .and_then(|x| x.strip_prefix('>'))
    .map(|x| x.trim_start())
}
