use crate::{error::AppError, state::AppState};
use axum::{extract::State, Json};
use shared::models::table::{SeatingChart, SeatingTable};

pub async fn list_tables(
    State(_state): State<AppState>,
) -> Result<Json<Vec<SeatingTable>>, AppError> {
    // TODO: query seating_tables from SQLite
    Ok(Json(vec![]))
}

/// Returns the full seating chart — tables with their assigned guests.
/// This is the primary data source for the D3/Sigma visualization.
pub async fn seating_chart(State(_state): State<AppState>) -> Result<Json<SeatingChart>, AppError> {
    // TODO: join seating_tables + table_assignments + guests
    Ok(Json(SeatingChart { tables: vec![] }))
}
