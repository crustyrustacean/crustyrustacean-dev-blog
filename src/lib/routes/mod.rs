// src/lib/routes/mod.rs

//! HTTP route handlers.
//!
//! This module contains all route handlers organized by domain.

// Module declarations
pub mod health_check;

// Re-exports
pub use health_check::*;

use actix_web::web;

/// Configure all application routes.
///
/// This function is called by the startup module to register all routes
/// with the Actix Web application.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        // Health check
        .route("/health_check", web::get().to(health_check))
        // TODO: Add more routes as they are migrated
        // .route("/", web::get().to(get_index))
        // .route("/api/users", web::post().to(register_user))
        // etc.
        ;
}
