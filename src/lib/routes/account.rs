// src/lib/routes/account.rs

use crate::{ApiError, AppState, auth::AuthenticatedUser};
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use chrono::Datelike;
use serde_json::json;

/// Get account settings page
/// GET /account
pub async fn get_account_page(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let conn = state.db.connect()?;

    // Get user information including theme preference
    let mut rows = conn
        .query(
            "SELECT username, email, bio, image, theme_preference FROM users WHERE id = ?",
            libsql::params![user.user_id.to_string()],
        )
        .await?;

    let row = rows
        .next()
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let username: String = row.get(0)?;
    let email: String = row.get(1)?;
    let bio: Option<String> = row.get(2).ok();
    let image: Option<String> = row.get(3).ok();
    let theme_preference: String = row.get::<String>(4).unwrap_or_else(|_| "auto".to_string());

    // Get available themes
    let available_themes = state.available_themes();

    let user_info = json!({
        "username": username.clone(),
        "email": email.clone(),
        "bio": bio.clone(),
        "image": image.clone(),
        "theme_preference": theme_preference.clone(),
    });

    let template = state.templates.render(
        "account/settings.html",
        &tera::Context::from_serialize(json!({
            "user": user_info,
            "username": username,
            "email": email,
            "bio": bio,
            "image": image,
            "theme_preference": theme_preference,
            "available_themes": available_themes,
            "authenticated": true,
            "current_theme": theme_preference,
            "current_year": chrono::Utc::now().year(),
        }))
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?,
    )?;

    Ok(Html(template))
}
