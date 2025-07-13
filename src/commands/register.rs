use chrono::Utc;
use bot_macros::register_command;
use crate::models::users::{User };
use crate::{BotContext, FullError};

#[register_command]
#[poise::command(slash_command)]
async fn register(ctx: BotContext<'_>) -> Result<(), FullError> {
    let user = ctx.author();
    let db = &ctx.data().db;
    let local = ctx.locale();

    let exists: Option<User> = sqlx::query_as::<_, User>(
        "SELECT id, name, lang_local, registered_at FROM users WHERE id = ?"
    )
        .bind(user.id.to_string())
        .fetch_optional(db)
        .await?;

    if exists.is_some() {
        ctx.say("You're already registered!").await?;
        return Ok(());
    }

    let now = Utc::now();

    sqlx::query(
        "INSERT INTO users (id, name, lang_local, registered_at) VALUES (?, ?, ?, ?)"
    )
        .bind(user.id.to_string())
        .bind(user.name.clone())
        .bind(local)
        .bind(now)
        .execute(db)
        .await?;

    ctx.say("You have been registered successfully! 🎉").await?;
    Ok(())
}
