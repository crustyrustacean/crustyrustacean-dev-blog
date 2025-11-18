// src/lib/routes/account.rs

use crate::{AppError, AppState, auth::AuthenticatedUser};
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use serde_json::json;

/// Get account settings page
/// GET /account
pub async fn get_account_page(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.db.connect()?;

    // Get user information
    let mut rows = conn
        .query(
            "SELECT username, email, bio, image FROM users WHERE id = ?",
            libsql::params![user.user_id.to_string()],
        )
        .await?;

    let row = rows
        .next()
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let username: String = row.get(0)?;
    let email: String = row.get(1)?;
    let bio: Option<String> = row.get(2).ok();
    let image: Option<String> = row.get(3).ok();

    let template = state
        .templates
        .render(
            "account/settings.html",
            &tera::Context::from_serialize(json!({
                "username": username,
                "email": email,
                "bio": bio,
                "image": image,
                "authenticated": true,
            }))
            .map_err(|e| AppError::InternalServerError(e.to_string()))?,
        )
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(Html(template))
}
