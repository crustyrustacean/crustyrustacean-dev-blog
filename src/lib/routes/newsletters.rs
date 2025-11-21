// src/lib/routes/newsletters.rs

use crate::auth::{AuthenticatedUser, AuthorUser};
use crate::errors::AppError;
use crate::models::{
    CreateNewsletterIssue, NewsletterIssueResponse, NewsletterStats, SubscribeRequest,
    UpdateNewsletterIssue,
};
use crate::response::ApiResponse;
use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use chrono::{Datelike, Utc};
use libsql::params;
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

/// Subscribe to newsletter
/// POST /api/newsletters/subscribe
pub async fn subscribe(
    State(state): State<AppState>,
    Json(payload): Json<SubscribeRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate request
    payload.validate()?;

    let conn = state.db.connect()?;
    let email = payload.email.to_lowercase();
    let name = payload.name;

    // Check if email already subscribed
    let mut existing = conn
        .query(
            "SELECT id, confirmed, unsubscribed_at FROM newsletter_subscribers WHERE email = ?",
            params![email.clone()],
        )
        .await?;

    if let Some(row) = existing.next().await? {
        let confirmed: i64 = row.get(1)?;
        let unsubscribed_at: Option<String> = row.get(2)?;

        // If already confirmed and not unsubscribed
        if confirmed == 1 && unsubscribed_at.is_none() {
            return Ok(ApiResponse::success(json!({
                "message": "You are already subscribed to our newsletter!",
                "email": email
            })));
        }

        // If unsubscribed, allow re-subscription
        if unsubscribed_at.is_some() {
            let confirmation_token = Uuid::new_v4().to_string();
            conn.execute(
                "UPDATE newsletter_subscribers SET confirmation_token = ?, unsubscribed_at = NULL, confirmed = 0, updated_at = ? WHERE email = ?",
                params![confirmation_token, Utc::now().to_rfc3339(), email.clone()],
            )
            .await?;

            // TODO: Send confirmation email
            return Ok(ApiResponse::success(json!({
                "message": "Please check your email to confirm your subscription.",
                "email": email
            })));
        }

        // If not confirmed yet, resend confirmation
        return Ok(ApiResponse::success(json!({
            "message": "A confirmation email has been sent. Please check your inbox.",
            "email": email
        })));
    }

    // Create new subscriber
    let id = Uuid::new_v4();
    let confirmation_token = Uuid::new_v4().to_string();
    let unsubscribe_token = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        r"INSERT INTO newsletter_subscribers (id, email, name, confirmation_token, unsubscribe_token, confirmed, subscribed_at, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, 0, ?, ?, ?)",
        params![
            id.to_string(),
            email.clone(),
            name.clone(),
            confirmation_token,
            unsubscribe_token,
            now.clone(),
            now.clone(),
            now.clone()
        ],
    )
    .await?;

    // TODO: Send confirmation email with token

    Ok(ApiResponse::success(json!({
        "message": "Please check your email to confirm your subscription.",
        "email": email
    })))
}

/// Confirm newsletter subscription
/// POST /api/newsletters/confirm/:token
pub async fn confirm_subscription(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.db.connect()?;

    // Find subscriber with this confirmation token
    let mut rows = conn
        .query(
            "SELECT id, email, confirmed FROM newsletter_subscribers WHERE confirmation_token = ?",
            params![token.clone()],
        )
        .await?;

    let row = rows
        .next()
        .await?
        .ok_or_else(|| AppError::NotFound("Invalid confirmation token".to_string()))?;

    let id: String = row.get(0)?;
    let email: String = row.get(1)?;
    let confirmed: i64 = row.get(2)?;

    // If already confirmed
    if confirmed == 1 {
        return Ok(ApiResponse::success(json!({
            "message": "Your subscription is already confirmed!",
            "email": email
        })));
    }

    // Update subscriber to confirmed (keep confirmation_token for idempotency)
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE newsletter_subscribers SET confirmed = 1, confirmed_at = ?, updated_at = ? WHERE id = ?",
        params![now.clone(), now.clone(), id],
    )
    .await?;

    Ok(ApiResponse::success(json!({
        "message": "Thank you! Your subscription has been confirmed.",
        "email": email
    })))
}

