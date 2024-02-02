use std::env;

use directories::ProjectDirs;

#[derive(Debug, Clone)]
pub struct Config {
    pub project_dirs: ProjectDirs,
    pub database_path: String,

    pub discord_token: String,
    pub is_debug: bool,
    pub debug_guild: Option<u64>,
    pub self_deaf: bool,
    pub max_volume: u32,
    pub should_publish_global: bool,
}

impl Config {
    pub fn get() -> Result<Self, Box<dyn std::error::Error>> {
        let project_dirs =
            directories::ProjectDirs::from("com", "github.sebbl0508", "discomfort-fm")
                .ok_or("couldn't get base directories")?;

        tracing::debug!(
            "creating folder \"{}\"",
            project_dirs.data_local_dir().display()
        );
        std::fs::create_dir_all(project_dirs.data_local_dir())?;

        let discord_token = env_load_or_err("DISCORD_TOKEN")?;

        let debug = env_load_bool_with_default("DEBUG", false);

        // Require "DEBUG_GUILD" to be set, if debug mode was enabled via the env variable
        let debug_guild = {
            let debug_guild_res = env_load_or_err("DEBUG_GUILD");
            if debug {
                Some(debug_guild_res?.parse::<u64>()?)
            } else {
                debug_guild_res
                    .and_then(|v| {
                        v.parse::<u64>()
                            .map_err(|_| "cannot parse dev guild as u64".to_string())
                    })
                    .ok()
            }
        };

        let should_publish_global = env_load_bool_with_default("PUBLISH_GLOBAL", false);

        let self_deaf = env_load_bool_with_default("SELF_DEAF", true);
        let max_volume = env_load_or_err("MAX_VOLUME")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<u32>()?;

        let database_path = env_load_or_err("DATABASE_URL")
            .unwrap_or_else(|_| default_database_path(&project_dirs));

        Ok(Self {
            project_dirs,
            database_path,

            discord_token,
            is_debug: debug,
            should_publish_global,
            debug_guild,
            self_deaf,
            max_volume,
        })
    }
}

fn default_database_path(project_dirs: &ProjectDirs) -> String {
    let mut db_dir = project_dirs.data_local_dir().to_path_buf();
    db_dir.push("data.db");

    format!("sqlite://{}?mode=rwc", db_dir.display())
}

fn env_load_or_err(key: &str) -> Result<String, String> {
    env::var(key).map_err(|_| format!("couldn't load {} from env", key))
}

fn env_load_bool_with_default(key: &str, default: bool) -> bool {
    match env_load_or_err(key)
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "true" => true,
        "false" => false,
        _ => default,
    }
}
