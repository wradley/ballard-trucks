use crate::db::Db;
use crate::domain;
use crate::domain::Vendors;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use log::error;

#[axum::debug_handler]
pub async fn get_vendors(State(db): State<Db>) -> Result<Json<Vendors>, StatusCode> {
    match domain::get_vendors(&db).await {
        Ok(vendors) => Ok(Json(vendors)),
        Err(e) => {
            error!("Failed to retrieve vendors: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
