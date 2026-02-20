use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotelRoom {
    pub id: Uuid,
    pub hotel_name: String,
    pub room_type: String,
    pub capacity: u32,
    pub price_usd: Option<f64>,
    pub block_code: Option<String>,
    pub booking_url: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotelBooking {
    pub id: Uuid,
    pub guest_id: Option<Uuid>,
    pub room_id: Uuid,
    pub check_in: Option<String>,
    pub check_out: Option<String>,
}
