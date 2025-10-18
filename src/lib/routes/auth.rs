// src/lib/routes/auth.rs

// route handlers for authentication pages

// dependencies
use crate::auth::OptionalUser;
use crate::errors::AppError;
use crate::state::AppState;
use axum::{extract::State, response::IntoResponse};
use axum_macros::debug_handler;
use axum_template::RenderHtml;
use chrono::{Datelike, Utc};
use serde::Serialize;
use serde_json::{Value, json};

// struct type to represent register page content
#[derive(Debug, Serialize)]
struct RegisterPageContent {
    title: String,
    error: Option<String>,
}

// handler which renders the login page template
#[debug_handler]
pub async fn get_login_page(
    State(state): State<AppState>,
    optional_user: OptionalUser,
) -> Result<impl IntoResponse, AppError> {
    // Get user info if authenticated
    let user_info = if let Some(auth_user) = optional_user.user {
        let conn = state
            .db
            .connect()
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let mut rows = conn
            .query(
                "SELECT username, email, bio, image FROM users WHERE id = ?",
                libsql::params![auth_user.user_id.to_string()],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
        {
            let username: String = row
                .get(0)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;
            let email: String = row
                .get(1)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;
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

    let context: Value = json!({
        "title": "Login",
        "error": null,
        "user": user_info,
        "current_year": Utc::now().year()
    });

    Ok(RenderHtml("auth/login.html", state.engine, context))
}

// handler which renders the register page template
#[debug_handler]
pub async fn get_register_page(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let register_content = RegisterPageContent {
        title: "Register".to_string(),
        error: None,
    };

    Ok(RenderHtml(
        "auth/register.html",
        state.engine,
        register_content,
    ))
}
