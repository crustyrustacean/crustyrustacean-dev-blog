// src/lib/state.rs

// dependencies
use crate::auth::Keys;
use crate::email::EmailService;
use crate::storage::StorageBackend;
use crate::{ApiError, AppConfig, DatabaseConnection};
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tera::{Tera, Value};
use tokio::sync::RwLock;

// type declarations
pub type TagCache = Arc<RwLock<Option<CachedTags>>>;

// initialized a static variable to hold the compiled templates
static COMPILED_TEMPLATES: OnceCell<Tera> = OnceCell::new();

#[derive(Clone)]
pub struct CachedTags {
    pub tags: Vec<String>,
    pub cached_at: Instant,
}

// struct type to represent the application state
#[derive(Clone)]
pub struct AppState {
    pub templates: &'static Tera,
    pub db: DatabaseConnection,
    pub jwt_keys: Keys,
    pub storage: Arc<dyn StorageBackend>,
    pub cached_tags: TagCache,
    pub email_service: EmailService,
    pub base_url: String,
}

// simplified setup function
fn setup_templates(config: &AppConfig) -> Result<&'static Tera, tera::Error> {
    load_templates(&config.templates_dir, config)
}

// utility function to load and compile the templates once, saving them in static memory
fn load_templates(templates_dir: &str, config: &AppConfig) -> Result<&'static Tera, tera::Error> {
    COMPILED_TEMPLATES.get_or_try_init(|| {
        let mut tera = Tera::new(templates_dir)?;

        let external = config.external_stylesheet.clone();
        let override_css = config.override_stylesheet.clone();

        tera.register_function(
            "theme_stylesheet",
            move |args: &HashMap<String, Value>| -> tera::Result<Value> {
                use tera::from_value;

                let which = args
                    .get("which")
                    .and_then(|v| from_value::<String>(v.clone()).ok())
                    .unwrap_or_else(|| "external".to_string());

                let url = match which.as_str() {
                    "external" => &external,
                    "override" => &override_css,
                    _ => "",
                };

                Ok(Value::String(url.to_string()))
            },
        );

        // Register cache-busting function for static assets
        let app_version = config.app_version.clone();
        tera.register_function(
            "versioned_asset",
            move |args: &HashMap<String, Value>| -> tera::Result<Value> {
                use tera::from_value;

                let path = args
                    .get("path")
                    .and_then(|v| from_value::<String>(v.clone()).ok())
                    .ok_or_else(|| {
                        tera::Error::msg("versioned_asset requires a 'path' argument")
                    })?;

                let versioned_url = format!("{}?v={}", path, app_version);
                Ok(Value::String(versioned_url))
            },
        );

        Ok(tera)
    })
}

// methods to build the configuration
impl AppState {
    // Create a new application state instance
    pub fn new(
        db: DatabaseConnection,
        storage: Arc<dyn StorageBackend>,
        config: &AppConfig,
    ) -> Result<Self, ApiError> {
        let templates = setup_templates(config)?;
        let jwt_keys = Keys::from_config(config);
        let email_service = EmailService::new(config.email_config.clone());

        Ok(Self {
            templates,
            db,
            jwt_keys,
            storage,
            cached_tags: Arc::new(RwLock::new(None)),
            email_service,
            base_url: config.base_url.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn test_config() -> AppConfig {
        AppConfig {
            jwt_secret: "test-secret-key".to_string(),
            templates_dir: "templates/**/*".to_string(),
            external_stylesheet: "https://cdn.example.com/bootstrap.css".to_string(),
            override_stylesheet: "/static/overrides.css".to_string(),
            app_version: "2.9.0".to_string(),
            allowed_origins: vec!["http://localhost:8000".to_string()],
        }
    }

    fn setup_test_templates_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let templates_path = temp_dir.path().join("templates");
        fs::create_dir(&templates_path).unwrap();

        // Create a minimal valid template
        fs::write(
            templates_path.join("test.html"),
            "<html><body>{{ content }}</body></html>",
        )
        .unwrap();

        temp_dir
    }

    #[test]
    fn test_setup_templates_succeeds_with_valid_config() {
        let config = test_config();
        let _temp_dir = setup_test_templates_dir();

        let result = setup_templates(&config);

        assert!(
            result.is_ok(),
            "setup_templates should succeed with valid config"
        );
    }

    #[test]
    fn test_setup_templates_with_empty_stylesheets() {
        let config = AppConfig {
            jwt_secret: "test-secret".to_string(),
            templates_dir: "templates/**/*".to_string(),
            external_stylesheet: "".to_string(),
            override_stylesheet: "".to_string(),
            app_version: "2.9.0".to_string(),
            allowed_origins: vec!["http://localhost:8000".to_string()],
        };
        let _temp_dir = setup_test_templates_dir();

        let result = setup_templates(&config);

        assert!(
            result.is_ok(),
            "setup_templates should handle empty stylesheets"
        );
    }

    #[test]
    fn test_setup_templates_with_special_characters() {
        let config = AppConfig {
            jwt_secret: "test-secret".to_string(),
            templates_dir: "templates/**/*".to_string(),
            external_stylesheet: "https://example.com/style.css?v=1.0&theme=dark".to_string(),
            override_stylesheet: "/static/override-theme.css".to_string(),
            app_version: "2.9.0".to_string(),
            allowed_origins: vec!["http://localhost:8000".to_string()],
        };
        let _temp_dir = setup_test_templates_dir();

        let result = setup_templates(&config);

        assert!(
            result.is_ok(),
            "setup_templates should handle URLs with special characters"
        );
    }
}
