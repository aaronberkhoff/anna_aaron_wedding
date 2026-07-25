use anyhow::{Context, Result};

pub struct Config {
    pub database_url: String,
    pub dist_dir: String,
    pub bind_addr: String,
    pub smtp: Option<SmtpConfig>,
    pub admin_key: Option<String>,
}

pub struct SmtpConfig {
    pub from: String,
    pub to: Vec<String>,
    pub username: String,
    pub password: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let smtp = match (
            std::env::var("SMTP_FROM"),
            std::env::var("SMTP_TO"),
            std::env::var("SMTP_USERNAME"),
            std::env::var("SMTP_PASSWORD"),
        ) {
            (Ok(from), Ok(to), Ok(username), Ok(password)) => Some(SmtpConfig {
                from,
                to: to.split(',').map(|s| s.trim().to_string()).collect(),
                username,
                password,
            }),
            _ => {
                tracing::warn!("SMTP_* env vars not set — RSVP email notifications disabled");
                None
            }
        };

        let admin_key = std::env::var("ADMIN_API_KEY").ok();
        if admin_key.is_none() {
            tracing::warn!("ADMIN_API_KEY not set — /api/rsvps is disabled");
        }

        Ok(Config {
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL must be set (e.g. sqlite://wedding.db)")?,
            dist_dir: std::env::var("DIST_DIR").unwrap_or_else(|_| "site/dist".to_string()),
            bind_addr: std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            smtp,
            admin_key,
        })
    }
}
