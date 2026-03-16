use serde::{Deserialize, Serialize};

/// RSVP attendance for a single party member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyMemberRsvp {
    /// References party_members.id
    pub id: String,
    /// Display name — used in email notifications so the record is human-readable.
    pub name: String,
    pub attending_reception: bool,
    /// None if the party member is not invited to the rehearsal dinner.
    pub attending_rehearsal: Option<bool>,
    /// Dietary preference string (e.g. "none", "vegetarian").
    pub dietary: String,
}

/// Submitted by the frontend to POST /api/rsvp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsvpRequest {
    /// References guests.id — set during the lookup step.
    pub guest_id: String,
    pub attending_reception: bool,
    /// None if the primary guest is not invited to the rehearsal dinner.
    pub attending_rehearsal: Option<bool>,
    /// Dietary preference string for the primary guest.
    pub dietary: String,
    pub party_members: Vec<PartyMemberRsvp>,
    /// Names of other guests they'd like to be seated near (for seating chart).
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
    pub dietary_restriction: String,
    pub known_guests: Option<String>,
    pub song_request: Option<String>,
    pub message: Option<String>,
    pub submitted_at: String,
}
