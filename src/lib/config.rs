// src/lib/config.rs

// dependencies
use crate::email::EmailConfig;
use anyhow::{Result, anyhow};
use shuttle_runtime::{CustomError, SecretStore};

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

// implement the TryFrom trait for the AppConfig type
impl TryFrom<&SecretStore> for AppConfig {
    type Error = CustomError;

    fn try_from(secrets: &SecretStore) -> Result<Self> {
        let jwt_secret = secrets
            .get("JWT_SECRET")
            .ok_or_else(|| anyhow!("Missing required configuration secret: JWT_SECRET"))?;

        let templates_dir = secrets
            .get("TEMPLATES_DIR")
            .ok_or_else(|| anyhow!("Missing required templates directory: TEMPLATES_DIR"))?;

        // Deprecated: Bootstrap has been removed. These now default to empty strings.
        let external_stylesheet = secrets.get("EXTERNAL_STYLESHEET").unwrap_or_default();

        let override_stylesheet = secrets.get("OVERRIDE_STYLESHEET").unwrap_or_default();

        // Get version from Cargo.toml at compile time
        let app_version = env!("CARGO_PKG_VERSION").to_string();

        // Parse allowed origins (optional, defaults to localhost for development)
        let allowed_origins = secrets
            .get("ALLOWED_ORIGINS")
            .map(|s| {
                s.split(',')
                    .map(|origin| origin.trim().to_string())
                    .filter(|origin| !origin.is_empty())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_else(|| {
                // Default to localhost for development
                vec![
                    "http://localhost:8000".to_string(),
                    "http://127.0.0.1:8000".to_string(),
                ]
            });

        // Load email configuration (all optional - falls back to logging sender)
        // If MAILTRAP_SANDBOX_INBOX_ID is set, uses sandbox mode for development
        let email = EmailConfig {
            mailtrap_api_token: secrets.get("MAILTRAP_API_TOKEN").filter(|s| !s.is_empty()),
            mailtrap_sandbox_inbox_id: secrets
                .get("MAILTRAP_SANDBOX_INBOX_ID")
                .filter(|s| !s.is_empty()),
            sender_email: secrets
                .get("MAILTRAP_SENDER_EMAIL")
                .unwrap_or_else(|| "noreply@example.com".to_string()),
            sender_name: secrets
                .get("MAILTRAP_SENDER_NAME")
                .unwrap_or_else(|| "CrustyRustacean Dev Blog".to_string()),
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
