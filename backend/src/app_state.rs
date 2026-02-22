use crate::db::Db;
use axum::extract::FromRef;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub ingest_api_key: Uuid,
}

impl AppState {
    pub async fn init(sqlite_path_override: Option<String>) -> anyhow::Result<Self> {
        let db = Db::init_with_path_override(sqlite_path_override).await?;
        let ingest_api_key =
            std::env::var("INGEST_API_KEY").expect("Missing `INGEST_API_KEY` env variable");
        let ingest_api_key = Uuid::from_str(&ingest_api_key)
            .expect(&format!("Invalid INGEST_API_KEY uuid: {}", ingest_api_key));

        Ok(Self { db, ingest_api_key })
    }
}

impl FromRef<AppState> for Db {
    fn from_ref(input: &AppState) -> Self {
        input.db.clone()
    }
}
