use crate::{error::AppError, mail, state::AppState};
use axum::{extract::State, Json};
use shared::models::{
    guest::DietaryRestriction,
    rsvp::{RsvpRecord, RsvpRequest, RsvpResponse},
};
use uuid::Uuid;

fn dietary_to_str(d: &DietaryRestriction) -> String {
    match d {
        DietaryRestriction::None => "none".to_string(),
        DietaryRestriction::Vegetarian => "vegetarian".to_string(),
        DietaryRestriction::Vegan => "vegan".to_string(),
        DietaryRestriction::GlutenFree => "gluten_free".to_string(),
        DietaryRestriction::HalalKosher => "halal_kosher".to_string(),
        DietaryRestriction::Other(s) => format!("other:{s}"),
    }
}

pub async fn submit_rsvp(
    State(state): State<AppState>,
    Json(payload): Json<RsvpRequest>,
) -> Result<Json<RsvpResponse>, AppError> {
    if payload.email.is_empty() {
        return Err(AppError::Validation("email is required".to_string()));
    }

    let dietary = dietary_to_str(&payload.dietary_restriction);
    let rsvp_status = if payload.attending { "attending" } else { "declined" };

    // Upsert guest: update if email already exists, otherwise insert.
    let existing = sqlx::query!(
        "SELECT id FROM guests WHERE email = ? LIMIT 1",
        payload.email
    )
    .fetch_optional(&state.pool)
    .await?;

    let guest_id = if let Some(row) = existing {
        sqlx::query!(
            "UPDATE guests
             SET first_name = ?, last_name = ?, rsvp_status = ?,
                 dietary = ?, plus_one = ?, plus_one_name = ?,
                 updated_at = datetime('now')
             WHERE id = ?",
            payload.first_name,
            payload.last_name,
            rsvp_status,
            dietary,
            payload.plus_one,
            payload.plus_one_name,
            row.id
        )
        .execute(&state.pool)
        .await?;
        row.id.expect("guest id is primary key, never null")
    } else {
        let id = Uuid::new_v4().to_string();
        sqlx::query!(
            "INSERT INTO guests
                (id, first_name, last_name, email, rsvp_status, dietary, plus_one, plus_one_name)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            id,
            payload.first_name,
            payload.last_name,
            payload.email,
            rsvp_status,
            dietary,
            payload.plus_one,
            payload.plus_one_name
        )
        .execute(&state.pool)
        .await?;
        id
    };

    let rsvp_id = Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO rsvps (id, guest_id, song_request, message) VALUES (?, ?, ?, ?)",
        rsvp_id,
        guest_id,
        payload.song_request,
        payload.message
    )
    .execute(&state.pool)
    .await?;

    // Send email notification — fire-and-forget, never fail the request.
    if let Some(smtp) = &state.config.smtp {
        mail::send_rsvp_notification(smtp, &payload).await;
    }

    Ok(Json(RsvpResponse {
        success: true,
        message: format!("Thank you, {}! Your RSVP has been received.", payload.first_name),
    }))
}

pub async fn list_rsvps(
    State(state): State<AppState>,
) -> Result<Json<Vec<RsvpRecord>>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT r.id, g.first_name, g.last_name, g.email,
                  g.rsvp_status, g.plus_one, g.plus_one_name,
                  g.dietary, r.song_request, r.message, r.submitted_at
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
            attending: row.rsvp_status == "attending",
            plus_one: row.plus_one != 0,
            plus_one_name: row.plus_one_name,
            dietary_restriction: row.dietary,
            song_request: row.song_request,
            message: row.message,
            submitted_at: row.submitted_at,
        })
        .collect();

    Ok(Json(records))
}