/// Unsubscribe from newsletter
/// POST /api/newsletters/unsubscribe/:token or GET /api/newsletters/unsubscribe/:token
pub async fn unsubscribe(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.db.connect()?;

    // Find subscriber with this unsubscribe token
    let mut rows = conn
        .query(
            "SELECT id, email, unsubscribed_at FROM newsletter_subscribers WHERE unsubscribe_token = ?",
            params![token.clone()],
        )
        .await?;

    let row = rows
        .next()
        .await?
        .ok_or_else(|| AppError::NotFound("Invalid unsubscribe token".to_string()))?;

    let id: String = row.get(0)?;
    let email: String = row.get(1)?;
    let unsubscribed_at: Option<String> = row.get(2)?;

    // If already unsubscribed
    if unsubscribed_at.is_some() {
        return Ok(ApiResponse::success(json!({
            "message": "You have already unsubscribed from our newsletter.",
            "email": email
        })));
    }

    // Update subscriber to unsubscribed
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE newsletter_subscribers SET unsubscribed_at = ?, updated_at = ? WHERE id = ?",
        params![now.clone(), now.clone(), id],
    )
    .await?;

    Ok(ApiResponse::success(json!({
        "message": "You have been successfully unsubscribed from our newsletter.",
        "email": email
    })))
}

/// List all newsletter issues (author or admin)
/// GET /api/admin/newsletters
pub async fn list_newsletters(
    _auth_user: AuthorUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.db.connect()?;

    let mut rows = conn
        .query(
            "SELECT id, title, subject, body, status, author_id, scheduled_at, sent_at, recipient_count, created_at, updated_at
             FROM newsletter_issues
             ORDER BY created_at DESC",
            params![],
        )
        .await?;

    let mut issues = Vec::new();
    while let Some(row) = rows.next().await? {
        let id: String = row.get(0)?;
        let title: String = row.get(1)?;
        let subject: String = row.get(2)?;
        let body: String = row.get(3)?;
        let status: String = row.get(4)?;
        let author_id: String = row.get(5)?;
        let scheduled_at: Option<String> = row.get(6)?;
        let sent_at: Option<String> = row.get(7)?;
        let recipient_count: i32 = row.get(8)?;
        let created_at: String = row.get(9)?;
        let updated_at: String = row.get(10)?;

        issues.push(NewsletterIssueResponse {
            id: Uuid::parse_str(&id)?,
            title,
            subject,
            body,
            status,
            author_id: Uuid::parse_str(&author_id)?,
            scheduled_at: scheduled_at.and_then(|s| s.parse().ok()),
            sent_at: sent_at.and_then(|s| s.parse().ok()),
            recipient_count,
            created_at: created_at.parse()?,
            updated_at: updated_at.parse()?,
        });
    }

    Ok(ApiResponse::success(json!({ "newsletters": issues })))
}

/// Create newsletter issue (author or admin)
/// POST /api/admin/newsletters
pub async fn create_newsletter(
    auth_user: AuthorUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateNewsletterIssue>,
) -> Result<impl IntoResponse, AppError> {
    // Validate request
    payload.validate()?;

    let conn = state.db.connect()?;
    let id = Uuid::new_v4();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        r"INSERT INTO newsletter_issues (id, title, subject, body, status, author_id, scheduled_at, created_at, updated_at)
         VALUES (?, ?, ?, ?, 'draft', ?, ?, ?, ?)",
        params![
            id.to_string(),
            payload.title,
            payload.subject,
            payload.body,
            auth_user.user_id.to_string(),
            payload.scheduled_at.map(|dt| dt.to_rfc3339()),
            now.clone(),
            now.clone()
        ],
    )
    .await?;

    Ok(ApiResponse::success_with_status(
        StatusCode::CREATED,
        json!({
            "message": "Newsletter created successfully",
            "id": id
        }),
    ))
}

/// Get single newsletter issue (author or admin)
/// GET /api/admin/newsletters/:id
pub async fn get_newsletter(
    _auth_user: AuthorUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.db.connect()?;

    let mut rows = conn
        .query(
            "SELECT id, title, subject, body, status, author_id, scheduled_at, sent_at, recipient_count, created_at, updated_at
             FROM newsletter_issues WHERE id = ?",
            params![id],
        )
        .await?;

    let row = rows
        .next()
        .await?
        .ok_or_else(|| AppError::NotFound("Newsletter not found".to_string()))?;

    let id: String = row.get(0)?;
    let title: String = row.get(1)?;
    let subject: String = row.get(2)?;
    let body: String = row.get(3)?;
    let status: String = row.get(4)?;
    let author_id: String = row.get(5)?;
    let scheduled_at: Option<String> = row.get(6)?;
    let sent_at: Option<String> = row.get(7)?;
    let recipient_count: i32 = row.get(8)?;
    let created_at: String = row.get(9)?;
    let updated_at: String = row.get(10)?;

    let issue = NewsletterIssueResponse {
        id: Uuid::parse_str(&id)?,
        title,
        subject,
        body,
        status,
        author_id: Uuid::parse_str(&author_id)?,
        scheduled_at: scheduled_at.and_then(|s| s.parse().ok()),
        sent_at: sent_at.and_then(|s| s.parse().ok()),
        recipient_count,
        created_at: created_at.parse()?,
        updated_at: updated_at.parse()?,
    };

    Ok(ApiResponse::success(json!({ "newsletter": issue })))
}

