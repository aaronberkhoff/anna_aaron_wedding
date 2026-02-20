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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guest {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub rsvp_status: RsvpStatus,
    pub dietary_restriction: DietaryRestriction,
    pub plus_one: bool,
    pub plus_one_name: Option<String>,
    pub invite_sent: bool,
    pub notes: Option<String>,
}

impl Guest {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}
