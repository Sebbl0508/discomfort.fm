use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(not(debug_assertions))]
const MAX_LEVEL: tracing::Level = tracing::Level::INFO;

#[cfg(debug_assertions)]
const MAX_LEVEL: tracing::Level = tracing::Level::DEBUG;

pub fn setup_log() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug,hyper=info,h2=info,rustls=info,reqwest=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    /*
    tracing_subscriber::fmt()
        .with_max_level(MAX_LEVEL)
        .init();
    */
}
