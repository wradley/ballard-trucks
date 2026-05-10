use crate::db::{BreweryRepo, Db, ScheduleUpsertInput, ScheduleWriteRepo, VendorWriteRepo};
use chrono::{Duration, Utc};
use thiserror::Error;

pub struct IngestEventInput {
    pub vendor_name: String,
    pub start_at: String,
    pub end_at: String,
    pub source_url: Option<String>,
}

pub struct IngestScheduleBatchInput {
    pub run_id: String,
    pub source: String,
    pub brewery_id: String,
    pub events: Vec<IngestEventInput>,
}

#[derive(Debug, Error)]
pub enum IngestError {
    #[error("Unknown brewery_id: {0}")]
    UnknownBreweryId(String),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

pub async fn ingest_schedule_batch(
    db: &Db,
    input: IngestScheduleBatchInput,
    retention_days: i64,
) -> Result<usize, IngestError> {
    if !db.brewery_exists(&input.brewery_id).await? {
        return Err(IngestError::UnknownBreweryId(input.brewery_id));
    }

    let now = Utc::now();
    let now_rfc3339 = now.to_rfc3339();
    let cutoff_rfc3339 = (now - Duration::days(retention_days)).to_rfc3339();

    for event in &input.events {
        let vendor_name = event.vendor_name.trim();
        if vendor_name.is_empty() {
            continue;
        }
        let normalized_name = normalize_vendor_name(vendor_name);
        if normalized_name.is_empty() {
            continue;
        }

        let vendor_id = db
            .resolve_or_create_vendor_id(vendor_name, &normalized_name, &now_rfc3339)
            .await?;

        db.upsert_schedule_entry(ScheduleUpsertInput {
            brewery_id: &input.brewery_id,
            vendor_id: &vendor_id,
            start_at: &event.start_at,
            end_at: &event.end_at,
            source: &input.source,
            source_url: event.source_url.as_deref(),
            run_id: &input.run_id,
            now_rfc3339: &now_rfc3339,
        })
        .await?;
    }

    db.supersede_stale_active_entries(
        &input.source,
        &input.brewery_id,
        &input.run_id,
        &now_rfc3339,
    )
    .await?;

    let deleted = db.delete_entries_older_than(&cutoff_rfc3339).await?;

    Ok(deleted)
}

pub fn normalize_vendor_name(name: &str) -> String {
    let lowered = name.trim().to_lowercase();
    let before_at = lowered
        .split_once('@')
        .map_or(lowered.as_str(), |(head, _)| head)
        .trim();
    let cleaned = strip_trailing_noise(before_at);

    cleaned
        .chars()
        .flat_map(char::to_lowercase)
        .filter(char::is_ascii_alphanumeric)
        .collect()
}

fn strip_trailing_noise(input: &str) -> &str {
    const NOISE_SUFFIXES: [&str; 4] = [" debut", " popup", " pop-up", " collaboration"];
    let mut value = input.trim();
    for suffix in NOISE_SUFFIXES {
        if let Some(stripped) = value.strip_suffix(suffix) {
            value = stripped.trim();
        }
    }
    value
}

#[cfg(test)]
mod tests {
    use super::normalize_vendor_name;

    #[test]
    fn normalize_vendor_name_keeps_only_ascii_alnum_lowercase() {
        let normalized = normalize_vendor_name("  Tacos & Beer!  ");
        assert_eq!(normalized, "tacosbeer");
    }

    #[test]
    fn normalize_vendor_name_strips_context_noise() {
        assert_eq!(
            normalize_vendor_name("Tacos Califas debut @ Fair Isle Brewing"),
            "tacoscalifas"
        );
        assert_eq!(normalize_vendor_name("Kaosamai Thai"), "kaosamaithai");
        assert_eq!(
            normalize_vendor_name("Impeckable Chicken"),
            "impeckablechicken"
        );
    }
}
