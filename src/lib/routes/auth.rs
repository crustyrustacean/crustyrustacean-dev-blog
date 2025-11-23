// src/lib/routes/auth.rs

// route handlers for authentication pages

// dependencies
use crate::auth::OptionalUser;
use crate::errors::ApiError;
use crate::state::AppState;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use axum_macros::debug_handler;
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
) -> Result<impl IntoResponse, ApiError> {
    // Get user info if authenticated
    let user_info = if let Some(auth_user) = optional_user.user {
        let conn = state.db.connect().map_err(ApiError::from_connection_error)?;

        let mut rows = conn
            .query(
                "SELECT username, email, bio, image FROM users WHERE id = ?",
                libsql::params![auth_user.user_id.to_string()],
            )
            .await
            ?;

        if let Some(row) = rows
            .next()
            .await
            ?
        {
            let username: String = row
                .get(0)
                ?;
            let email: String = row
                .get(1)
                ?;
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

    let html = state
        .templates
        .render("auth/login.html", &tera::Context::from_serialize(&context)?)
        .map_err(|e| ApiError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}

// handler which renders the register page template
#[debug_handler]
pub async fn get_register_page(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let register_content = RegisterPageContent {
        title: "Register".to_string(),
        error: None,
    };

    let html = state
        .templates
        .render(
            "auth/register.html",
            &tera::Context::from_serialize(&register_content)?,
        )
        .map_err(|e| ApiError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}
