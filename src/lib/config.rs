// src/lib/config.rs

// dependencies
use crate::email::EmailConfig;
use anyhow::{Result, anyhow};
use std::env;

// struct type to represent the application configuration
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub jwt_secret: String,
    pub templates_dir: String,
    /// Deprecated: Bootstrap has been removed. Kept for backwards compatibility.
    pub external_stylesheet: String,
    /// Deprecated: Bootstrap has been removed. Kept for backwards compatibility.
    pub override_stylesheet: String,
    pub app_version: String,
    pub allowed_origins: Vec<String>,
    pub email: EmailConfig,
}

impl AppConfig {
    /// Load application configuration from environment variables.
    ///
    /// Required environment variables:
    /// - `JWT_SECRET`: Secret key for JWT token signing
    /// - `TEMPLATES_DIR`: Glob pattern for template directory (e.g., "templates/**/*")
    ///
    /// Optional environment variables:
    /// - `ALLOWED_ORIGINS`: Comma-separated list of allowed CORS origins
    /// - `MAILTRAP_API_TOKEN`: Mailtrap API token for email sending
    /// - `MAILTRAP_SANDBOX_INBOX_ID`: Mailtrap sandbox inbox ID (for development)
    /// - `MAILTRAP_SENDER_EMAIL`: Sender email address
    /// - `MAILTRAP_SENDER_NAME`: Sender display name
    pub fn from_env() -> Result<Self> {
        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| anyhow!("Missing required environment variable: JWT_SECRET"))?;

        let templates_dir = env::var("TEMPLATES_DIR")
            .map_err(|_| anyhow!("Missing required environment variable: TEMPLATES_DIR"))?;

        // Deprecated: Bootstrap has been removed. These now default to empty strings.
        let external_stylesheet = env::var("EXTERNAL_STYLESHEET").unwrap_or_default();
        let override_stylesheet = env::var("OVERRIDE_STYLESHEET").unwrap_or_default();

        // Get version from Cargo.toml at compile time
        let app_version = env!("CARGO_PKG_VERSION").to_string();

        // Parse allowed origins (optional, defaults to localhost for development)
        let allowed_origins = env::var("ALLOWED_ORIGINS")
            .map(|s| {
                s.split(',')
                    .map(|origin| origin.trim().to_string())
                    .filter(|origin| !origin.is_empty())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_else(|_| {
                // Default to localhost for development
                vec![
                    "http://localhost:8000".to_string(),
                    "http://127.0.0.1:8000".to_string(),
                ]
            });

        // Load email configuration (all optional - falls back to logging sender)
        // If MAILTRAP_SANDBOX_INBOX_ID is set, uses sandbox mode for development
        let email = EmailConfig {
            mailtrap_api_token: env::var("MAILTRAP_API_TOKEN").ok().filter(|s| !s.is_empty()),
            mailtrap_sandbox_inbox_id: env::var("MAILTRAP_SANDBOX_INBOX_ID")
                .ok()
                .filter(|s| !s.is_empty()),
            sender_email: env::var("MAILTRAP_SENDER_EMAIL")
                .unwrap_or_else(|_| "noreply@example.com".to_string()),
            sender_name: env::var("MAILTRAP_SENDER_NAME")
                .unwrap_or_else(|_| "CrustyRustacean Dev Blog".to_string()),
        };

        Ok(Self {
            jwt_secret,
            templates_dir,
            external_stylesheet,
            override_stylesheet,
            app_version,
            allowed_origins,
            email,
        })
    }
}
