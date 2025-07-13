use poise::serenity_prelude::{Context, Ready};
use poise::FrameworkContext;
use serenity::all::FullEvent;
use bot_macros::register_event;
use crate::{Data, FullError};

#[register_event]
async fn on_ready(
    ctx: &Context,
    event: &FullEvent,
    framework: FrameworkContext<'_, Data, FullError>,
    _data: &Data,
) -> Result<(), FullError> {
    if let FullEvent::Ready { data_about_bot } = event {
        println!("✅ Bot is now online as {}!", data_about_bot.user.name);
    }
    Ok(())
}