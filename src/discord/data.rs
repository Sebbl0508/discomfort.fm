use std::collections::HashMap;
use std::sync::Arc;

use poise::serenity_prelude::GuildId;
use songbird::tracks::TrackHandle;
use tokio::sync::RwLock;

use crate::{config::Config, database::DatabaseContext};

/// Data shared by discord-related code
pub struct Data {
    /// Reference to the application config
    pub config: Arc<Config>,

    pub database: DatabaseContext,

    pub guild_tracks: RwLock<HashMap<GuildId, TrackHandle>>,
}
