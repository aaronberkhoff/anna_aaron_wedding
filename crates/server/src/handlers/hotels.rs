use crate::{error::AppError, state::AppState};
use axum::{extract::State, Json};
use shared::models::hotel::HotelRoom;

pub async fn list_hotels(State(_state): State<AppState>) -> Result<Json<Vec<HotelRoom>>, AppError> {
    // TODO: query hotel_rooms from SQLite
    Ok(Json(vec![]))
}
