use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeatingTable {
    pub id: Uuid,
    pub name: String,
    pub capacity: u32,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableAssignment {
    pub guest_id: Uuid,
    pub table_id: Uuid,
    pub seat_number: Option<u32>,
}

/// Full seating chart: tables with their assigned guests.
/// Used by the D3/Sigma visualization on the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeatingChart {
    pub tables: Vec<TableWithGuests>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableWithGuests {
    pub table: SeatingTable,
    pub guest_ids: Vec<Uuid>,
}
