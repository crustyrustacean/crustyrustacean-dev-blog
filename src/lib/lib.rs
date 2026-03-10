// src/lib/lib.rs

//! CrustyRustacean Dev Blog Library
//!
//! A developer blog application built with Actix Web, Tera, and PostgreSQL.

// Module declarations
pub mod configuration;
pub mod auth;
pub mod email;
pub mod errors;
pub mod markdown;
pub mod models;
pub mod repositories;
pub mod response;
pub mod routes;
pub mod shortcodes;
pub mod startup;
pub mod state;
pub mod storage;
pub mod telemetry;
pub mod theme;
pub mod xml;

// Re-exports for easier access
pub use auth::*;
pub use configuration::*;
pub use email::*;
pub use errors::*;
pub use markdown::*;
pub use models::*;
pub use response::*;
pub use shortcodes::*;
pub use startup::*;
pub use state::*;
pub use storage::*;
pub use telemetry::*;
pub use theme::*;
pub use xml::*;
