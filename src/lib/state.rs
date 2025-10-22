// src/lib/state.rs

// dependencies
use crate::auth::Keys;
use crate::{AppConfig, AppError, DatabaseConnection};
use axum_template::engine::Engine;
use std::collections::HashMap;
use tera::{Tera, Value};

// type declarations
type AppEngine = Engine<Tera>;

// struct type to represent the application state
#[derive(Clone)]
pub struct AppState {
    pub engine: AppEngine,
    pub db: DatabaseConnection,
    pub jwt_keys: Keys,
}

// function to setup the Tera templates
fn setup_templates(config: &AppConfig) -> Engine<Tera> {
    let mut tera = Tera::new("templates/**/*").unwrap();

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

    Engine::from(tera)
}

// methods to build the configuration
impl AppState {
    // Create a new application state instance
    pub fn new(db: DatabaseConnection, config: &AppConfig) -> Result<Self, AppError> {
        let engine = setup_templates(config);
        let jwt_keys = Keys::from_config(config);

        Ok(Self {
            engine,
            db,
            jwt_keys,
        })
    }
}
