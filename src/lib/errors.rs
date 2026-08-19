// src/lib/errors.rs

//! API error types and handling.
//!
//! This module provides a unified error type for the application
//! with proper HTTP status code mapping and JSON responses.

use actix_web::http::{StatusCode, header::ContentType};
use actix_web::{HttpResponse, ResponseError, body::BoxBody};
use serde::Serialize;

/// The main error type for API operations.
#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Internal server error: {0}")]
    InternalServerError(String),
}

impl From<tera::Error> for ApiError {
    fn from(e: tera::Error) -> Self {
        crate::ApiError::InternalServerError(format!("Template error: {}", e))
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ApiError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let error_message = self.to_string();
        let body = ApiResponse::<()>::error(&error_message);

        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(body)
    }
}

/// Utility function to preserve the error chain in the error message.
pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

/// Standard API response wrapper.
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    /// Create a successful response with data.
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// Create an error response with a message.
    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.to_string()),
        }
    }
}

impl ApiResponse<()> {
    /// Create a success response without data.
    pub fn success_empty() -> Self {
        Self {
            success: true,
            data: None,
            error: None,
        }
    }
}
