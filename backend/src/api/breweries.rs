use crate::app_state::AppState;
use crate::domain;
use crate::domain::Breweries;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use tracing::error;

#[axum::debug_handler]
pub async fn get_breweries(State(state): State<AppState>) -> Result<Json<Breweries>, StatusCode> {
    match domain::get_breweries(&state.db).await {
        Ok(breweries) => Ok(Json(breweries)),
        Err(e) => {
            error!("Failed to retrieve breweries: {e:#}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
