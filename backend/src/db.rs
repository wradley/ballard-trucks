mod brewery_repo;
mod schedule_repo;
mod vendor_repo;

use anyhow::Context;
pub use brewery_repo::{BreweryRepo, BreweryRow};
pub use schedule_repo::{ScheduleRepo, ScheduleRow};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Pool, Postgres};
use std::env;
use std::time::Duration;
pub use vendor_repo::{VendorRepo, VendorRow};

#[derive(Clone)]
pub struct Db {
    pool: Pool<Postgres>,
}

impl Db {
    /// Initializes the shared database handle from `DB_*` environment variables.
    pub async fn init() -> anyhow::Result<Self> {
        let host = env::var("DB_HOST").unwrap_or("localhost".to_string());
        let port = env::var("DB_PORT")
            .unwrap_or("5432".to_string())
            .parse::<u16>()
            .context("Failed to parse DB_PORT environment variable")?;
        let user = env::var("DB_USER").unwrap_or("ballard".to_string());
        let password = env::var("DB_PASSWORD").unwrap_or("ballard".to_string());
        let db_name = env::var("DB_NAME").unwrap_or("ballard_trucks".to_string());

        let connection_options = PgConnectOptions::new()
            .host(&host)
            .port(port)
            .username(&user)
            .password(&password)
            .database(&db_name);

        let pool = match PgPoolOptions::new()
            .max_connections(10)
            .max_lifetime(Some(Duration::from_hours(1)))
            .connect_with(connection_options)
            .await
        {
            Ok(p) => p,
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to connect to database: {}", e));
            }
        };

        Ok(Db { pool })
    }

    pub(crate) fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }
}
