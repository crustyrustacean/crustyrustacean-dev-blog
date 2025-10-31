// src/lib/config.rs

// dependencies
use anyhow::{Result, anyhow};
use shuttle_runtime::{CustomError, SecretStore};

// struct type to represent the application configuration
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub jwt_secret: String,
    pub templates_dir: String,
    pub external_stylesheet: String,
    pub override_stylesheet: String,
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

        Ok(Self {
            jwt_secret,
            templates_dir,
            external_stylesheet,
            override_stylesheet,
        })
    }
}
