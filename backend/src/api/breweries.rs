use crate::db::Db;
use crate::domain;
use crate::domain::Breweries;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use log::error;

#[axum::debug_handler]
pub async fn get_breweries(State(db): State<Db>) -> Result<Json<Breweries>, StatusCode> {
    match domain::get_breweries(&db).await {
        Ok(breweries) => Ok(Json(breweries)),
        Err(e) => {
            error!("Failed to retrieve breweries: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
