use crate::{error::AppError, state::AppState};
use axum::{extract::State, Json};
use shared::models::guest::Guest;

pub async fn list_guests(State(_state): State<AppState>) -> Result<Json<Vec<Guest>>, AppError> {
    // TODO: query guests from SQLite via sqlx
    // Example:
    // let rows = sqlx::query_as!(GuestRow, "SELECT * FROM guests")
    //     .fetch_all(&state.pool)
    //     .await?;
    Ok(Json(vec![]))
}
