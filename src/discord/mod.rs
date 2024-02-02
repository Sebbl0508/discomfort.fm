mod commands;
mod data;
mod error;
mod utils;
mod voice;

use std::collections::HashMap;
use std::sync::Arc;

pub use data::Data;
pub use error::Error;

use poise::serenity_prelude::{self as serenity, Client, GuildId};
use songbird::SerenityInit;
use tokio::signal::unix::SignalKind;
use tokio::sync::RwLock;

use crate::{config::Config, database::DatabaseContext};

type Context<'a> = poise::Context<'a, Data, Error>;

pub async fn start(config: Config, db: DatabaseContext) -> Result<(), Error> {
    let token = config.discord_token.clone();

    let options = poise::FrameworkOptions {
        commands: vec![
            commands::echo(),
            commands::audio::volume(),
            commands::audio::play(),
            commands::audio::pause(),
            commands::audio::stop(),
            commands::audio::join(),
            commands::audio::disconnect(),
        ],
        on_error: |error| Box::pin(error::on_error(error)),
        pre_command: |ctx| {
            Box::pin(async move {
                tracing::debug!("executing command \"{}\"...", ctx.command().qualified_name,);
            })
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(move |ctx, ready, framework| {
            Box::pin(async move {
                tracing::info!("logged in as {}", ready.user.name);

                if config.is_debug {
                    let Some(dev_guild) = config.debug_guild else {
                        panic!("config.debug_guild is None, event though config.is_debug == true");
                    };

                    tracing::info!("registering commands in guild {dev_guild}");
                    poise::builtins::register_in_guild(
                        ctx,
                        &framework.options().commands,
                        GuildId::new(dev_guild),
                    )
                    .await?;
                } else if config.should_publish_global {
                    tracing::info!("registering commands globally");
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                }

                Ok(Data {
                    config: Arc::new(config),
                    database: db,
                    guild_tracks: RwLock::new(HashMap::new()),
                })
            })
        })
        .options(options)
        .build();

    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::GUILD_VOICE_STATES;

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .register_songbird()
        .await?;

    setup_graceful_shutdown(&mut client).await;

    client.start().await?;

    Ok(())
}

async fn setup_graceful_shutdown(client: &mut Client) {
    {
        let shartman = Arc::clone(&client.shard_manager);
        tokio::spawn(async move {
            tokio::signal::ctrl_c()
                .await
                .expect("couldn't register CTRL+C handler");

            tracing::warn!("received CTRL+C event, shutting down...");
            shartman.shutdown_all().await;
        });
    }
    {
        let shartman = Arc::clone(&client.shard_manager);
        tokio::spawn(async move {
            let mut stream = tokio::signal::unix::signal(SignalKind::terminate()).unwrap();
            stream.recv().await;

            tracing::warn!("received UNIX terminate signal, shutting down...");
            shartman.shutdown_all().await;
        });
    }
}
