// src/lib/configuration.rs

//! Application configuration management.
//!
//! This module provides configuration loading from YAML files with environment
//! variable overrides. Configuration is loaded from:
//! 1. `configuration/base.yaml` - Base configuration
//! 2. `configuration/{environment}.yaml` - Environment-specific overrides
//! 3. Environment variables with `APP_` prefix

use secrecy::{ExposeSecret, SecretString};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use std::convert::{TryFrom, TryInto};

/// Main application settings.
#[derive(serde::Deserialize, Clone, Debug)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub jwt: JwtSettings,
    pub email: EmailSettings,
    pub storage: StorageSettings,
    pub templates: TemplatesSettings,
    pub app: AppSettings,
}

/// Application server settings.
#[derive(serde::Deserialize, Clone, Debug)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub base_url: String,
}

/// Database connection settings.
#[derive(serde::Deserialize, Clone, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    /// Create PostgreSQL connection options from settings.
    pub fn connect_options(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
            .database(&self.database_name)
    }
}

/// JWT authentication settings.
#[derive(serde::Deserialize, Clone, Debug)]
pub struct JwtSettings {
    pub secret: SecretString,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub expiration_hours: i64,
}

/// Email service settings.
#[derive(serde::Deserialize, Clone, Debug)]
pub struct EmailSettings {
    pub smtp_host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: SecretString,
    pub from_address: String,
    pub from_name: String,
}

/// Storage backend settings.
#[derive(serde::Deserialize, Clone, Debug)]
pub struct StorageSettings {
    pub backend: String,
    pub fs_root: String,
    pub s3_bucket: String,
    pub s3_region: String,
    pub s3_access_key: String,
    pub s3_secret_key: SecretString,
}

/// Template engine settings.
#[derive(serde::Deserialize, Clone, Debug)]
pub struct TemplatesSettings {
    pub dir: String,
}

/// General application settings.
#[derive(serde::Deserialize, Clone, Debug)]
pub struct AppSettings {
    pub version: String,
    pub allowed_origins: Vec<String>,
}

/// Load configuration from YAML files and environment variables.
///
/// Configuration is loaded in order (later sources override earlier):
/// 1. `configuration/base.yaml`
/// 2. `configuration/{environment}.yaml` (based on `APP_ENVIRONMENT`)
/// 3. Environment variables with `APP_` prefix and `__` separator
///
/// # Example Environment Variables
/// - `APP_APPLICATION__PORT=8001` sets `application.port`
/// - `APP_DATABASE__PASSWORD=secret` sets `database.password`
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");

    // Detect the running environment.
    // Default to `local` if unspecified.
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    let environment_filename = format!("{}.yaml", environment.as_str());

    let settings = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        // Add in settings from environment variables (with a prefix of APP and '__' as separator)
        // E.g. `APP_APPLICATION__PORT=5001 would set `Settings.application.port`
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize::<Settings>()
}

/// The possible runtime environment for our application.
#[derive(Debug, Clone, Copy)]
pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_try_from() {
        assert!(matches!(Environment::try_from("local".to_string()), Ok(Environment::Local)));
        assert!(matches!(Environment::try_from("production".to_string()), Ok(Environment::Production)));
        assert!(matches!(Environment::try_from("LOCAL".to_string()), Ok(Environment::Local)));
        assert!(Environment::try_from("staging".to_string()).is_err());
    }
}
