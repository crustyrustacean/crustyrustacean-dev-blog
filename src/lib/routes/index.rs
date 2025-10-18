// src/lib/routes/index.rs

// route to serve the home page template

// dependencies
use crate::errors::AppError;
use crate::state::AppState;
use axum::{extract::State, response::IntoResponse};
use axum_macros::debug_handler;
use axum_template::RenderHtml;
use serde::Serialize;

// struct type to represent page content
#[derive(Debug, Serialize)]
struct IndexContent {
    title: String,
    page: String,
    message: String,
}

// handler which renders the index page template
#[debug_handler]
pub async fn get_index(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let index_content = IndexContent {
        title: "Shuttle Template Axum Tera".to_string(),
        page: "Home".to_string(),
        message: "Hello, world!".to_string(),
    };

    Ok(RenderHtml("index.html", state.engine, index_content))
}
