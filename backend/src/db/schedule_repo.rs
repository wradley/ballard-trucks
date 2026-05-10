use crate::db::Db;
use anyhow::Context;
use sqlx::types::chrono::{DateTime, Utc};
use std::time::Duration;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct ScheduleRow {
    pub brewery_id: String,
    pub brewery_name: String,
    pub vendor_id: String,
    pub vendor_name: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Read access for schedule query rows.
#[allow(async_fn_in_trait)]
pub trait ScheduleRepo {
    async fn get_schedules_within(
        &self,
        start: jiff::Timestamp,
        duration_hours: u64,
    ) -> anyhow::Result<Vec<ScheduleRow>>;
}

pub struct ScheduleUpsertInput<'a> {
    pub brewery_id: &'a str,
    pub vendor_id: &'a str,
    pub start_at: &'a str,
    pub end_at: &'a str,
    pub source: &'a str,
    pub source_url: Option<&'a str>,
    pub run_id: &'a str,
    pub now_rfc3339: &'a str,
}

#[allow(async_fn_in_trait)]
pub trait ScheduleWriteRepo {
    async fn upsert_schedule_entry(&self, input: ScheduleUpsertInput<'_>) -> anyhow::Result<()>;
    async fn supersede_stale_active_entries(
        &self,
        source: &str,
        brewery_id: &str,
        run_id: &str,
        now_rfc3339: &str,
    ) -> anyhow::Result<()>;
    async fn delete_entries_older_than(&self, cutoff_rfc3339: &str) -> anyhow::Result<usize>;
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
    SELECT
      s.brewery_id,
      b.name AS brewery_name,
      s.vendor_id,
      v.name AS vendor_name,
      s.start_at,
      s.end_at,
      s.updated_at
    FROM schedule_entries s
    JOIN breweries b ON b.id = s.brewery_id
    JOIN vendors v ON v.id = s.vendor_id
    WHERE s.start_at < ? AND s.end_at > ? AND s.is_active = 1
    ORDER BY start_at ASC
    LIMIT 100;
                "#,
        )
        .bind(soon.to_string())
        .bind(start.to_string())
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch schedule entries from SQLite table 'schedule_entries'")
    }
}

impl ScheduleWriteRepo for Db {
    async fn upsert_schedule_entry(&self, input: ScheduleUpsertInput<'_>) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO schedule_entries (
              id, brewery_id, vendor_id,
              start_at, end_at, source, source_url,
              scrape_run_id, is_active, superseded_at, scraped_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1, NULL, ?, ?)
            ON CONFLICT(source, brewery_id, vendor_id, start_at, end_at) DO UPDATE SET
              source_url = excluded.source_url,
              scrape_run_id = excluded.scrape_run_id,
              is_active = 1,
              superseded_at = NULL,
              scraped_at = excluded.scraped_at,
              updated_at = excluded.updated_at
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(input.brewery_id)
        .bind(input.vendor_id)
        .bind(input.start_at)
        .bind(input.end_at)
        .bind(input.source)
        .bind(input.source_url)
        .bind(input.run_id)
        .bind(input.now_rfc3339)
        .bind(input.now_rfc3339)
        .execute(&self.pool)
        .await
        .context("Failed to upsert schedule entry during ingest")?;

        Ok(())
    }

    async fn supersede_stale_active_entries(
        &self,
        source: &str,
        brewery_id: &str,
        run_id: &str,
        now_rfc3339: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE schedule_entries
            SET is_active = 0, superseded_at = ?, updated_at = ?
            WHERE source = ? AND brewery_id = ? AND is_active = 1 AND scrape_run_id != ?;
            "#,
        )
        .bind(now_rfc3339)
        .bind(now_rfc3339)
        .bind(source)
        .bind(brewery_id)
        .bind(run_id)
        .execute(&self.pool)
        .await
        .context("Failed to supersede prior active schedule entries")?;

        Ok(())
    }

    async fn delete_entries_older_than(&self, cutoff_rfc3339: &str) -> anyhow::Result<usize> {
        let deleted = sqlx::query("DELETE FROM schedule_entries WHERE end_at < ?;")
            .bind(cutoff_rfc3339)
            .execute(&self.pool)
            .await
            .context("Failed to delete expired schedule entries")?
            .rows_affected() as usize;
        Ok(deleted)
    }
}
