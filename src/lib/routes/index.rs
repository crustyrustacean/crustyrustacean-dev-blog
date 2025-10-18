// src/lib/routes/index.rs

// route to serve the home page template

// dependencies
use crate::errors::AppError;
use crate::state::AppState;
use axum::{extract::State, response::IntoResponse};
use axum_macros::debug_handler;
use axum_template::RenderHtml;
use chrono::Datelike;
use serde_json::{json, Value};

// handler which renders the index page template
#[debug_handler]
pub async fn get_index(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    // Create a complete context with all variables that base.html might expect
    let current_year = chrono::Utc::now().year();
    let context: Value = json!({
        "title": "CrustyRustacean Dev Blog",
        "page": "Home",
        "message": "Welcome to CrustyRustacean Dev Blog",
        "user": null,
        "flash_message": null,
        "flash_type": null,
        "current_year": current_year
    });

    Ok(RenderHtml("index.html", state.engine, context))
}
