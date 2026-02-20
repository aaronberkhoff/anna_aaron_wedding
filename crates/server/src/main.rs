use anyhow::Result;
use axum::http::HeaderValue;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod config;
mod db;
mod error;
mod handlers;
mod state;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env in development (silently no-ops if the file is absent).
    let _ = dotenvy::dotenv();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::from_env()?;
    let pool = db::create_pool(&config.database_url).await?;
    db::run_migrations(&pool).await?;

    let dist_dir = config.dist_dir.clone();
    let bind_addr = config.bind_addr.clone();
    let state = state::AppState {
        pool,
        config: Arc::new(config),
    };

    // In dev, Trunk runs on :3000 and the API on :8080 — different origins.
    // In production both are served from the same origin, so CORS is a no-op.
    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:3000"
                .parse::<HeaderValue>()
                .expect("valid origin"),
            "http://127.0.0.1:3000"
                .parse::<HeaderValue>()
                .expect("valid origin"),
        ])
        .allow_methods(Any)
        .allow_headers(Any);

    let app = handlers::router(state)
        // Fallback: serve Leptos SPA. Any path not matched by API routes
        // returns index.html so leptos_router handles client-side routing.
        .fallback_service(
            ServeDir::new(&dist_dir)
                .not_found_service(ServeFile::new(format!("{}/index.html", dist_dir))),
        )
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http());

    tracing::info!("Listening on {}", bind_addr);
    let listener = TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
