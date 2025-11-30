// src/lib/routes/password_reset.rs

use crate::{
    ApiError, AppState,
    auth::{AuthenticatedUser, hash_password, verify_password},
    models::{ChangePasswordRequest, PasswordResetComplete, PasswordResetRequest},
    response::ApiResponse,
};
use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
    response::{Html, IntoResponse},
};
use chrono::{Duration, Utc};
use libsql::params;
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

/// Get password reset request page
/// GET /password-reset/request
pub async fn get_password_reset_request_page(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let template = state
        .templates
        .render("auth/password_reset_request.html", &tera::Context::new())
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(Html(template))
}

/// Request password reset
/// POST /api/password-reset/request
pub async fn request_password_reset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<PasswordResetRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    payload.validate()?;

    let conn = state.db.connect()?;
    let email = payload.email.to_lowercase();

    // Find user by email
    let mut rows = conn
        .query(
            "SELECT id, username FROM users WHERE email = ?",
            params![email.clone()],
        )
        .await?;

    // Always return success even if email not found (security best practice)
    // This prevents email enumeration attacks
    if let Some(row) = rows.next().await? {
        let user_id: String = row.get(0)?;
        let username: String = row.get(1)?;

        // Create password reset token
        let token_id = Uuid::new_v4();
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::hours(1);

        // Save token to database
        conn.execute(
            "INSERT INTO password_reset_tokens (id, user_id, token, expires_at) VALUES (?, ?, ?, ?)",
            params![
                token_id.to_string(),
                user_id,
                token.clone(),
                expires_at.to_rfc3339()
            ],
        )
        .await?;

        // Send password reset email
        // Derive base URL from request headers
        let host = headers
            .get("host")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("localhost:8000");
        let protocol = if host.contains("localhost") || host.contains("127.0.0.1") {
            "http"
        } else {
            "https"
        };
        let base_url = format!("{}://{}", protocol, host);

        if let Err(e) = state
            .email
            .send_password_reset_email(&email, &username, &token, &base_url)
            .await
        {
            tracing::warn!("Failed to send password reset email to {}: {}", email, e);
        }
    }

    Ok(ApiResponse::success(json!({
        "message": "If your email is registered, you will receive password reset instructions shortly."
    })))
}

/// Get password reset page
/// GET /password-reset/:token
pub async fn get_password_reset_page(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let conn = state.db.connect()?;

    // Validate token exists and is not expired
    let mut rows = conn
        .query(
            "SELECT id, expires_at, used FROM password_reset_tokens WHERE token = ?",
            params![token.clone()],
        )
        .await?;

    if let Some(row) = rows.next().await? {
        let expires_at: String = row.get(1)?;
        let used: i64 = row.get(2)?;

        // Check if token is used
        if used == 1 {
            return Err(ApiError::BadRequest(
                "This password reset link has already been used.".to_string(),
            ));
        }

        // Check if token is expired
        let exp_date = chrono::DateTime::parse_from_rfc3339(&expires_at)
            .map_err(|_| ApiError::InternalServerError("Invalid date format".to_string()))?
            .with_timezone(&Utc);

        if exp_date < Utc::now() {
            return Err(ApiError::BadRequest(
                "This password reset link has expired. Please request a new one.".to_string(),
            ));
        }

        // Render password reset form
        let template = state
            .templates
            .render(
                "auth/password_reset.html",
                &tera::Context::from_serialize(json!({
                    "token": token,
                }))
                .map_err(|e| ApiError::InternalServerError(e.to_string()))?,
            )
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

        Ok(Html(template))
    } else {
        Err(ApiError::NotFound(
            "Invalid password reset link.".to_string(),
        ))
    }
}

/// Complete password reset
/// POST /api/password-reset/:token
pub async fn complete_password_reset(
    State(state): State<AppState>,
    Path(token): Path<String>,
    Json(payload): Json<PasswordResetComplete>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    payload.validate()?;

    let conn = state.db.connect()?;

    // Find and validate token
    let mut rows = conn
        .query(
            "SELECT id, user_id, expires_at, used FROM password_reset_tokens WHERE token = ?",
            params![token],
        )
        .await?;

    let (token_id, user_id) = if let Some(row) = rows.next().await? {
        let token_id: String = row.get(0)?;
        let user_id: String = row.get(1)?;
        let expires_at: String = row.get(2)?;
        let used: i64 = row.get(3)?;

        // Check if token is used
        if used == 1 {
            return Err(ApiError::BadRequest(
                "This password reset link has already been used.".to_string(),
            ));
        }

        // Check if token is expired
        let exp_date = chrono::DateTime::parse_from_rfc3339(&expires_at)
            .map_err(|_| ApiError::InternalServerError("Invalid date format".to_string()))?
            .with_timezone(&Utc);

        if exp_date < Utc::now() {
            return Err(ApiError::BadRequest(
                "This password reset link has expired. Please request a new one.".to_string(),
            ));
        }

        (token_id, user_id)
    } else {
        return Err(ApiError::NotFound(
            "Invalid password reset link.".to_string(),
        ));
    };

    // Hash new password
    let password_hash = hash_password(&payload.password)?;
    let now = Utc::now().to_rfc3339();

    // Update user password and password_changed_at to invalidate existing tokens
    conn.execute(
        "UPDATE users SET password_hash = ?, updated_at = ?, password_changed_at = ? WHERE id = ?",
        params![password_hash, now.clone(), now, user_id.clone()],
    )
    .await?;

    // Mark token as used
    conn.execute(
        "UPDATE password_reset_tokens SET used = 1 WHERE id = ?",
        params![token_id],
    )
    .await?;

    Ok(ApiResponse::success(json!({
        "message": "Your password has been reset successfully. You can now log in with your new password."
    })))
}

/// Change password (for authenticated users)
/// POST /api/account/password
pub async fn change_password(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    payload.validate()?;

    let conn = state.db.connect()?;

    // Get current password hash
    let mut rows = conn
        .query(
            "SELECT password_hash FROM users WHERE id = ?",
            params![user.user_id.to_string()],
        )
        .await?;

    let current_hash = if let Some(row) = rows.next().await? {
        let hash: String = row.get(0)?;
        hash
    } else {
        return Err(ApiError::NotFound("User not found.".to_string()));
    };

    // Verify current password
    if !verify_password(&payload.current_password, &current_hash)? {
        return Err(ApiError::Unauthorized(
            "Current password is incorrect.".to_string(),
        ));
    }

    // Hash new password
    let new_password_hash = hash_password(&payload.new_password)?;
    let now = Utc::now().to_rfc3339();

    // Update password and password_changed_at to invalidate existing tokens
    conn.execute(
        "UPDATE users SET password_hash = ?, updated_at = ?, password_changed_at = ? WHERE id = ?",
        params![
            new_password_hash,
            now.clone(),
            now,
            user.user_id.to_string()
        ],
    )
    .await?;

    Ok(ApiResponse::success(json!({
        "message": "Your password has been changed successfully."
    })))
}
