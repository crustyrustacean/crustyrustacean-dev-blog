// src/lib/state.rs

// dependencies
use crate::{AppError, DatabaseConnection, AppConfig};
use crate::auth::Keys;
use axum_template::engine::Engine;
use tera::Tera;

// type declarations
type AppEngine = Engine<Tera>;

// struct type to represent the application state
#[derive(Clone)]
pub struct AppState {
    pub engine: AppEngine,
    pub db: DatabaseConnection,
    pub jwt_keys: Keys,
}

// methods to build the configuration
impl AppState {
    // Create a new application state instance
    pub fn new(db: DatabaseConnection, config: &AppConfig) -> Result<Self, AppError> {
        let tera = Tera::new("templates/**/*")?;
        let jwt_keys = Keys::from_config(config);
        Ok(Self {
            engine: Engine::from(tera),
            db,
            jwt_keys,
        })
    }
}
