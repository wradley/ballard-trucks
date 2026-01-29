use crate::db::Db;
use anyhow::Context;
use sqlx::postgres::types::PgRange;
use sqlx::types::chrono::{DateTime, Utc};
use std::time::Duration;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct ScheduleRow {
    pub id: Uuid,
    pub brewery_id: Uuid,
    pub brewery_name: String,
    pub food_vendor_id: Uuid,
    pub food_vendor_name: String,
    pub open_hours: PgRange<DateTime<Utc>>,
    pub source: String,
    pub updated_at: DateTime<Utc>,
}

/// Read access for schedule query rows.
pub trait ScheduleRepo {
    async fn get_schedules_within(
        &self,
        start: jiff::Timestamp,
        duration_hours: u64,
    ) -> anyhow::Result<Vec<ScheduleRow>>;
}

impl ScheduleRepo for Db {
    async fn get_schedules_within(
        &self,
        start: jiff::Timestamp,
        duration_hours: u64,
    ) -> anyhow::Result<Vec<ScheduleRow>> {
        let soon = start + Duration::from_hours(duration_hours);
        sqlx::query_as::<_, ScheduleRow>(
            r#"
    SELECT * FROM public.schedule_entries WHERE
    TSTZRANGE($1::timestamptz, $2::timestamptz) && open_hours
    LIMIT 100;
                "#,
        )
        .bind(start.to_string())
        .bind(soon.to_string())
        .fetch_all(self.pool())
        .await
        .context("Failed to fetch schedule entries")
    }
}
