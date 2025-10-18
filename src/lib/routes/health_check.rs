// src/lib/routes/health_check.rs

// dependencies
use crate::response::ApiResponse;

// health check handler; returns a 200 OK with a small JSON envelope
pub async fn health_check() -> ApiResponse<()> {
    ApiResponse::success(())
}
