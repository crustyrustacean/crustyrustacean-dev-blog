// src/lib/response.rs

//! Standard API response types.
//!
//! This module provides a unified response format for all API endpoints.

use actix_web::{HttpRequest, HttpResponse, Responder, body::BoxBody, http::header::ContentType};
use serde::{Deserialize, Serialize};

/// Standard API response wrapper.
#[derive(Deserialize, Serialize)]
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

impl<T: Serialize> Responder for ApiResponse<T> {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        match serde_json::to_string(&self) {
            Ok(body) => HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(body),
            Err(e) => {
                tracing::error!("Failed to serialize API response: {:?}", e);
                HttpResponse::InternalServerError()
                    .content_type(ContentType::plaintext())
                    .body("Internal Server Error")
            }
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
