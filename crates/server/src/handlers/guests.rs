use crate::{error::AppError, state::AppState};
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use shared::models::guest::{GuestLookup, GuestSearchResult, GuestSummary};

#[derive(Deserialize)]
pub struct LookupParams {
    /// Look up by 4-digit invite code.
    pub code: Option<String>,
    /// Look up by guest UUID (used after name-search selects a guest).
    pub id: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String,
}

fn map_summary(
    id: Option<String>,
    first_name: String,
    last_name: String,
    rehearsal_invited: i64,
    rsvp_status: String,
) -> GuestSummary {
    GuestSummary {
        id: id.expect("guests.id is primary key, never null"),
        first_name,
        last_name,
        rehearsal_invited: rehearsal_invited != 0,
        rsvp_status,
    }
}

/// GET /api/guests/lookup?code=XXXX
/// GET /api/guests/lookup?id=UUID
///
/// Returns all guests sharing the same invite code. Used by the RSVP form Step 1.
/// When looking up by id, finds that guest's invite code then returns the full party.
pub async fn lookup_guest(
    State(state): State<AppState>,
    Query(params): Query<LookupParams>,
) -> Result<Json<GuestLookup>, AppError> {
    let members: Vec<GuestSummary> = if let Some(code) = params.code.as_deref() {
        let c = code.trim().to_string();
        let rows = sqlx::query!(
            "SELECT id, first_name, last_name, rehearsal_invited, rsvp_status
             FROM guests WHERE invite_code = ? ORDER BY rowid",
            c
        )
        .fetch_all(&state.pool)
        .await?;

        if rows.is_empty() {
            return Err(AppError::NotFound);
        }

        rows.into_iter()
            .map(|r| map_summary(r.id, r.first_name, r.last_name, r.rehearsal_invited, r.rsvp_status))
            .collect()
    } else if let Some(guest_id_str) = params.id.as_deref() {
        let i = guest_id_str.trim().to_string();

        // Find this guest's invite_code, then return everyone in that party.
        let invite_code = sqlx::query!(
            "SELECT invite_code FROM guests WHERE id = ? LIMIT 1",
            i
        )
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?
        .invite_code;

        if let Some(code) = invite_code {
            let rows = sqlx::query!(
                "SELECT id, first_name, last_name, rehearsal_invited, rsvp_status
                 FROM guests WHERE invite_code = ? ORDER BY rowid",
                code
            )
            .fetch_all(&state.pool)
            .await?;

            rows.into_iter()
                .map(|r| map_summary(r.id, r.first_name, r.last_name, r.rehearsal_invited, r.rsvp_status))
                .collect()
        } else {
            // No invite code — return just this one guest.
            let row = sqlx::query!(
                "SELECT id, first_name, last_name, rehearsal_invited, rsvp_status
                 FROM guests WHERE id = ? LIMIT 1",
                i
            )
            .fetch_optional(&state.pool)
            .await?
            .ok_or(AppError::NotFound)?;

            vec![map_summary(row.id, row.first_name, row.last_name, row.rehearsal_invited, row.rsvp_status)]
        }
    } else {
        return Err(AppError::Validation("provide code or id".to_string()));
    };

    Ok(Json(GuestLookup { members }))
}

/// GET /api/guests/search?q=NAME
///
/// Fuzzy name search — returns up to 20 matches. Used as fallback when a guest
/// doesn't have their invite code.
pub async fn search_guests(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<GuestSearchResult>>, AppError> {
    let pattern = format!("%{}%", params.q.to_lowercase());

    let rows = sqlx::query!(
        "SELECT id, first_name, last_name
         FROM guests
         WHERE lower(first_name || ' ' || last_name) LIKE ?
         LIMIT 20",
        pattern
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|r| GuestSearchResult {
                id: r.id.expect("guests.id is primary key, never null"),
                full_name: format!("{} {}", r.first_name, r.last_name),
            })
            .collect(),
    ))
}

/// GET /api/guests — full guest list (admin use).
pub async fn list_guests(
    State(_state): State<AppState>,
) -> Result<Json<Vec<shared::models::guest::Guest>>, AppError> {
    Ok(Json(vec![]))
}
