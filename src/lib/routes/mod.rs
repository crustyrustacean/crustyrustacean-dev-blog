// src/lib/routes/mod.rs

// modules
pub mod health_check;
pub mod index;
pub mod users;
pub mod auth;
pub mod profile;
pub mod error_pages;

// re-exports
pub use health_check::*;
pub use index::*;
pub use users::*;
pub use auth::*;
pub use profile::*;
pub use error_pages::*;
