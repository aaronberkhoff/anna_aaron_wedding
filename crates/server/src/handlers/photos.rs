use crate::{error::AppError, state::AppState};
use axum::{extract::State, Json};
use shared::models::photo::Photo;

pub async fn list_photos(State(_state): State<AppState>) -> Result<Json<Vec<Photo>>, AppError> {
    // TODO: query photos from SQLite
    Ok(Json(vec![]))
}
