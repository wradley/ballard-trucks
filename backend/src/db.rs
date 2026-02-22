mod brewery_repo;
mod schedule_repo;
mod vendor_repo;

use anyhow::Context;
pub use brewery_repo::{BreweryRepo, BreweryRow};
pub use schedule_repo::{ScheduleRepo, ScheduleRow, ScheduleUpsertInput, ScheduleWriteRepo};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{Pool, Row, Sqlite};
use std::env;
use std::fs;
use std::path::PathBuf;
use tracing::info;
pub use vendor_repo::{VendorRepo, VendorRow, VendorWriteRepo};

const DEFAULT_DB_PATH: &str = "./data/ballard.sqlite";
const SCHEMA_SQL: &str = include_str!("../../db/schema.sql");
const SEED_SQL: &str = include_str!("../../db/seed.sql");

#[derive(Clone)]
pub struct Db {
    pub(crate) pool: Pool<Sqlite>,
}

impl Db {
    /// Initializes the shared SQLite database with an optional path override.
    pub async fn init_with_path_override(path_override: Option<String>) -> anyhow::Result<Self> {
        let db_path = sqlite_path_from_env(path_override)?;
        let db_path = PathBuf::from(db_path);
        info!("Resolved SQLite path: {}", db_path.display());

        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "Failed to create SQLite parent directory {}",
                    parent.display()
                )
            })?;
        }

        let connection_options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(connection_options)
            .await
            .with_context(|| {
                format!("Failed to connect to SQLite database {}", db_path.display())
            })?;

        sqlx::query("PRAGMA busy_timeout = 5000;")
            .execute(&pool)
            .await
            .context("Failed to configure SQLite busy_timeout")?;

        sqlx::raw_sql(SCHEMA_SQL)
            .execute(&pool)
            .await
            .context("Failed to initialize SQLite schema")?;

        ensure_schema_compatibility(&pool).await?;

        sqlx::raw_sql(SEED_SQL)
            .execute(&pool)
            .await
            .context("Failed to seed SQLite database")?;

        Ok(Db { pool })
    }
}

fn sqlite_path_from_env(path_override: Option<String>) -> anyhow::Result<String> {
    if let Some(path) = path_override {
        return Ok(path);
    }

    if let Ok(path) = env::var("SQLITE_PATH") {
        return Ok(path);
    }

    if let Ok(database_url) = env::var("DATABASE_URL") {
        return sqlite_path_from_url(&database_url);
    }

    Ok(DEFAULT_DB_PATH.to_string())
}

fn sqlite_path_from_url(database_url: &str) -> anyhow::Result<String> {
    let prefix = "sqlite://";
    if let Some(path) = database_url.strip_prefix(prefix) {
        if path.is_empty() {
            anyhow::bail!("DATABASE_URL must include a SQLite file path");
        }
        return Ok(path.to_string());
    }

    anyhow::bail!("DATABASE_URL must start with sqlite://");
}

async fn ensure_schema_compatibility(pool: &Pool<Sqlite>) -> anyhow::Result<()> {
    // Additive reconciliation for older DB files mounted from existing volumes.
    ensure_column(
        pool,
        "breweries",
        "drink_menu",
        "ALTER TABLE breweries ADD COLUMN drink_menu TEXT",
    )
    .await?;
    ensure_column(
        pool,
        "breweries",
        "food_schedule",
        "ALTER TABLE breweries ADD COLUMN food_schedule TEXT",
    )
    .await?;
    ensure_column(
        pool,
        "vendors",
        "normalized_name",
        "ALTER TABLE vendors ADD COLUMN normalized_name TEXT",
    )
    .await?;
    ensure_column(
        pool,
        "vendors",
        "needs_review",
        "ALTER TABLE vendors ADD COLUMN needs_review INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_column(
        pool,
        "vendors",
        "match_method",
        "ALTER TABLE vendors ADD COLUMN match_method TEXT",
    )
    .await?;
    ensure_column(
        pool,
        "vendors",
        "match_score",
        "ALTER TABLE vendors ADD COLUMN match_score REAL",
    )
    .await?;
    ensure_column(
        pool,
        "vendors",
        "updated_at",
        "ALTER TABLE vendors ADD COLUMN updated_at TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%fZ', 'now'))",
    )
    .await?;
    ensure_column(
        pool,
        "schedule_entries",
        "source_url",
        "ALTER TABLE schedule_entries ADD COLUMN source_url TEXT",
    )
    .await?;
    ensure_column(
        pool,
        "schedule_entries",
        "scrape_run_id",
        "ALTER TABLE schedule_entries ADD COLUMN scrape_run_id TEXT",
    )
    .await?;
    ensure_column(
        pool,
        "schedule_entries",
        "is_active",
        "ALTER TABLE schedule_entries ADD COLUMN is_active INTEGER NOT NULL DEFAULT 1",
    )
    .await?;
    ensure_column(
        pool,
        "schedule_entries",
        "superseded_at",
        "ALTER TABLE schedule_entries ADD COLUMN superseded_at TEXT",
    )
    .await?;
    ensure_column(
        pool,
        "schedule_entries",
        "scraped_at",
        "ALTER TABLE schedule_entries ADD COLUMN scraped_at TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%fZ', 'now'))",
    )
    .await?;

    Ok(())
}

async fn ensure_column(
    pool: &Pool<Sqlite>,
    table: &str,
    column: &str,
    alter_sql: &str,
) -> anyhow::Result<()> {
    let pragma_sql = format!("PRAGMA table_info({table});");
    let columns = sqlx::query(&pragma_sql)
        .fetch_all(pool)
        .await
        .with_context(|| format!("Failed to inspect SQLite schema for table {table}"))?;

    let exists = columns.iter().any(|row| {
        row.try_get::<String, _>("name")
            .is_ok_and(|name| name == column)
    });

    if !exists {
        sqlx::query(alter_sql)
            .execute(pool)
            .await
            .with_context(|| format!("Failed to add SQLite column {table}.{column}"))?;
    }

    Ok(())
}
