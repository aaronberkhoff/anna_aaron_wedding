use crate::config::Config;
use sqlx::SqlitePool;
use std::sync::Arc;

/// Shared application state injected into every Axum handler.
/// Wrapped in Arc so it clones cheaply across requests.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub config: Arc<Config>,
}
