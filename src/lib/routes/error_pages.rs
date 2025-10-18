// src/lib/routes/error_pages.rs

use crate::{errors::AppError, state::AppState};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_macros::debug_handler;
use axum_template::RenderHtml;
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
) -> Result<impl IntoResponse, AppError> {
    let request_path = request.uri().path().to_string();

    let content = NotFoundPageContent {
        title: "Page Not Found".to_string(),
        request_path: Some(request_path),
    };

    let response = RenderHtml("errors/404.html", state.engine, content);
    Ok((StatusCode::NOT_FOUND, response))
}

#[debug_handler]
pub async fn handle_404_simple(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let content = NotFoundPageContent {
        title: "Page Not Found".to_string(),
        request_path: None,
    };

    let response = RenderHtml("errors/404.html", state.engine, content);
    Ok((StatusCode::NOT_FOUND, response))
}