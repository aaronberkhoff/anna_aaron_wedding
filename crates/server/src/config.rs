use anyhow::{Context, Result};

pub struct Config {
    pub database_url: String,
    pub dist_dir: String,
    pub bind_addr: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Config {
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL must be set (e.g. sqlite://wedding.db)")?,
            dist_dir: std::env::var("DIST_DIR").unwrap_or_else(|_| "site/dist".to_string()),
            bind_addr: std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
        })
    }
}
