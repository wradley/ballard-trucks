use crate::db::{ScheduleRepo, ScheduleRow};
use anyhow::anyhow;
use serde::Serialize;
use std::ops::Bound;
use uuid::Uuid;

pub async fn get_schedules<R: ScheduleRepo>(
    db: &R,
    start: jiff::Timestamp,
    duration_hours: u64,
) -> anyhow::Result<VendorSchedules> {
    let schedules = db.get_schedules_within(start, duration_hours).await?;
    let daily_trucks = VendorSchedules::try_from(schedules).map_err(|e| anyhow!(e))?;
    Ok(daily_trucks)
}

#[derive(Serialize)]
pub struct Schedule {
    brewery_name: String,
    brewery_id: Uuid,
    vendor_name: String,
    vendor_id: Uuid,
    start_at: String,
    end_at: String,
    updated_at: String,
}

impl TryFrom<ScheduleRow> for Schedule {
    type Error = &'static str;

    fn try_from(value: ScheduleRow) -> Result<Self, Self::Error> {
        if value.brewery_name.is_empty() {
            return Err("brewery name is empty");
        }

        if value.food_vendor_name.is_empty() {
            return Err("vendor name is empty");
        }

        let open_hours = value.open_hours;

        let start_at = match open_hours.start {
            Bound::Included(ts) | Bound::Excluded(ts) => ts.to_rfc3339(),
            Bound::Unbounded => return Err("schedule open_hours start is unbounded"),
        };

        let end_at = match open_hours.end {
            Bound::Included(ts) | Bound::Excluded(ts) => ts.to_rfc3339(),
            Bound::Unbounded => return Err("schedule open_hours end is unbounded"),
        };

        Ok(Schedule {
            brewery_name: value.brewery_name,
            brewery_id: value.brewery_id,
            vendor_name: value.food_vendor_name,
            vendor_id: value.food_vendor_id,
            start_at,
            end_at,
            updated_at: value.updated_at.to_rfc3339(),
        })
    }
}

#[derive(Serialize)]
pub struct VendorSchedules {
    schedules: Vec<Schedule>,
}

impl TryFrom<Vec<ScheduleRow>> for VendorSchedules {
    type Error = &'static str;

    fn try_from(value: Vec<ScheduleRow>) -> Result<Self, Self::Error> {
        let schedules = value
            .into_iter()
            .map(Schedule::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(VendorSchedules { schedules })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::types::PgRange;
    use sqlx::types::chrono::{TimeZone, Utc};
    use std::sync::Mutex;

    struct MockScheduleRepo {
        rows: Mutex<Option<Vec<ScheduleRow>>>,
    }

    impl ScheduleRepo for MockScheduleRepo {
        async fn get_schedules_within(
            &self,
            _start: jiff::Timestamp,
            _duration_hours: u64,
        ) -> anyhow::Result<Vec<ScheduleRow>> {
            Ok(self.rows.lock().expect("lock poisoned").take().unwrap_or_default())
        }
    }

    fn sample_schedule_row(brewery_name: &str, vendor_name: &str) -> ScheduleRow {
        let start = Utc
            .with_ymd_and_hms(2026, 2, 1, 17, 0, 0)
            .single()
            .expect("valid datetime");
        let end = Utc
            .with_ymd_and_hms(2026, 2, 2, 3, 0, 0)
            .single()
            .expect("valid datetime");
        let updated_at = Utc
            .with_ymd_and_hms(2026, 2, 1, 18, 30, 0)
            .single()
            .expect("valid datetime");

        ScheduleRow {
            id: Uuid::nil(),
            brewery_id: Uuid::nil(),
            brewery_name: brewery_name.to_string(),
            food_vendor_id: Uuid::nil(),
            food_vendor_name: vendor_name.to_string(),
            open_hours: PgRange {
                start: Bound::Included(start),
                end: Bound::Excluded(end),
            },
            source: "seed".to_string(),
            updated_at,
        }
    }

    #[tokio::test]
    async fn get_schedules_maps_timestamps_and_names() {
        let repo = MockScheduleRepo {
            rows: Mutex::new(Some(vec![sample_schedule_row(
                "Stoup Brewing",
                "El Pirata Tortas Y Burritos",
            )])),
        };

        let start = "2026-02-01T17:00:00Z"
            .parse::<jiff::Timestamp>()
            .expect("valid timestamp");
        let result = get_schedules(&repo, start, 24).await.expect("valid result");

        assert_eq!(result.schedules.len(), 1);
        let first = &result.schedules[0];
        assert_eq!(first.brewery_name, "Stoup Brewing");
        assert_eq!(first.vendor_name, "El Pirata Tortas Y Burritos");
        assert_eq!(first.start_at, "2026-02-01T17:00:00+00:00");
        assert_eq!(first.end_at, "2026-02-02T03:00:00+00:00");
        assert_eq!(first.updated_at, "2026-02-01T18:30:00+00:00");
    }

    #[test]
    fn schedule_try_from_rejects_empty_vendor_name() {
        let row = sample_schedule_row("Stoup Brewing", "");
        let result = Schedule::try_from(row);
        assert!(result.is_err());
    }
}
