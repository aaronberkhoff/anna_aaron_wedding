use anyhow::Result;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;
use std::str::FromStr;

pub async fn create_pool(database_url: &str) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(database_url)?
        // Create the database file if it doesn't exist.
        .create_if_missing(true)
        // Enable WAL mode for better concurrent read performance.
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        // Foreign key enforcement is off by default in SQLite.
        .foreign_keys(true);

    let pool = SqlitePool::connect_with(options).await?;
    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    tracing::info!("Database migrations applied successfully");
    Ok(())
}
