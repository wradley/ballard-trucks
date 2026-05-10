use crate::app_state::AppState;
use crate::domain::{
    IngestError, IngestEventInput, IngestScheduleBatchInput, ingest_schedule_batch,
};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum_extra::TypedHeader;
use axum_extra::headers::{Authorization, authorization::Bearer};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;
use tracing::error;
use uuid::Uuid;

const DEFAULT_RETENTION_DAYS: i64 = 7;

#[derive(Deserialize)]
pub struct IngestEventPayload {
    vendor_name: String,
    start_at: String,
    end_at: String,
    source_url: Option<String>,
}

#[derive(Deserialize)]
pub struct IngestScheduleBatchPayload {
    run_id: String,
    source: String,
    brewery_id: String,
    events: Vec<IngestEventPayload>,
}

#[derive(Serialize)]
pub struct IngestResponse {
    ingested_events: usize,
}

#[derive(Debug, Error)]
pub enum IngestApiRejection {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Unknown brewery_id: {0}")]
    UnknownBreweryId(String),
    #[error("Internal server error")]
    Internal,
}

impl IntoResponse for IngestApiRejection {
    fn into_response(self) -> Response {
        let status = match self {
            IngestApiRejection::Unauthorized => StatusCode::UNAUTHORIZED,
            IngestApiRejection::UnknownBreweryId(_) => StatusCode::BAD_REQUEST,
            IngestApiRejection::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

#[axum::debug_handler]
pub async fn post_ingest_schedules(
    State(state): State<AppState>,
    auth: Option<TypedHeader<Authorization<Bearer>>>,
    Json(payload): Json<IngestScheduleBatchPayload>,
) -> Result<Json<IngestResponse>, IngestApiRejection> {
    if !is_authorized(&state, auth) {
        return Err(IngestApiRejection::Unauthorized);
    }

    let events_len = payload.events.len();
    let input = IngestScheduleBatchInput {
        run_id: payload.run_id,
        source: payload.source,
        brewery_id: payload.brewery_id,
        events: payload
            .events
            .into_iter()
            .map(|event| IngestEventInput {
                vendor_name: event.vendor_name,
                start_at: event.start_at,
                end_at: event.end_at,
                source_url: event.source_url,
            })
            .collect(),
    };

    match ingest_schedule_batch(&state.db, input, DEFAULT_RETENTION_DAYS).await {
        Ok(_) => Ok(Json(IngestResponse {
            ingested_events: events_len,
        })),
        Err(IngestError::UnknownBreweryId(id)) => Err(IngestApiRejection::UnknownBreweryId(id)),
        Err(e) => {
            error!("Failed to ingest schedules batch: {e:#}");
            Err(IngestApiRejection::Internal)
        }
    }
}

fn is_authorized(state: &AppState, auth: Option<TypedHeader<Authorization<Bearer>>>) -> bool {
    if let Some(token) = auth.map(|header| header.token().to_string()) {
        if let Ok(key) = Uuid::from_str(&token) {
            return key == state.ingest_api_key;
        }
    }
    false
}
