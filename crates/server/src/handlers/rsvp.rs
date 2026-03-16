use crate::{error::AppError, mail, state::AppState};
use axum::{extract::State, Json};
use shared::models::rsvp::{RsvpRecord, RsvpRequest, RsvpResponse};
use uuid::Uuid;

pub async fn submit_rsvp(
    State(state): State<AppState>,
    Json(payload): Json<RsvpRequest>,
) -> Result<Json<RsvpResponse>, AppError> {
    // Validate that the guest_id exists and fetch display info for the response.
    let guest = sqlx::query!(
        "SELECT first_name, last_name, email, invite_code FROM guests WHERE id = ? LIMIT 1",
        payload.guest_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::Validation("guest not found".to_string()))?;

    let rsvp_status = if payload.attending_reception {
        "attending"
    } else {
        "declined"
    };

    // Update primary guest's RSVP status and dietary preference.
    sqlx::query!(
        "UPDATE guests
         SET rsvp_status = ?, dietary = ?, updated_at = datetime('now')
         WHERE id = ?",
        rsvp_status,
        payload.dietary,
        payload.guest_id
    )
    .execute(&state.pool)
    .await?;

    // Update each party member's attendance and dietary preference.
    for pm in &payload.party_members {
        sqlx::query!(
            "UPDATE party_members
             SET dietary = ?, attending_reception = ?, attending_rehearsal = ?
             WHERE id = ?",
            pm.dietary,
            pm.attending_reception,
            pm.attending_rehearsal,
            pm.id
        )
        .execute(&state.pool)
        .await?;
    }

    // Serialize the known_guests list as a JSON array for storage.
    let known_guests_json = serde_json::to_string(&payload.known_guests)
        .unwrap_or_else(|_| "[]".to_string());

    // Remove any existing RSVP for this guest so re-submissions replace rather
    // than duplicate. The guest row's rsvp_status is already updated above.
    sqlx::query!("DELETE FROM rsvps WHERE guest_id = ?", payload.guest_id)
        .execute(&state.pool)
        .await?;

    let rsvp_id = Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO rsvps
             (id, guest_id, attending_reception, attending_rehearsal,
              known_guests, song_request, message)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
        rsvp_id,
        payload.guest_id,
        payload.attending_reception,
        payload.attending_rehearsal,
        known_guests_json,
        payload.song_request,
        payload.message
    )
    .execute(&state.pool)
    .await?;

    // Send email notification — fire-and-forget, never fail the request.
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
) -> Result<Json<Vec<RsvpRecord>>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT r.id, g.first_name, g.last_name, g.email, g.dietary,
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
            dietary_restriction: row.dietary,
            known_guests: row.known_guests,
            song_request: row.song_request,
            message: row.message,
            submitted_at: row.submitted_at,
        })
        .collect();

    Ok(Json(records))
}
