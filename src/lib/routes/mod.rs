// src/lib/routes/mod.rs

// modules
pub mod auth;
pub mod error_pages;
pub mod health_check;
pub mod index;
pub mod profile;
pub mod users;

// re-exports
pub use auth::*;
pub use error_pages::*;
pub use health_check::*;
pub use index::*;
pub use profile::*;
pub use users::*;
