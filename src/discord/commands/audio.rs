use poise::serenity_prelude::GuildChannel;
use songbird::tracks::Track;
use songbird::TrackEvent;
use url::Url;

use crate::database::actions::{volume_get_or_insert_default, volume_insert_or_update};
use crate::discord::error::VoiceChannelJoinError;
use crate::discord::utils::{
    get_guild_id_or_error, get_songbird_or_error, try_get_user_voice_channel,
    try_join_user_voice_channel,
};
use crate::discord::voice::TrackErrorNotifier;
use crate::discord::{Context, Error};

const INITIAL_DEFAULT_VOLUME: i32 = 100;

/// Play some radio!
#[poise::command(slash_command)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "Webradio URL"] url: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let mut conn = ctx.data().database.get_connection().await?;

    let Ok(url) = Url::parse(&url) else {
        ctx.say(format!(
            "Error parsing URL \"{}\". Are you sure it's correct?",
            url
        ))
        .await?;
        return Ok(());
    };

    let guild_id = get_guild_id_or_error(&ctx)?;
    let songbird_mgr = get_songbird_or_error(&ctx).await?;

    let voice_handler = match songbird_mgr.get(guild_id) {
        Some(v) => v,
        None => match try_join_user_voice_channel(&ctx, &songbird_mgr).await {
            Ok(v) => {
                v.lock()
                    .await
                    .add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
                v
            }
            Err(VoiceChannelJoinError::UserNotInVoiceChannel) => {
                ctx.say("I'm not in a voice channel and it seems you are not in a voice channel i can access...").await?;
                return Ok(());
            }
            Err(VoiceChannelJoinError::Other(e)) => {
                ctx.say("There was an error joining your voice channel...")
                    .await?;
                return Err(e);
            }
        },
    };

    {
        let voice_handler_lock = voice_handler.lock().await;

        // A voice handle can still exist for a guild, even though the bot isn't connected to any VC
        if voice_handler_lock.current_connection().is_none() {
            // Prevent deadlock. This could be handled better, but i don't care :3
            drop(voice_handler_lock);

            let user_vc = match try_get_user_voice_channel(&ctx, &ctx.author().id).await {
                Ok(v) => v,
                Err(VoiceChannelJoinError::UserNotInVoiceChannel) => {
                    ctx.say("I'm not in a voice channel and it seems you are not in a voice channel i can access").await?;
                    return Ok(());
                }
                Err(VoiceChannelJoinError::Other(e)) => {
                    return Err(e);
                }
            };

            songbird_mgr.join(guild_id, user_vc).await?;
        }
    }

    let vol = volume_get_or_insert_default(&mut conn, guild_id, INITIAL_DEFAULT_VOLUME).await?;

    // convert 0-100 to 0.0-1.0
    let vol: f32 = vol as f32 / 100.0;

    let mut voice_handler_lock = voice_handler.lock().await;

    let webradio_input_ytdl =
        songbird::input::YoutubeDl::new(reqwest::Client::new(), url.to_string());

    let track_handle = voice_handler_lock.play_only(Track::from(webradio_input_ytdl).volume(vol));

    ctx.data()
        .guild_tracks
        .write()
        .await
        .insert(guild_id, track_handle);

    ctx.say(format!("Playing {}", url)).await?;

    Ok(())
}

/// Stop playing radio
#[poise::command(slash_command)]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = get_guild_id_or_error(&ctx)?;
    let songbird_mgr = get_songbird_or_error(&ctx).await?;

    const NOT_IN_VC_ERR: &str = "It appears i'm not in a voice channel. Then how the fuck should i stop playing something?!?!\n**IDIOT**!";

    let Some(voice_handler) = songbird_mgr.get(guild_id) else {
        ctx.say(NOT_IN_VC_ERR).await?;
        return Ok(());
    };

    let mut voice_handler_lock = voice_handler.lock().await;

    if voice_handler_lock.current_connection().is_none() {
        ctx.say(NOT_IN_VC_ERR).await?;
        return Ok(());
    }

    ctx.data().guild_tracks.write().await.remove(&guild_id);
    voice_handler_lock.stop();

    ctx.say("Stopping...").await?;

    Ok(())
}

/// Pause the playing radio station
#[poise::command(slash_command)]
pub async fn pause(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    ctx.say("TODO").await?;

    Ok(())
}

/// Join a voice channel
#[poise::command(slash_command)]
pub async fn join(
    ctx: Context<'_>,
    #[description = "The voice channel to join"]
    #[channel_types("Voice")]
    channel: Option<GuildChannel>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let songbird_mgr = get_songbird_or_error(&ctx).await?;

    if let Some(channel) = channel {
        match songbird_mgr.join(channel.guild_id, channel.id).await {
            Ok(handler) => {
                handler
                    .lock()
                    .await
                    .add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
            }
            Err(e) => {
                ctx.say("There was an error joining the voice channel...")
                    .await?;
                return Err(e.into());
            }
        }

        ctx.say("Here i am!").await?;
        return Ok(());
    }

    match try_join_user_voice_channel(&ctx, &songbird_mgr).await {
        Ok(_) => {}
        Err(VoiceChannelJoinError::UserNotInVoiceChannel) => {
            ctx.say("You don't seem to be in any voice channel i can access!")
                .await?;
            return Ok(());
        }
        Err(VoiceChannelJoinError::Other(e)) => {
            ctx.say("There was an error joining the voice channel you are in...")
                .await?;
            return Err(e);
        }
    }

    ctx.say("Here i am!").await?;

    Ok(())
}

/// Leave the voice channel, if connected
#[poise::command(slash_command)]
pub async fn disconnect(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = get_guild_id_or_error(&ctx)?;
    let songbird_mgr = get_songbird_or_error(&ctx).await?;

    let Some(handler) = songbird_mgr.get(guild_id) else {
        ctx.say("I'm not in any voice channel, idiot!").await?;
        return Ok(());
    };

    ctx.data().guild_tracks.write().await.remove(&guild_id);

    handler.lock().await.leave().await?;

    ctx.say("Bye bye!").await?;

    Ok(())
}

/// Get or set the audio volume
#[poise::command(slash_command)]
pub async fn volume(
    ctx: Context<'_>,
    #[description = "Get or set the audio volume"] volume: Option<u32>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let mut conn = ctx.data().database.get_connection().await?;

    let guild_id = get_guild_id_or_error(&ctx)?;
    let songbird_mgr = get_songbird_or_error(&ctx).await?;

    let voice_handler = songbird_mgr.get(guild_id);

    if let Some(volume) = volume {
        if volume > ctx.data().config.max_volume {
            ctx.say(format!(
                "Volume `{volume}` is higher than the maximum (`{}`)",
                ctx.data().config.max_volume
            ))
            .await?;
            return Ok(());
        }

        if voice_handler.is_some() {
            if let Some(track_handle) = ctx.data().guild_tracks.read().await.get(&guild_id) {
                track_handle.set_volume(volume as f32 / 100.0).ok();
            }
        }

        volume_insert_or_update(&mut conn, guild_id, volume as i32).await?;

        ctx.say(format!("Set volume to `{volume}`")).await?;
        return Ok(());
    };

    let vol = volume_get_or_insert_default(&mut conn, guild_id, INITIAL_DEFAULT_VOLUME).await?;

    ctx.say(format!("The volume is set to `{vol}`")).await?;
    Ok(())
}
