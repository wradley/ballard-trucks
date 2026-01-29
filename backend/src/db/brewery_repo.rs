use crate::db::Db;
use anyhow::Context;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct BreweryRow {
    pub id: Uuid,
    pub name: String,
    pub notes: Option<String>,
    pub website: Option<String>,
    pub address: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub drink_menu: Option<String>,
    pub food_schedule: Option<String>,
}

/// Read access for brewery rows.
pub trait BreweryRepo {
    async fn get_breweries(&self) -> anyhow::Result<Vec<BreweryRow>>;
}

impl BreweryRepo for Db {
    async fn get_breweries(&self) -> anyhow::Result<Vec<BreweryRow>> {
        sqlx::query_as::<_, BreweryRow>(
            r#"
    SELECT * FROM public.breweries
    ORDER BY name;
                "#,
        )
        .fetch_all(self.pool())
        .await
        .context("Failed to fetch breweries")
    }
}
