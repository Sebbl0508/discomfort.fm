use std::str::FromStr;

use chrono::{DateTime, Utc};
use poise::serenity_prelude::GuildId;
use sqlx::{pool::PoolConnection, sqlite::SqlitePoolOptions, Sqlite};
use uuid::Uuid;

use crate::discord::Error;

#[derive(Clone)]
pub struct DatabaseContext {
    pub pool: sqlx::Pool<Sqlite>,
}

impl DatabaseContext {
    pub async fn new(connection_uri: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(connection_uri)
            .await?;

        Ok(Self { pool })
    }

    pub async fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;

        Ok(())
    }

    pub async fn get_connection(&self) -> Result<PoolConnection<Sqlite>, Error> {
        Ok(self.pool.acquire().await?)
    }
}

pub trait FromRawRow {
    type RawRow;
    fn from_raw_row(raw_row: Self::RawRow) -> Self;
}

#[derive(Debug, sqlx::FromRow)]
pub struct GuildRowRaw {
    pub id: String,
    pub volume: i32,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GuildRow {
    pub id: GuildId,
    pub volume: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl FromRawRow for GuildRow {
    type RawRow = GuildRowRaw;

    fn from_raw_row(raw_row: Self::RawRow) -> Self {
        GuildRow {
            id: GuildId::new(
                raw_row
                    .id
                    .parse()
                    .expect(format!("couldn't parse guild-id from \"{}\"", &raw_row.id).as_str()),
            ),
            volume: raw_row.volume,
            created_at: raw_row.created_at.parse().expect(
                format!(
                    "couldn't parse timestamp \"{}\" (guild_id {})",
                    &raw_row.created_at, &raw_row.id
                )
                .as_str(),
            ),
            updated_at: raw_row.updated_at.map(|v| {
                v.parse::<DateTime<Utc>>().expect(
                    format!(
                        "couldn't parse timestamp \"{}\" (guild_id {})",
                        &v, &raw_row.id
                    )
                    .as_str(),
                )
            }),
        }
    }
}

pub mod actions {
    use crate::database::{FromRawRow, GuildRow, GuildRowRaw};
    use crate::discord::Error;
    use chrono::Utc;
    use poise::serenity_prelude::GuildId;
    use sqlx::SqliteConnection;

    pub async fn volume_insert_or_update(
        conn: &mut SqliteConnection,
        guild_id: GuildId,
        volume: i32,
    ) -> Result<(), Error> {
        let now = Utc::now().to_rfc3339();

        let _res = sqlx::query(
            r"INSERT INTO guilds (id, volume, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(id) DO UPDATE SET volume=excluded.volume, updated_at=excluded.updated_at",
        )
        .bind(guild_id.get().to_string())
        .bind(volume)
        .bind(&now)
        .bind(&now)
        .execute(conn)
        .await?;

        Ok(())
    }

    pub async fn volume_get_or_insert_default(
        conn: &mut SqliteConnection,
        guild_id: GuildId,
        default_volume: i32,
    ) -> Result<i32, Error> {
        let guild = sqlx::query_as::<_, GuildRowRaw>("SELECT * FROM guilds WHERE id = ?1")
            .bind(guild_id.get().to_string())
            .fetch_optional(&mut *conn)
            .await?;

        if let Some(guild) = guild {
            let guild = GuildRow::from_raw_row(guild);
            return Ok(guild.volume);
        }

        let now = Utc::now().to_rfc3339();

        let _res = sqlx::query(
            "INSERT INTO guilds (id, volume, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(guild_id.get().to_string())
        .bind(default_volume)
        .bind(&now)
        .bind(&now)
        .execute(conn)
        .await?;

        Ok(default_volume)
    }
}
