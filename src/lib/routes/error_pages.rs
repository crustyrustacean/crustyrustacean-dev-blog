// src/lib/routes/error_pages.rs

use crate::{errors::ApiError, state::AppState};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use axum_macros::debug_handler;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct NotFoundPageContent {
    title: String,
    request_path: Option<String>,
}

#[debug_handler]
pub async fn handle_404(
    State(state): State<AppState>,
    request: Request,
) -> Result<impl IntoResponse, ApiError> {
    let request_path = request.uri().path().to_string();

    let content = NotFoundPageContent {
        title: "Page Not Found".to_string(),
        request_path: Some(request_path),
    };

    let html = state
        .templates
        .render("errors/404.html", &tera::Context::from_serialize(&content)?)
        .map_err(|e| ApiError::InternalServerError(format!("Template error: {}", e)))?;

    Ok((StatusCode::NOT_FOUND, Html(html)))
}

#[debug_handler]
pub async fn handle_404_simple(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let content = NotFoundPageContent {
        title: "Page Not Found".to_string(),
        request_path: None,
    };

    let html = state
        .templates
        .render("errors/404.html", &tera::Context::from_serialize(&content)?)
        .map_err(|e| ApiError::InternalServerError(format!("Template error: {}", e)))?;

    Ok((StatusCode::NOT_FOUND, Html(html)))
}
