use crate::{config::Config, database::DatabaseContext};

mod config;
mod database;
mod discord;
mod logger;

#[tokio::main]
async fn main() {
    logger::setup_log();

    if let Err(e) = dotenvy::dotenv() {
        tracing::warn!("couldn't load dotenv: {e:?}");
    }

    tracing::info!("Hello world");

    let config = Config::get().unwrap();
    tracing::info!("Database URI: \"{}\"", config.database_path);
    let db = DatabaseContext::new(&config.database_path).await.unwrap();

    db.init().await.unwrap();

    if let Err(e) = discord::start(config, db).await {
        tracing::error!("error while executing discord bot: {e}");
    }
}
