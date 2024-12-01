use loco_rs::{app::AppContext, Result};
use serde::Deserialize;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Deserialize)]
#[serde(tag = "kind")]
pub enum DBConfig {
    Postgres(Postgres),
    Sqlite(Sqlite),
    // Redis(RedisConfig),
}

#[derive(Debug, Deserialize)]
pub struct Postgres {
    pub uri: String,
    pub connect_timeout: u64,
    pub idle_timeout: u64,
    pub min_connections: u32,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize)]
pub struct Sqlite {
    pub uri: String,
}

impl DBConfig {
    /// Return db config
    ///
    /// # Errors
    ///
    /// This function will return an error if it fails
    pub fn from_context(ctx: &AppContext) -> Result<Self> {
        let connection = ctx
            .config
            .settings
            .as_ref()
            .and_then(|settings| settings.get("db"))
            .ok_or_else(|| {
                tracing::error!(
                    environment = %ctx.environment,
                    "connection settings not found in configuration file"
                );
                loco_rs::Error::string("Connection to worker not configure")
            })?;

        let db_config: Self = serde_json::from_value(connection.clone()).map_err(|err| {
            tracing::error!(err = %err, "invalid worker configuration");
            err
        })?;
        Ok(db_config)
    }
}
