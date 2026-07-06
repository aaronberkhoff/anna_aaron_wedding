use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RsvpStatus {
    Pending,
    Attending,
    Declined,
}

/// Full guest record (used internally and by the admin guest list).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guest {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub rsvp_status: RsvpStatus,
    pub invite_code: Option<String>,
    pub rehearsal_invited: bool,
    pub invite_sent: bool,
    pub notes: Option<String>,
}

impl Guest {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

/// Lightweight guest info returned by the invite-code lookup and name-search endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestSummary {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub rehearsal_invited: bool,
    /// "pending", "attending", or "declined"
    pub rsvp_status: String,
}

impl GuestSummary {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

/// Returned by GET /api/guests/lookup — all guests sharing the same invite code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestLookup {
    pub members: Vec<GuestSummary>,
}

/// Returned by GET /api/guests/search — minimal info for the name-search dropdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestSearchResult {
    pub id: String,
    pub full_name: String,
}
