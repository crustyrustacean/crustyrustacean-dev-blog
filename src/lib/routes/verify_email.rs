// src/lib/routes/verify_email.rs

use crate::{ApiError, AppState};
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use chrono::Utc;
use libsql::params;

/// Display email verification page (does NOT verify - just shows the form)
/// GET /verify-email/:token
pub async fn get_verify_email_page(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let conn = state.db.connect()?;

    // Check if token exists and is valid (but don't verify yet)
    let mut rows = conn
        .query(
            "SELECT id, user_id, expires_at FROM email_verification_tokens WHERE token = ?",
            params![token.clone()],
        )
        .await?;

    let (valid, expired, message) = if let Some(row) = rows.next().await? {
        let expires_at: String = row.get(2)?;

        let exp_date = chrono::DateTime::parse_from_rfc3339(&expires_at)
            .map_err(|_| ApiError::InternalServerError("Invalid date format".to_string()))?
            .with_timezone(&Utc);

        if exp_date < Utc::now() {
            (
                false,
                true,
                "This verification link has expired. Please register again.",
            )
        } else {
            (
                true,
                false,
                "Click the button below to verify your email address.",
            )
        }
    } else {
        (
            false,
            false,
            "Invalid verification link. It may have already been used.",
        )
    };

    // Render the verification page with a form/button
    let mut context = tera::Context::new();
    context.insert("token", &token);
    context.insert("valid", &valid);
    context.insert("expired", &expired);
    context.insert("message", &message);

    let template = state
        .templates
        .render("auth/verify_email_confirm.html", &context)
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(Html(template))
}

/// Actually perform the email verification
/// POST /verify-email/:token
pub async fn verify_email(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let conn = state.db.connect()?;

    // Find and validate token
    let mut rows = conn
        .query(
            "SELECT id, user_id, expires_at FROM email_verification_tokens WHERE token = ?",
            params![token.clone()],
        )
        .await?;

    let (success, message, username) = if let Some(row) = rows.next().await? {
        let token_id: String = row.get(0)?;
        let user_id: String = row.get(1)?;
        let expires_at: String = row.get(2)?;

        let exp_date = chrono::DateTime::parse_from_rfc3339(&expires_at)
            .map_err(|_| ApiError::InternalServerError("Invalid date format".to_string()))?
            .with_timezone(&Utc);

        if exp_date < Utc::now() {
            conn.execute(
                "DELETE FROM email_verification_tokens WHERE id = ?",
                params![token_id],
            )
            .await?;

            (
                false,
                "This verification link has expired. Please register again.".to_string(),
                None,
            )
        } else {
            let mut user_rows = conn
                .query(
                    "SELECT username, email_verified FROM users WHERE id = ?",
                    params![user_id.clone()],
                )
                .await?;

            if let Some(user_row) = user_rows.next().await? {
                let username: String = user_row.get(0)?;
                let email_verified: i64 = user_row.get(1)?;

                if email_verified == 1 {
                    (
                        true,
                        "Your email has already been verified. You can log in now.".to_string(),
                        Some(username),
                    )
                } else {
                    let now = Utc::now();
                    conn.execute(
                        "UPDATE users SET email_verified = 1, updated_at = ? WHERE id = ?",
                        params![now.to_rfc3339(), user_id],
                    )
                    .await?;

                    conn.execute(
                        "DELETE FROM email_verification_tokens WHERE id = ?",
                        params![token_id],
                    )
                    .await?;

                    (
                        true,
                        "Your email has been verified successfully! You can now log in."
                            .to_string(),
                        Some(username),
                    )
                }
            } else {
                (
                    false,
                    "User not found. The account may have been deleted.".to_string(),
                    None,
                )
            }
        }
    } else {
        (
            false,
            "Invalid verification link. It may have already been used or expired.".to_string(),
            None,
        )
    };

    let mut context = tera::Context::new();
    context.insert("success", &success);
    context.insert("message", &message);
    if let Some(name) = username {
        context.insert("username", &name);
    }

    let template = state
        .templates
        .render("auth/verify_email.html", &context)
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(Html(template))
}
