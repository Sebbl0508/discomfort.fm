use poise::FrameworkError;

use crate::discord::Data;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// Error thrown, when the bot couldn't join a channel
#[derive(Debug)]
pub enum VoiceChannelJoinError {
    /// The calling user is not in any voice channel accessible by the bot
    UserNotInVoiceChannel,
    /// Some other error
    Other(Error),
}

pub async fn on_error(error: FrameworkError<'_, Data, Error>) {
    match error {
        FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {error:?}"),
        FrameworkError::Command { error, ctx, .. } => {
            tracing::error!("error in command `{}`: {:?}", ctx.command().name, error);

            if let Err(e) = ctx
                .reply("There was an error processing your request")
                .await
            {
                tracing::error!("error replying with error message :D => {}", e);
            }
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                tracing::error!("error while handling error: {}", e);
            }
        }
    }
}

impl From<&'static str> for VoiceChannelJoinError {
    fn from(value: &'static str) -> Self {
        Self::Other(value.into())
    }
}
