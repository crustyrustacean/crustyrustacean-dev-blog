// src/lib/config.rs

// dependencies
use anyhow::{Result, anyhow};
use shuttle_runtime::{CustomError, SecretStore};

// Email configuration for SMTP
#[derive(Clone, Debug)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
    pub from_name: String,
}

// struct type to represent the application configuration
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub jwt_secret: String,
    pub templates_dir: String,
    pub external_stylesheet: String,
    pub override_stylesheet: String,
    pub app_version: String,
    pub email_config: Option<EmailConfig>,
    pub base_url: String,
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

        let external_stylesheet = secrets
            .get("EXTERNAL_STYLESHEET")
            .ok_or_else(|| anyhow!("Missing required configuration secret: EXTERNAL_STYLESHEET"))?;

        let override_stylesheet = secrets
            .get("OVERRIDE_STYLESHEET")
            .ok_or_else(|| anyhow!("Missing required configuration secret: OVERRIDE_STYLESHEET"))?;

        // Get version from Cargo.toml at compile time
        let app_version = env!("CARGO_PKG_VERSION").to_string();

        // Get base URL (optional, defaults to localhost for development)
        let base_url = secrets
            .get("BASE_URL")
            .unwrap_or_else(|| "http://localhost:8000".to_string());

        // Email configuration (optional - if not provided, emails will be logged to console)
        let email_config = if let (Some(smtp_host), Some(smtp_username), Some(smtp_password), Some(from_email)) = (
            secrets.get("SMTP_HOST"),
            secrets.get("SMTP_USERNAME"),
            secrets.get("SMTP_PASSWORD"),
            secrets.get("FROM_EMAIL"),
        ) {
            let smtp_port = secrets
                .get("SMTP_PORT")
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(587); // Default to standard SMTP submission port

            let from_name = secrets
                .get("FROM_NAME")
                .unwrap_or_else(|| "CrustyRustacean Dev Blog".to_string());

            Some(EmailConfig {
                smtp_host,
                smtp_port,
                smtp_username,
                smtp_password,
                from_email,
                from_name,
            })
        } else {
            None
        };

        Ok(Self {
            jwt_secret,
            templates_dir,
            external_stylesheet,
            override_stylesheet,
            app_version,
            email_config,
            base_url,
        })
    }
}
