// src/lib/state.rs

// dependencies
use crate::auth::Keys;
use crate::storage::StorageBackend;
use crate::{AppConfig, AppError, DatabaseConnection};
use axum_template::engine::Engine;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tera::{Tera, Value};
use tokio::sync::RwLock;

// type declarations
type AppEngine = Engine<Tera>;
pub type TagCache = Arc<RwLock<Option<CachedTags>>>;

#[derive(Clone)]
pub struct CachedTags {
    pub tags: Vec<String>,
    pub cached_at: Instant,
}

// struct type to represent the application state
#[derive(Clone)]
pub struct AppState {
    pub engine: AppEngine,
    pub db: DatabaseConnection,
    pub jwt_keys: Keys,
    pub storage: Arc<dyn StorageBackend>,
    pub cached_tags: TagCache,
}

// function to setup the Tera templates
fn setup_templates(config: &AppConfig) -> Result<Engine<Tera>, tera::Error> {
    let mut tera = Tera::new("templates/**/*")?;

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

    Ok(Engine::from(tera))
}

// methods to build the configuration
impl AppState {
    // Create a new application state instance
    pub fn new(
        db: DatabaseConnection,
        storage: Arc<dyn StorageBackend>,
        config: &AppConfig,
    ) -> Result<Self, AppError> {
        let engine = setup_templates(config)?;
        let jwt_keys = Keys::from_config(config);

        Ok(Self {
            engine,
            db,
            jwt_keys,
            storage,
            cached_tags: Arc::new(RwLock::new(None)),
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
            external_stylesheet: "https://cdn.example.com/bootstrap.css".to_string(),
            override_stylesheet: "/static/overrides.css".to_string(),
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
            external_stylesheet: "".to_string(),
            override_stylesheet: "".to_string(),
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
            external_stylesheet: "https://example.com/style.css?v=1.0&theme=dark".to_string(),
            override_stylesheet: "/static/override-theme.css".to_string(),
        };
        let _temp_dir = setup_test_templates_dir();

        let result = setup_templates(&config);

        assert!(
            result.is_ok(),
            "setup_templates should handle URLs with special characters"
        );
    }
}