/// Update newsletter issue (author or admin)
/// PUT /api/admin/newsletters/:id
pub async fn update_newsletter(
    _auth_user: AuthorUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateNewsletterIssue>,
) -> Result<impl IntoResponse, AppError> {
    // Validate request
    payload.validate()?;

    let conn = state.db.connect()?;

    // Check if newsletter exists
    let mut rows = conn
        .query(
            "SELECT id FROM newsletter_issues WHERE id = ?",
            params![id.clone()],
        )
        .await?;

    if rows.next().await?.is_none() {
        return Err(AppError::NotFound("Newsletter not found".to_string()));
    }

    // Build update query dynamically
    let mut updates = Vec::new();
    let mut params_vec: Vec<String> = Vec::new();

    if let Some(title) = payload.title {
        updates.push("title = ?");
        params_vec.push(title);
    }

    if let Some(subject) = payload.subject {
        updates.push("subject = ?");
        params_vec.push(subject);
    }

    if let Some(body) = payload.body {
        updates.push("body = ?");
        params_vec.push(body);
    }

    if let Some(status) = payload.status {
        updates.push("status = ?");
        params_vec.push(status);
    }

    if let Some(scheduled_at) = payload.scheduled_at {
        updates.push("scheduled_at = ?");
        params_vec.push(scheduled_at.to_rfc3339());
    }

    if updates.is_empty() {
        return Err(AppError::BadRequest("No fields to update".to_string()));
    }

    updates.push("updated_at = ?");
    params_vec.push(Utc::now().to_rfc3339());

    let query = format!(
        "UPDATE newsletter_issues SET {} WHERE id = ?",
        updates.join(", ")
    );

    params_vec.push(id.clone());

    // Convert Vec<String> to params
    let params_refs: Vec<&str> = params_vec.iter().map(|s| s.as_str()).collect();
    conn.execute(&query, libsql::params_from_iter(params_refs))
        .await?;

    Ok(ApiResponse::success(json!({
        "message": "Newsletter updated successfully",
        "id": id
    })))
}

/// Delete newsletter issue (author or admin)
/// DELETE /api/admin/newsletters/:id
pub async fn delete_newsletter(
    _auth_user: AuthorUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.db.connect()?;

    let result = conn
        .execute(
            "DELETE FROM newsletter_issues WHERE id = ?",
            params![id.clone()],
        )
        .await?;

    if result == 0 {
        return Err(AppError::NotFound("Newsletter not found".to_string()));
    }

    Ok(ApiResponse::success(json!({
        "message": "Newsletter deleted successfully"
    })))
}

/// Send newsletter to all confirmed subscribers (author or admin)
/// POST /api/admin/newsletters/:id/send
pub async fn send_newsletter(
    _auth_user: AuthorUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.db.connect()?;

    // Get newsletter
    let mut rows = conn
        .query(
            "SELECT id, title, subject, body, status FROM newsletter_issues WHERE id = ?",
            params![id.clone()],
        )
        .await?;

    let row = rows
        .next()
        .await?
        .ok_or_else(|| AppError::NotFound("Newsletter not found".to_string()))?;

    let status: String = row.get(4)?;

    // Check if already sent
    if status == "sent" {
        return Err(AppError::BadRequest(
            "Newsletter has already been sent".to_string(),
        ));
    }

    // Get all confirmed subscribers
    let mut subscribers_rows = conn
        .query(
            "SELECT id, email, name FROM newsletter_subscribers WHERE confirmed = 1 AND unsubscribed_at IS NULL",
            params![],
        )
        .await?;

    // Collect subscriber IDs
    let mut subscriber_ids = Vec::new();
    while let Some(row) = subscribers_rows.next().await? {
        let subscriber_id: String = row.get(0)?;
        subscriber_ids.push(subscriber_id);
    }

    let subscriber_count = subscriber_ids.len();

    // TODO: Implement actual email sending logic
    // For now, we'll just create delivery logs with "sent" status

    // Update newsletter status to "sent"
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE newsletter_issues SET status = 'sent', sent_at = ?, recipient_count = ?, updated_at = ? WHERE id = ?",
        params![now.clone(), subscriber_count as i32, now.clone(), id.clone()],
    )
    .await?;

    // Create delivery logs for all subscribers
    for subscriber_id in subscriber_ids {
        let log_id = Uuid::new_v4();

        conn.execute(
            "INSERT INTO newsletter_delivery_logs (id, issue_id, subscriber_id, status, sent_at, created_at) VALUES (?, ?, ?, 'sent', ?, ?)",
            params![log_id.to_string(), id.clone(), subscriber_id, now.clone(), now.clone()],
        )
        .await?;
    }

    Ok(ApiResponse::success(json!({
        "message": format!("Newsletter sent to {} subscribers", subscriber_count),
        "recipient_count": subscriber_count
    })))
}

