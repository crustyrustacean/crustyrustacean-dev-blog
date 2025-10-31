// src/lib/errors.rs

// dependencies
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

// Unified error type for the application
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unprocessable entity: {0}")]
    UnprocessableEntity(String),

    #[error("Internal server error: {0}")]
    InternalServerError(String),

    // Template-specific errors
    #[error(transparent)]
    Tera(#[from] tera::Error),
}

#[derive(Serialize)]
struct ErrorResponse {
    errors: ErrorBody,
}

#[derive(Serialize)]
struct ErrorBody {
    body: Vec<String>,
}

// implement the IntoResponse trait for the AppError type
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::UnprocessableEntity(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            AppError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Tera(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Tera template rendering error: {err}"),
            ),
        };

        let error_response = ErrorResponse {
            errors: ErrorBody {
                body: vec![message],
            },
        };

        (status, axum::Json(error_response)).into_response()
    }
}
