use crate::db::Db;
use crate::domain;
use crate::domain::VendorSchedules;
use axum::body::Body;
use axum::extract::rejection::QueryRejection;
use axum::extract::{FromRequestParts, Query, Request, State};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, RequestPartsExt};
use log::{error, warn};
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize)]
pub struct GetSchedulesQueryParams {
    start_hour_utc: String,
    duration_hours: u64,
}

#[derive(Debug)]
pub struct GetSchedulesInput {
    start_hour_utc: jiff::Timestamp,
    duration_hours: u64,
}

#[derive(Debug, Error)]
pub enum GetSchedulesRejection {
    #[error("Invalid start_hour_utc. Must be a valid ISO 8601 timestamp with hour precision")]
    InvalidStartHourUtc,
    #[error("Invalid duration_hours. Must be between 1 and 168 hours")]
    InvalidDurationHours,
    #[error(transparent)]
    QueryRejection(#[from] QueryRejection),
}

impl IntoResponse for GetSchedulesRejection {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(format!("{}", self)))
            .unwrap()
    }
}

impl<S: Sync> FromRequestParts<S> for GetSchedulesInput {
    type Rejection = GetSchedulesRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let params = parts.extract::<Query<GetSchedulesQueryParams>>().await?;
        let start = match format!("{}:00:00Z", params.start_hour_utc).parse::<jiff::Timestamp>() {
            Ok(ts) => ts,
            Err(e) => {
                warn!(
                    "Failed to parse start_hour_utc '{}': {}",
                    params.start_hour_utc, e
                );
                return Err(GetSchedulesRejection::InvalidStartHourUtc);
            }
        };

        if params.duration_hours < 1 || params.duration_hours > 168 {
            warn!(
                "Invalid duration_hours. Must be between 1 and 168 hours: {}",
                params.duration_hours
            );
            return Err(GetSchedulesRejection::InvalidDurationHours);
        }

        Ok(GetSchedulesInput {
            start_hour_utc: start,
            duration_hours: params.duration_hours,
        })
    }
}

/// Handles schedule queries after query extraction/validation by `GetSchedulesInput`.
#[axum::debug_handler]
pub async fn get_schedules(
    State(db): State<Db>,
    params: GetSchedulesInput,
) -> Result<Json<VendorSchedules>, StatusCode> {
    match domain::get_schedules(&db, params.start_hour_utc, params.duration_hours).await {
        Ok(schedules) => Ok(Json(schedules)),
        Err(e) => {
            error!("Failed to retrieve schedules: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_missing_query_params() {
        let (mut parts, _body) = Request::builder()
            .uri("/schedules?start_hour_utc=2024-01-01T00")
            .body(Body::empty())
            .unwrap()
            .into_parts();

        let input = GetSchedulesInput::from_request_parts(&mut parts, &())
            .await
            .unwrap_err();

        assert!(matches!(input, GetSchedulesRejection::QueryRejection(_)));
        assert_eq!(input.into_response().status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_duration_below_min() {
        let (mut parts, _body) = Request::builder()
            .uri("/schedules?start_hour_utc=2024-01-01T00&duration_hours=0")
            .body(Body::empty())
            .unwrap()
            .into_parts();

        let input = GetSchedulesInput::from_request_parts(&mut parts, &())
            .await
            .unwrap_err();

        assert!(matches!(input, GetSchedulesRejection::InvalidDurationHours));
        assert_eq!(input.into_response().status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_duration_above_max() {
        let (mut parts, _body) = Request::builder()
            .uri("/schedules?start_hour_utc=2024-01-01T00&duration_hours=169")
            .body(Body::empty())
            .unwrap()
            .into_parts();

        let input = GetSchedulesInput::from_request_parts(&mut parts, &())
            .await
            .unwrap_err();

        assert!(matches!(input, GetSchedulesRejection::InvalidDurationHours));
        assert_eq!(input.into_response().status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_invalid_start_hour() {
        let (mut parts, _body) = Request::builder()
            .uri("/schedules?start_hour_utc=2024-01-01&duration_hours=1")
            .body(Body::empty())
            .unwrap()
            .into_parts();

        let input = GetSchedulesInput::from_request_parts(&mut parts, &())
            .await
            .unwrap_err();

        assert!(matches!(input, GetSchedulesRejection::InvalidStartHourUtc));
        assert_eq!(input.into_response().status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_valid() {
        let (mut parts, _body) = Request::builder()
            .uri("/schedules?start_hour_utc=2024-01-01T12&duration_hours=12")
            .body(Body::empty())
            .unwrap()
            .into_parts();

        let input = GetSchedulesInput::from_request_parts(&mut parts, &())
            .await
            .unwrap();

        assert_eq!(
            input.start_hour_utc,
            "2024-01-01T12:00:00Z".parse().unwrap()
        );
        assert_eq!(input.duration_hours, 12);
    }
}
