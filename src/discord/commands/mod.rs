pub mod audio;

use crate::discord::{Context, Error};

#[poise::command(slash_command)]
pub async fn echo(
    ctx: Context<'_>,
    #[description = "The message to repeate"] message: String,
) -> Result<(), Error> {
    ctx.say(message).await?;
    Ok(())
}
