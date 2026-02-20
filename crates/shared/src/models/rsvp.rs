use super::guest::DietaryRestriction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsvpRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub attending: bool,
    pub plus_one: bool,
    pub plus_one_name: Option<String>,
    pub dietary_restriction: DietaryRestriction,
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
    pub attending: bool,
    pub plus_one: bool,
    pub plus_one_name: Option<String>,
    pub dietary_restriction: String,
    pub song_request: Option<String>,
    pub message: Option<String>,
    pub submitted_at: String,
}
