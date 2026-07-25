use crate::{error::AppError, mail, state::AppState};
use axum::{extract::State, http::HeaderMap, Json};
use shared::models::rsvp::{RsvpRecord, RsvpRequest, RsvpResponse};
use uuid::Uuid;

pub async fn submit_rsvp(
    State(state): State<AppState>,
    Json(payload): Json<RsvpRequest>,
) -> Result<Json<RsvpResponse>, AppError> {
    if payload.party.is_empty() {
        return Err(AppError::Validation("party is empty".to_string()));
    }

    // Fetch the first guest for the response message and email notification.
    let first_id = &payload.party[0].guest_id;
    let guest = sqlx::query!(
        "SELECT first_name, last_name, email, invite_code FROM guests WHERE id = ? LIMIT 1",
        first_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::Validation("guest not found".to_string()))?;

    let known_guests_json =
        serde_json::to_string(&payload.known_guests).unwrap_or_else(|_| "[]".to_string());

    // Update each guest's status and upsert their RSVP record.
    for g in &payload.party {
        let rsvp_status = if g.attending_reception {
            "attending"
        } else {
            "declined"
        };

        sqlx::query!(
            "UPDATE guests SET rsvp_status = ?, updated_at = datetime('now') WHERE id = ?",
            rsvp_status,
            g.guest_id
        )
        .execute(&state.pool)
        .await?;

        sqlx::query!("DELETE FROM rsvps WHERE guest_id = ?", g.guest_id)
            .execute(&state.pool)
            .await?;

        let rsvp_id = Uuid::new_v4().to_string();
        sqlx::query!(
            "INSERT INTO rsvps
                 (id, guest_id, attending_reception, attending_rehearsal,
                  known_guests, song_request, message)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            rsvp_id,
            g.guest_id,
            g.attending_reception,
            g.attending_rehearsal,
            known_guests_json,
            payload.song_request,
            payload.message
        )
        .execute(&state.pool)
        .await?;
    }

    if let Some(smtp) = &state.config.smtp {
        let guest_name = format!("{} {}", guest.first_name, guest.last_name);
        mail::send_rsvp_notification(
            smtp,
            &guest_name,
            guest.email.as_deref(),
            guest.invite_code.as_deref(),
            &payload,
        )
        .await;
    }

    Ok(Json(RsvpResponse {
        success: true,
        message: format!(
            "Thank you, {}! Your RSVP has been received.",
            guest.first_name
        ),
    }))
}

pub async fn list_rsvps(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<RsvpRecord>>, AppError> {
    let provided = headers.get("x-admin-key").and_then(|v| v.to_str().ok());
    match (&state.config.admin_key, provided) {
        (Some(expected), Some(provided)) if expected == provided => {}
        _ => return Err(AppError::Unauthorized),
    }

    let rows = sqlx::query!(
        r#"SELECT r.id, g.first_name, g.last_name, g.email,
                  r.attending_reception, r.attending_rehearsal,
                  r.known_guests, r.song_request, r.message, r.submitted_at
           FROM rsvps r
           JOIN guests g ON r.guest_id = g.id
           ORDER BY r.submitted_at DESC"#
    )
    .fetch_all(&state.pool)
    .await?;

    let records = rows
        .into_iter()
        .map(|row| RsvpRecord {
            id: row.id.expect("rsvp id is primary key, never null"),
            first_name: row.first_name,
            last_name: row.last_name,
            email: row.email,
            attending_reception: row.attending_reception.map(|v| v != 0),
            attending_rehearsal: row.attending_rehearsal.map(|v| v != 0),
            known_guests: row.known_guests,
            song_request: row.song_request,
            message: row.message,
            submitted_at: row.submitted_at,
        })
        .collect();

    Ok(Json(records))
}
