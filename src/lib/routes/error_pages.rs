// src/lib/routes/error_pages.rs

use crate::{auth::OptionalUser, errors::ApiError, state::AppState};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use axum_macros::debug_handler;
use chrono::Datelike;
use serde_json::json;

#[debug_handler]
pub async fn handle_404(
    State(state): State<AppState>,
    optional_user: OptionalUser,
    request: Request,
) -> Result<impl IntoResponse, ApiError> {
    let request_path = request.uri().path().to_string();

    // Get user info if authenticated
    let user_info = if let Some(ref auth_user) = optional_user.user {
        let conn = state.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT username, email, bio, image FROM users WHERE id = ?",
                libsql::params![auth_user.user_id.to_string()],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let username: String = row.get(0)?;
            let email: String = row.get(1)?;
            let bio: Option<String> = row.get(2).ok();
            let image: Option<String> = row.get(3).ok();

            Some(json!({
                "username": username,
                "email": email,
                "bio": bio,
                "image": image
            }))
        } else {
            None
        }
    } else {
        None
    };

    let content = json!({
        "title": "Page Not Found",
        "request_path": request_path,
        "user": user_info,
        "current_year": chrono::Utc::now().year(),
    });

    let html = state
        .templates
        .render("errors/404.html", &tera::Context::from_serialize(&content)?)
        .map_err(|e| ApiError::InternalServerError(format!("Template error: {}", e)))?;

    Ok((StatusCode::NOT_FOUND, Html(html)))
}

#[debug_handler]
pub async fn handle_404_simple(
    State(state): State<AppState>,
    optional_user: OptionalUser,
) -> Result<impl IntoResponse, ApiError> {
    // Get user info if authenticated
    let user_info = if let Some(ref auth_user) = optional_user.user {
        let conn = state.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT username, email, bio, image FROM users WHERE id = ?",
                libsql::params![auth_user.user_id.to_string()],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let username: String = row.get(0)?;
            let email: String = row.get(1)?;
            let bio: Option<String> = row.get(2).ok();
            let image: Option<String> = row.get(3).ok();

            Some(json!({
                "username": username,
                "email": email,
                "bio": bio,
                "image": image
            }))
        } else {
            None
        }
    } else {
        None
    };

    let content = json!({
        "title": "Page Not Found",
        "request_path": serde_json::Value::Null,
        "user": user_info,
        "current_year": chrono::Utc::now().year(),
    });

    let html = state
        .templates
        .render("errors/404.html", &tera::Context::from_serialize(&content)?)
        .map_err(|e| ApiError::InternalServerError(format!("Template error: {}", e)))?;

    Ok((StatusCode::NOT_FOUND, Html(html)))
}
