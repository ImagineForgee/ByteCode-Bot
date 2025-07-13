mod commands;
mod models;
mod events;
mod utils;

use std::fs::exists;
use std::io;
use std::path::Path;
use clap::Parser;
use poise::serenity_prelude as serenity;
use poise::Context;
use serde::{Deserialize, Serialize};
use serenity::all::GuildId;
use sqlx::SqlitePool;
use log::log;
use reqwest::Client;
use serenity::prelude::GatewayIntents;
use crate::commands::collect_commands;
use crate::events::dispatch_all_events;
use crate::utils::translate::detect_language_lingva;

pub struct Data {
    pub db: SqlitePool,
}

impl Clone for Data {
    fn clone(&self) -> Self {
        Data {db: self.db.clone()}
    }
}

pub type FullError = Box<dyn std::error::Error + Send + Sync>;
type BotContext<'a> = Context<'a, Data, FullError>;

#[tokio::main]
async fn main() {
	let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            event_handler: dispatch_all_events,
            commands: collect_commands(),
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                setup_db_file().await.expect("Failed to create database file");
                let pool = SqlitePool::connect("sqlite://data/users.db").await?;


                sqlx::query(
                    r#"
                        CREATE TABLE IF NOT EXISTS users (
                            id TEXT PRIMARY KEY,
                            name TEXT NOT NULL,
                            lang_local TEXT NOT NULL,
                            registered_at TEXT NOT NULL
                        )
                        "#,
                )
                    .execute(&pool)
                    .await?;

                poise::builtins::register_in_guild(ctx, &framework.options().commands, GuildId::new(1392716273741201408)).await?;
                Ok(Data { db: pool })
            })
        })
        .build();

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGES;

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect("Error building client");

    client.start().await.expect("Error starting client");
}

async fn setup_db_file() -> io::Result<()> {
    let db_path = Path::new("data/users.db");
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if db_path.exists() {
        log::info!("Database already exists: {}", db_path.display());
    } else {
        log::info!("Creating database: {}", db_path.display());
        std::fs::File::create(db_path)?;
    }
    Ok(())
}
