//! Additional helpers for working with Discord.

use std::borrow::Cow;

use serenity::all::{Context, Error, Message, MessageReference};

pub trait MessageExt {
  /// Get the reply of this message, looking it up in the cache or via the API if required.
  async fn find_reply<'a>(&'a self, ctx: &'a Context) -> Result<Option<Cow<'a, Message>>, Error>;
}

impl MessageExt for Message {
  async fn find_reply<'a>(&'a self, ctx: &'a Context) -> Result<Option<Cow<'a, Message>>, Error> {
    if let Some(reply_to) = &self.referenced_message {
      // If we're a reply and we have a referenced_message, use that.
      Ok(Some(Cow::Borrowed(reply_to.as_ref())))
    } else if let Some(MessageReference {
      channel_id,
      message_id: Some(message_id),
      ..
    }) = &self.message_reference
    {
      // If we're a reply (and don't have a referenced message), look the message up.
      if let Some(message) = ctx.cache.message(channel_id, message_id) {
        // It would be nice to avoid the the clone() here. However, we can't return the CacheRef, as that's not Send
        // (and we unfortunately need it to be in our async code).
        Ok(Some(Cow::Owned(message.clone())))
      } else {
        Ok(Some(Cow::Owned(
          ctx.http.get_message(*channel_id, *message_id).await?,
        )))
      }
    } else {
      // Otherwise we can't find the message content.
      Ok(None)
    }
  }
}
