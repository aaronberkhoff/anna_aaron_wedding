use crate::{error::AppError, state::AppState};
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use shared::models::guest::{GuestLookup, GuestSearchResult, GuestSummary, PartyMember};

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

/// GET /api/guests/lookup?code=XXXX
/// GET /api/guests/lookup?id=UUID
///
/// Looks up a primary guest by their invite code or UUID and returns their
/// info plus any pre-loaded party members. Used by the RSVP form Step 1.
pub async fn lookup_guest(
    State(state): State<AppState>,
    Query(params): Query<LookupParams>,
) -> Result<Json<GuestLookup>, AppError> {
    // Each sqlx::query! arm produces a different anonymous type, so we map
    // each result into GuestSummary before unifying them.
    let (guest_id, guest_summary) = if let Some(code) = params.code.as_deref() {
        let c = code.trim().to_string();
        let row = sqlx::query!(
            "SELECT id, first_name, last_name, rehearsal_invited, dietary, rsvp_status
             FROM guests WHERE invite_code = ? LIMIT 1",
            c
        )
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;
        let id = row.id.expect("guests.id is primary key, never null");
        (
            id.clone(),
            GuestSummary {
                id,
                first_name: row.first_name,
                last_name: row.last_name,
                rehearsal_invited: row.rehearsal_invited != 0,
                dietary: row.dietary,
                rsvp_status: row.rsvp_status,
            },
        )
    } else if let Some(guest_id_str) = params.id.as_deref() {
        let i = guest_id_str.trim().to_string();
        let row = sqlx::query!(
            "SELECT id, first_name, last_name, rehearsal_invited, dietary, rsvp_status
             FROM guests WHERE id = ? LIMIT 1",
            i
        )
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;
        let id = row.id.expect("guests.id is primary key, never null");
        (
            id.clone(),
            GuestSummary {
                id,
                first_name: row.first_name,
                last_name: row.last_name,
                rehearsal_invited: row.rehearsal_invited != 0,
                dietary: row.dietary,
                rsvp_status: row.rsvp_status,
            },
        )
    } else {
        return Err(AppError::Validation("provide code or id".to_string()));
    };

    let party_rows = sqlx::query!(
        "SELECT id, name, dietary FROM party_members WHERE guest_id = ?",
        guest_id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(GuestLookup {
        guest: guest_summary,
        party_members: party_rows
            .into_iter()
            .map(|r| PartyMember {
                id: r.id.expect("party_members.id is primary key, never null"),
                name: r.name,
                dietary: r.dietary,
            })
            .collect(),
    }))
}

/// GET /api/guests/search?q=NAME
///
/// Fuzzy name search for the fallback dropdown when a guest doesn't have
/// their invite code handy. Returns up to 20 matches.
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

/// GET /api/guests — full guest list (admin use, not yet implemented).
pub async fn list_guests(
    State(_state): State<AppState>,
) -> Result<Json<Vec<shared::models::guest::Guest>>, AppError> {
    Ok(Json(vec![]))
}
