// src/lib/state.rs

//! Application state management.
//!
//! This module provides the shared application state that is passed to all
//! route handlers. It contains database connections, configuration, and
//! other shared resources.

use crate::configuration::Settings;
use crate::theme::{ThemeListItem, ThemeRegistry};
use once_cell::sync::OnceCell;
use sqlx::PgPool;
use std::sync::Arc;
use tera::{Tera, Value};
use std::collections::HashMap;

// Static variable to hold the compiled templates
static COMPILED_TEMPLATES: OnceCell<Tera> = OnceCell::new();

// Static variable to hold the theme registry
static THEME_REGISTRY: OnceCell<ThemeRegistry> = OnceCell::new();

/// The main application state shared across all request handlers.
#[derive(Clone)]
pub struct AppState {
    /// Tera template engine.
    pub templates: &'static Tera,
    /// Database connection pool.
    pub db_pool: PgPool,
    /// JWT encoding/decoding keys.
    pub jwt_keys: crate::auth::Keys,
    /// Theme registry for CSS generation.
    pub themes: &'static ThemeRegistry,
    /// Application configuration.
    pub config: Arc<Settings>,
}

/// Load templates from the templates directory.
fn load_templates(templates_dir: &str, config: &Settings) -> Result<&'static Tera, tera::Error> {
    COMPILED_TEMPLATES.get_or_try_init(|| {
        let mut tera = Tera::new(templates_dir)?;

        // Register cache-busting function for static assets
        let app_version = config.app.version.clone();
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

        // Register theme_css function
        tera.register_function(
            "theme_css",
            |args: &HashMap<String, Value>| -> tera::Result<Value> {
                use tera::from_value;

                let theme_id = args
                    .get("theme")
                    .and_then(|v| from_value::<String>(v.clone()).ok())
                    .unwrap_or_else(|| "default".to_string());

                let registry = THEME_REGISTRY.get();
                let css = registry
                    .and_then(|r| r.get(&theme_id))
                    .map(|theme| theme.to_css())
                    .unwrap_or_default();

                Ok(Value::String(css))
            },
        );

        // Register available_themes function
        tera.register_function(
            "available_themes",
            |_args: &HashMap<String, Value>| -> tera::Result<Value> {
                let registry = THEME_REGISTRY.get();
                let themes: Vec<ThemeListItem> =
                    registry.map(|r| r.to_theme_list()).unwrap_or_default();

                serde_json::to_value(themes)
                    .map(|v| tera::to_value(v).unwrap_or(Value::Array(vec![])))
                    .map_err(|e| tera::Error::msg(format!("Failed to serialize themes: {}", e)))
            },
        );

        Ok(tera)
    })
}

/// Load themes from the themes directory.
fn setup_themes() -> &'static ThemeRegistry {
    THEME_REGISTRY.get_or_init(|| match ThemeRegistry::load_from_directory("themes") {
        Ok(registry) => {
            tracing::info!("Loaded {} themes", registry.len());
            registry
        }
        Err(e) => {
            tracing::warn!("Failed to load themes: {}, using empty registry", e);
            ThemeRegistry::new()
        }
    })
}

impl AppState {
    /// Create a new application state instance.
    pub fn new(db_pool: PgPool, config: Settings) -> Result<Self, crate::ApiError> {
        // Load themes first (so they're available during template setup)
        let themes = setup_themes();
        let templates = load_templates(&config.templates.dir, &config)?;

        // Create JWT keys from configuration
        let jwt_keys = crate::auth::Keys::from_config(&config.jwt);

        Ok(Self {
            templates,
            db_pool,
            jwt_keys,
            themes,
            config: Arc::new(config),
        })
    }

    /// Get the theme registry.
    pub fn theme_registry(&self) -> &ThemeRegistry {
        self.themes
    }

    /// Get available themes as a list.
    pub fn available_themes(&self) -> Vec<ThemeListItem> {
        self.themes.to_theme_list()
    }
}
