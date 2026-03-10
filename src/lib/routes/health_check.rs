// src/lib/routes/health_check.rs

//! Health check endpoint for monitoring.

use actix_web::{HttpResponse, Responder};
use serde_json::json;

/// Health check endpoint.
///
/// Returns a 200 OK response to indicate the service is healthy.
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "service": "crustyrustacean-dev-blog"
    }))
}
