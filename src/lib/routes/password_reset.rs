// src/lib/routes/password_reset.rs

use crate::{
    AppError, AppState, EmailService,
    auth::{AuthenticatedUser, hash_password, verify_password},
    models::{ChangePasswordRequest, PasswordResetComplete, PasswordResetRequest},
    response::ApiResponse,
};
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    Json,
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
) -> Result<impl IntoResponse, AppError> {
    let template = state
        .templates
        .render("auth/password_reset_request.html", &tera::Context::new())
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(Html(template))
}

/// Request password reset
/// POST /api/password-reset/request
pub async fn request_password_reset(
    State(state): State<AppState>,
    Json(payload): Json<PasswordResetRequest>,
) -> Result<impl IntoResponse, AppError> {
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
        // TODO: Get base URL from config
        let base_url = "http://localhost:8000"; // Placeholder
        let email_service = EmailService::new(
            "noreply@crustyrustacean.dev".to_string(),
            "CrustyRustacean Dev Blog".to_string(),
        );
        email_service
            .send_password_reset_email(&email, &username, &token, base_url)
            .await?;
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
) -> Result<impl IntoResponse, AppError> {
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
            return Err(AppError::BadRequest(
                "This password reset link has already been used.".to_string(),
            ));
        }

        // Check if token is expired
        let exp_date = chrono::DateTime::parse_from_rfc3339(&expires_at)
            .map_err(|_| AppError::InternalServerError("Invalid date format".to_string()))?
            .with_timezone(&Utc);

        if exp_date < Utc::now() {
            return Err(AppError::BadRequest(
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
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            )
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(Html(template))
    } else {
        Err(AppError::NotFound(
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
) -> Result<impl IntoResponse, AppError> {
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
            return Err(AppError::BadRequest(
                "This password reset link has already been used.".to_string(),
            ));
        }

        // Check if token is expired
        let exp_date = chrono::DateTime::parse_from_rfc3339(&expires_at)
            .map_err(|_| AppError::InternalServerError("Invalid date format".to_string()))?
            .with_timezone(&Utc);

        if exp_date < Utc::now() {
            return Err(AppError::BadRequest(
                "This password reset link has expired. Please request a new one.".to_string(),
            ));
        }

        (token_id, user_id)
    } else {
        return Err(AppError::NotFound(
            "Invalid password reset link.".to_string(),
        ));
    };

    // Hash new password
    let password_hash = hash_password(&payload.password)?;

    // Update user password
    conn.execute(
        "UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?",
        params![password_hash, Utc::now().to_rfc3339(), user_id.clone()],
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
) -> Result<impl IntoResponse, AppError> {
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
        return Err(AppError::NotFound("User not found.".to_string()));
    };

    // Verify current password
    if !verify_password(&payload.current_password, &current_hash)? {
        return Err(AppError::Unauthorized(
            "Current password is incorrect.".to_string(),
        ));
    }

    // Hash new password
    let new_password_hash = hash_password(&payload.new_password)?;

    // Update password
    conn.execute(
        "UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?",
        params![
            new_password_hash,
            Utc::now().to_rfc3339(),
            user.user_id.to_string()
        ],
    )
    .await?;

    Ok(ApiResponse::success(json!({
        "message": "Your password has been changed successfully."
    })))
}
