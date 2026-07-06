use serde::{Deserialize, Serialize};

/// RSVP data for one person in the party.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestRsvp {
    pub guest_id: String,
    /// Full name — used in email notifications.
    pub name: String,
    pub attending_reception: bool,
    /// None if not invited to the rehearsal dinner.
    pub attending_rehearsal: Option<bool>,
}

/// Submitted by the frontend to POST /api/rsvp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsvpRequest {
    /// One entry per person in the party.
    pub party: Vec<GuestRsvp>,
    pub known_guests: Vec<String>,
    pub song_request: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsvpResponse {
    pub success: bool,
    pub message: String,
}

/// Returned by GET /api/rsvps — joins rsvp + guest rows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsvpRecord {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub attending_reception: Option<bool>,
    pub attending_rehearsal: Option<bool>,
    pub known_guests: Option<String>,
    pub song_request: Option<String>,
    pub message: Option<String>,
    pub submitted_at: String,
}
