// src/lib/lib.rs

// this is the library crate for the application
// it contains the application logic and types

// module declarations
pub mod auth;
pub mod config;
pub mod database;
pub mod email;
pub mod errors;
pub mod markdown;
pub mod models;
pub mod repositories;
pub mod response;
pub mod routes;
pub mod service;
pub mod shortcodes;
pub mod state;
pub mod storage;
pub mod telemetry;
pub mod theme;
pub mod xml;

// re-exports for easier access
pub use auth::*;
pub use config::*;
pub use database::*;
pub use email::*;
pub use errors::*;
pub use markdown::*;
pub use models::*;
pub use response::*;
pub use service::*;
pub use shortcodes::*;
pub use state::*;
pub use storage::*;
pub use telemetry::*;
pub use theme::*;
pub use xml::*;
