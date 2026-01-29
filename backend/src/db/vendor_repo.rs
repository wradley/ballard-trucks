use crate::db::Db;
use anyhow::Context;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct VendorRow {
    pub id: Uuid,
    pub name: String,
    pub notes: Option<String>,
    pub website: Option<String>,
    pub menu: Option<String>,
}

/// Read access for vendor rows.
pub trait VendorRepo {
    async fn get_vendors(&self) -> anyhow::Result<Vec<VendorRow>>;
}

impl VendorRepo for Db {
    async fn get_vendors(&self) -> anyhow::Result<Vec<VendorRow>> {
        sqlx::query_as::<_, VendorRow>(
            r#"
    SELECT * FROM public.food_vendors
    ORDER BY name;
                "#,
        )
        .fetch_all(self.pool())
        .await
        .context("Failed to fetch vendors")
    }
}
