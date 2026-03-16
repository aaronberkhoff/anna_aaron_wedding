use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RsvpStatus {
    Pending,
    Attending,
    Declined,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DietaryRestriction {
    None,
    Vegetarian,
    Vegan,
    GlutenFree,
    HalalKosher,
    Other(String),
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
    pub dietary_restriction: DietaryRestriction,
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
/// Used to populate the RSVP form on the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestSummary {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub rehearsal_invited: bool,
    /// Current dietary preference stored as a string (e.g. "none", "vegetarian").
    pub dietary: String,
    /// Current RSVP status: "pending", "attending", or "declined".
    /// "pending" means no RSVP has been submitted yet.
    pub rsvp_status: String,
}

impl GuestSummary {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

/// A pre-loaded party member associated with a primary guest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyMember {
    pub id: String,
    pub name: String,
    pub dietary: String,
}

/// Returned by GET /api/guests/lookup — the primary guest plus their party.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestLookup {
    pub guest: GuestSummary,
    pub party_members: Vec<PartyMember>,
}

/// Returned by GET /api/guests/search — minimal info for the name-search dropdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestSearchResult {
    pub id: String,
    pub full_name: String,
}
