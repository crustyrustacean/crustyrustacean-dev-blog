// src/lib/lib.rs

// this is the library crate for the application
// it contains the application logic and types

// module declarations
pub mod auth;
pub mod config;
pub mod database;
pub mod errors;
pub mod markdown;
pub mod models;
pub mod response;
pub mod routes;
pub mod startup;
pub mod state;
pub mod storage;
pub mod telemetry;

// re-exports for easier access
pub use auth::*;
pub use config::*;
pub use database::*;
pub use errors::*;
pub use markdown::*;
pub use models::*;
pub use response::*;
pub use startup::*;
pub use state::*;
pub use storage::*;
pub use telemetry::*;
