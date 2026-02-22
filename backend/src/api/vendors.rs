use crate::app_state::AppState;
use crate::domain;
use crate::domain::Vendors;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use tracing::error;

#[axum::debug_handler]
pub async fn get_vendors(State(state): State<AppState>) -> Result<Json<Vendors>, StatusCode> {
    match domain::get_vendors(&state.db).await {
        Ok(vendors) => Ok(Json(vendors)),
        Err(e) => {
            error!("Failed to retrieve vendors: {e:#}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
