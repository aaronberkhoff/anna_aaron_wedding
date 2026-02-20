pub mod guests;
pub mod health;
pub mod hotels;
pub mod photos;
pub mod rsvp;
pub mod tables;

use crate::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

/// Assemble the full API router with all routes and shared state.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health_check))
        .route("/api/guests", get(guests::list_guests))
        .route("/api/rsvp", post(rsvp::submit_rsvp))
        .route("/api/rsvps", get(rsvp::list_rsvps))
        .route("/api/tables", get(tables::list_tables))
        .route("/api/tables/chart", get(tables::seating_chart))
        .route("/api/hotels", get(hotels::list_hotels))
        .route("/api/photos", get(photos::list_photos))
        .with_state(state)
}
