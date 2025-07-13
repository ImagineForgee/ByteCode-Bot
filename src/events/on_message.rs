use poise::serenity_prelude::Context;
use poise::FrameworkContext;
use serenity::all::FullEvent;
use bot_macros::register_event;

use crate::{Data, FullError};
use crate::utils::translate::translate_if_needed;

#[register_event]
async fn on_message(
    ctx: &Context,
    event: &FullEvent,
    _framework: FrameworkContext<'_, Data, FullError>,
    data: &Data,
) -> Result<(), FullError> {
    if let FullEvent::Message { new_message } = event {
        if new_message.author.bot {
            return Ok(());
        }

        let content = &new_message.content;
        if let Some(translated) = translate_if_needed(
            content,
            &new_message.author.id.to_string(),
            &data.db,
        )
            .await?
        {
            new_message.channel_id.say(&ctx.http, translated).await?;
        }
    }

    Ok(())
}
