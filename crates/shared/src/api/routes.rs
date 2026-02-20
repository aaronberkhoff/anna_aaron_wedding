// Centralized API route path constants.
// Both the server (Axum router) and frontend (fetch calls) import from here.
// Keeping paths in one place prevents frontend/backend string drift.

pub const HEALTH: &str = "/health";

pub const GUESTS_LIST: &str = "/api/guests";
pub const GUEST_BY_ID: &str = "/api/guests/:id";

pub const RSVP_SUBMIT: &str = "/api/rsvp";

pub const TABLES_LIST: &str = "/api/tables";
pub const TABLE_BY_ID: &str = "/api/tables/:id";
pub const SEATING_CHART: &str = "/api/tables/chart";

pub const HOTELS_LIST: &str = "/api/hotels";

pub const PHOTOS_LIST: &str = "/api/photos";
pub const PHOTOS_UPLOAD: &str = "/api/photos/upload";
