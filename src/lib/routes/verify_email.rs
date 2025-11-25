// src/lib/routes/verify_email.rs

use crate::{ApiError, AppState};
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use chrono::Utc;
use libsql::params;

/// Verify email address
/// GET /verify-email/:token
/// This endpoint is called when user clicks the verification link in their email
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

        // Check if token is expired
        let exp_date = chrono::DateTime::parse_from_rfc3339(&expires_at)
            .map_err(|_| ApiError::InternalServerError("Invalid date format".to_string()))?
            .with_timezone(&Utc);

        if exp_date < Utc::now() {
            // Token expired - delete it and show error
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
            // Get username for the welcome message
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
                    // Already verified
                    (
                        true,
                        "Your email has already been verified. You can log in now.".to_string(),
                        Some(username),
                    )
                } else {
                    // Mark email as verified
                    let now = Utc::now();
                    conn.execute(
                        "UPDATE users SET email_verified = 1, updated_at = ? WHERE id = ?",
                        params![now.to_rfc3339(), user_id],
                    )
                    .await?;

                    // Delete the verification token (it's been used)
                    conn.execute(
                        "DELETE FROM email_verification_tokens WHERE id = ?",
                        params![token_id],
                    )
                    .await?;

                    (
                        true,
                        "Your email has been verified successfully! You can now log in.".to_string(),
                        Some(username),
                    )
                }
            } else {
                // User not found (shouldn't happen, but handle gracefully)
                (
                    false,
                    "User not found. The account may have been deleted.".to_string(),
                    None,
                )
            }
        }
    } else {
        // Token not found
        (
            false,
            "Invalid verification link. It may have already been used or expired.".to_string(),
            None,
        )
    };

    // Render the verification result page
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