/// Get newsletter statistics (author or admin)
/// GET /api/admin/newsletters/stats
pub async fn get_newsletter_stats(
    _auth_user: AuthorUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.db.connect()?;

    // Get total subscribers
    let mut total_rows = conn
        .query(
            "SELECT COUNT(*) FROM newsletter_subscribers WHERE unsubscribed_at IS NULL",
            params![],
        )
        .await?;
    let total_subscribers: i32 = total_rows.next().await?.unwrap().get(0)?;

    // Get confirmed subscribers
    let mut confirmed_rows = conn
        .query(
            "SELECT COUNT(*) FROM newsletter_subscribers WHERE confirmed = 1 AND unsubscribed_at IS NULL",
            params![],
        )
        .await?;
    let confirmed_subscribers: i32 = confirmed_rows.next().await?.unwrap().get(0)?;

    // Get total issues
    let mut issues_rows = conn
        .query("SELECT COUNT(*) FROM newsletter_issues", params![])
        .await?;
    let total_issues: i32 = issues_rows.next().await?.unwrap().get(0)?;

    // Get sent issues
    let mut sent_rows = conn
        .query(
            "SELECT COUNT(*) FROM newsletter_issues WHERE status = 'sent'",
            params![],
        )
        .await?;
    let sent_issues: i32 = sent_rows.next().await?.unwrap().get(0)?;

    let stats = NewsletterStats {
        total_subscribers,
        confirmed_subscribers,
        total_issues,
        sent_issues,
    };

    Ok(ApiResponse::success(json!({ "stats": stats })))
}

/// Newsletter subscription page
/// GET /newsletter
pub async fn newsletter_page(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    let mut context = tera::Context::new();
    context.insert("title", "Newsletter Subscription");

    let html = state
        .templates
        .render("newsletter/subscribe.html", &context)?;

    Ok(Html(html))
}

/// Newsletter confirmation success page
/// GET /newsletter/confirmed (used via redirect from API)
pub async fn newsletter_confirmed_page(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Html<String>, AppError> {
    let conn = state.db.connect()?;

    // Get subscriber email from token
    let mut rows = conn
        .query(
            "SELECT email FROM newsletter_subscribers WHERE confirmation_token = ?",
            params![token],
        )
        .await?;

    let email = if let Some(row) = rows.next().await? {
        row.get::<String>(0)?
    } else {
        return Err(AppError::NotFound("Invalid confirmation link".to_string()));
    };

    let mut context = tera::Context::new();
    context.insert("title", "Subscription Confirmed");
    context.insert("email", &email);

    let html = state
        .templates
        .render("newsletter/confirmed.html", &context)?;

    Ok(Html(html))
}

/// Newsletter unsubscribe success page
/// GET /newsletter/unsubscribed (used via redirect from API)
pub async fn newsletter_unsubscribed_page(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Html<String>, AppError> {
    let conn = state.db.connect()?;

    // Get subscriber email from token
    let mut rows = conn
        .query(
            "SELECT email FROM newsletter_subscribers WHERE unsubscribe_token = ?",
            params![token],
        )
        .await?;

    let email = if let Some(row) = rows.next().await? {
        row.get::<String>(0)?
    } else {
        return Err(AppError::NotFound("Invalid unsubscribe link".to_string()));
    };

    let mut context = tera::Context::new();
    context.insert("title", "Unsubscribed");
    context.insert("email", &email);

    let html = state
        .templates
        .render("newsletter/unsubscribed.html", &context)?;

    Ok(Html(html))
}

/// Admin newsletter management page
/// GET /admin/newsletters
pub async fn admin_newsletters_page(
    auth_user: AuthorUser,
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Get user info for template
    let mut user_rows = conn
        .query(
            "SELECT username, email, bio, image FROM users WHERE id = ?",
            libsql::params![auth_user.user_id.to_string()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let user_info = if let Some(row) = user_rows
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

        serde_json::json!({
            "username": username,
            "email": email,
            "bio": bio,
            "image": image,
        })
    } else {
        serde_json::json!(null)
    };

    let mut context = tera::Context::new();
    context.insert("title", "Newsletter Management");
    context.insert("user", &user_info);
    context.insert("current_year", &chrono::Utc::now().year());

    let html = state.templates.render("admin/newsletters.html", &context)?;

    Ok(Html(html))
}
