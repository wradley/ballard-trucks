use anyhow::Context;
use ballard_trucks_backend::db::{Db, ScheduleRepo};
use ballard_trucks_backend::domain::{
    IngestEventInput, IngestScheduleBatchInput, ingest_schedule_batch,
};
use sqlx::sqlite::SqlitePoolOptions;

const TEST_BREWERY_ID: &str = "6373ac59-7f83-4565-8f94-9d4d6d7582ec";

#[tokio::test]
async fn ingest_flow_merges_vendor_variants_and_supersedes_old_rows() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir().context("failed to create temp dir")?;
    let db_path = temp_dir.path().join("ingest-flow.sqlite");
    let db_path_str = db_path
        .to_str()
        .context("db path is not valid UTF-8")?
        .to_string();

    let db = Db::init_with_path_override(Some(db_path_str.clone())).await?;

    ingest_schedule_batch(
        &db,
        IngestScheduleBatchInput {
            run_id: "run-1".to_string(),
            source: "integration-test".to_string(),
            brewery_id: TEST_BREWERY_ID.to_string(),
            events: vec![IngestEventInput {
                vendor_name: "Tacos Califas".to_string(),
                start_at: "2026-05-15T01:00:00Z".to_string(),
                end_at: "2026-05-15T04:00:00Z".to_string(),
                source_url: None,
            }],
        },
        30,
    )
    .await?;

    ingest_schedule_batch(
        &db,
        IngestScheduleBatchInput {
            run_id: "run-2".to_string(),
            source: "integration-test".to_string(),
            brewery_id: TEST_BREWERY_ID.to_string(),
            events: vec![IngestEventInput {
                vendor_name: "Tacos Califas debut @ Fair Isle Brewing".to_string(),
                start_at: "2026-05-16T01:00:00Z".to_string(),
                end_at: "2026-05-16T04:00:00Z".to_string(),
                source_url: None,
            }],
        },
        30,
    )
    .await?;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite://{db_path_str}"))
        .await
        .context("failed to connect to integration test sqlite db")?;

    let tacos_vendor_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM vendors WHERE normalized_name = 'tacoscalifas';")
            .fetch_one(&pool)
            .await?;
    assert_eq!(tacos_vendor_count, 1);

    let active_rows: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM schedule_entries WHERE source = ? AND brewery_id = ? AND is_active = 1;",
    )
    .bind("integration-test")
    .bind(TEST_BREWERY_ID)
    .fetch_one(&pool)
    .await?;
    assert_eq!(active_rows, 1);

    let inactive_rows: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM schedule_entries WHERE source = ? AND brewery_id = ? AND is_active = 0;",
    )
    .bind("integration-test")
    .bind(TEST_BREWERY_ID)
    .fetch_one(&pool)
    .await?;
    assert_eq!(inactive_rows, 1);

    let schedules = db
        .get_schedules_within("2026-05-15T00:00:00Z".parse()?, 72)
        .await?;
    assert_eq!(schedules.len(), 1);
    assert_eq!(schedules[0].vendor_name, "Tacos Califas");

    Ok(())
}
