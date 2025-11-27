// src/lib/errors.rs

// dependencies
use crate::response::ApiResponse;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

// Unified error type for the application
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
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

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    // Template-specific errors
    #[error(transparent)]
    Tera(#[from] tera::Error),
}

impl ApiError {
    /// Sanitize database errors to avoid leaking implementation details
    /// Logs the actual error for debugging while returning a generic message
    pub fn from_db_error(err: libsql::Error) -> Self {
        // Log the actual error for debugging
        tracing::error!("Database error: {}", err);

        let error_string = err.to_string();

        // Check for specific constraint violations and provide user-friendly messages
        if error_string.contains("UNIQUE constraint failed") {
            if error_string.contains("username") {
                return ApiError::Conflict("Username already exists".to_string());
            } else if error_string.contains("email") {
                return ApiError::Conflict("Email already exists".to_string());
            } else {
                return ApiError::Conflict("A record with this value already exists".to_string());
            }
        }

        // For all other database errors, return a generic message
        ApiError::InternalServerError("A database error occurred".to_string())
    }

    /// Sanitize connection errors
    pub fn from_connection_error(err: impl std::fmt::Display) -> Self {
        tracing::error!("Connection error: {}", err);
        ApiError::InternalServerError("Unable to connect to the database".to_string())
    }
}

// Implement From for validator::ValidationErrors
impl From<validator::ValidationErrors> for ApiError {
    fn from(err: validator::ValidationErrors) -> Self {
        ApiError::BadRequest(err.to_string())
    }
}

// Implement From for libsql::Error
impl From<libsql::Error> for ApiError {
    fn from(err: libsql::Error) -> Self {
        ApiError::from_db_error(err)
    }
}

// Implement From for uuid::Error
impl From<uuid::Error> for ApiError {
    fn from(err: uuid::Error) -> Self {
        tracing::warn!("Invalid UUID format: {}", err);
        ApiError::BadRequest("Invalid ID format".to_string())
    }
}

// Implement From for chrono::ParseError
impl From<chrono::ParseError> for ApiError {
    fn from(err: chrono::ParseError) -> Self {
        tracing::warn!("Invalid date format: {}", err);
        ApiError::BadRequest("Invalid date format".to_string())
    }
}

// Implement From for email::SendError
impl From<crate::email::SendError> for ApiError {
    fn from(err: crate::email::SendError) -> Self {
        tracing::error!("Email sending error: {}", err);

        // Provide user-friendly messages based on error type
        let message = match &err {
            crate::email::SendError::Network(_) => {
                "Unable to send verification email. Please try again later.".to_string()
            }
            crate::email::SendError::RateLimited => {
                "Too many requests. Please wait a moment and try again.".to_string()
            }
            crate::email::SendError::Authentication(_) => {
                "Email service configuration error. Please contact support.".to_string()
            }
            _ => "Unable to send verification email. Please try again later.".to_string(),
        };

        ApiError::ServiceUnavailable(message)
    }
}

// implement the IntoResponse trait for the ApiError type
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            ApiError::UnprocessableEntity(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            ApiError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
            ApiError::Tera(err) => {
                // Log the actual template error for debugging
                tracing::error!("Template rendering error: {}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An error occurred while rendering the page".to_string(),
                )
            }
        };

        ApiResponse::<()>::error(&message, status).into_response()
    }
}
