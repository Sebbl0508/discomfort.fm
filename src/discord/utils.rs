use crate::discord::error::VoiceChannelJoinError;
use crate::discord::{Context, Error};
use poise::serenity_prelude::{ChannelId, GuildId, UserId};
use songbird::{Call, Songbird};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn get_guild_id_or_error(ctx: &Context<'_>) -> Result<GuildId, Error> {
    ctx.guild_id().ok_or_else(|| "couldn't get guild_id".into())
}

pub async fn get_songbird_or_error(ctx: &Context<'_>) -> Result<Arc<Songbird>, Error> {
    songbird::get(ctx.serenity_context())
        .await
        .ok_or_else(|| "couldn't get songbird manager".into())
}

pub async fn try_get_user_voice_channel(
    ctx: &Context<'_>,
    user_id: &UserId,
) -> Result<ChannelId, VoiceChannelJoinError> {
    let guild = ctx.guild().ok_or("couldn't get guild")?;

    guild
        .voice_states
        .get(user_id)
        .and_then(|vs| vs.channel_id)
        .ok_or(VoiceChannelJoinError::UserNotInVoiceChannel)
}

pub async fn try_join_user_voice_channel(
    ctx: &Context<'_>,
    songbird_mgr: &Songbird,
) -> Result<Arc<Mutex<Call>>, VoiceChannelJoinError> {
    let guild = ctx.guild().ok_or("couldn't get guild")?.to_owned();
    let channel_id = guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|vs| vs.channel_id)
        .ok_or(VoiceChannelJoinError::UserNotInVoiceChannel)?;

    songbird_mgr
        .join(guild.id, channel_id)
        .await
        .map_err(|e| VoiceChannelJoinError::Other(e.into()))
}
