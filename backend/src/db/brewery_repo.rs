use crate::db::Db;
use anyhow::Context;

#[derive(sqlx::FromRow)]
pub struct BreweryRow {
    pub id: String,
    pub name: String,
    pub website: Option<String>,
    pub address: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
}

/// Read access for brewery rows.
#[allow(async_fn_in_trait)]
pub trait BreweryRepo {
    async fn get_breweries(&self) -> anyhow::Result<Vec<BreweryRow>>;
    async fn brewery_exists(&self, brewery_id: &str) -> anyhow::Result<bool>;
}

impl BreweryRepo for Db {
    async fn get_breweries(&self) -> anyhow::Result<Vec<BreweryRow>> {
        sqlx::query_as::<_, BreweryRow>(
            r#"
    SELECT
      id,
      name,
      website,
      address,
      lat,
      lng
    FROM breweries
    ORDER BY name;
                "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch breweries from SQLite table 'breweries'")
    }

    async fn brewery_exists(&self, brewery_id: &str) -> anyhow::Result<bool> {
        let exists: Option<i64> =
            sqlx::query_scalar("SELECT 1 FROM breweries WHERE id = ? LIMIT 1;")
                .bind(brewery_id)
                .fetch_optional(&self.pool)
                .await
                .context("Failed to validate brewery id against SQLite table 'breweries'")?;
        Ok(exists.is_some())
    }
}
