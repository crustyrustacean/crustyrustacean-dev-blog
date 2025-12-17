// src/lib/routes/newsletters.rs

use crate::auth::AuthorUser;
use crate::email::{AuthorArticleSummary, NewsletterIssueParams};
use crate::errors::ApiError;
use crate::models::{
    CreateNewsletterIssue, NewsletterIssueResponse, NewsletterStats, SubscribeRequest,
    UpdateNewsletterIssue,
};
use crate::response::ApiResponse;
use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
};
use chrono::{Datelike, Utc};
use libsql::params;
use serde_json::json;
use tracing::{error, info, warn};
use uuid::Uuid;
use validator::Validate;

/// Helper to construct base URL from headers
fn get_base_url(headers: &HeaderMap) -> String {
    let host = headers
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost:8000");

    // In production, use HTTPS; for localhost, use HTTP
    if host.contains("localhost") || host.contains("127.0.0.1") {
        format!("http://{}", host)
    } else {
        format!("https://{}", host)
    }
}

/// Subscribe to newsletter
/// POST /api/newsletters/subscribe
pub async fn subscribe(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SubscribeRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    payload.validate()?;

    let conn = state.db.connect()?;
    let email = payload.email.to_lowercase();
    let name = payload.name;
    let base_url = get_base_url(&headers);

    // Check if email already subscribed
    let mut existing = conn
        .query(
            "SELECT id, confirmed, unsubscribed_at, confirmation_token FROM newsletter_subscribers WHERE email = ?",
            params![email.clone()],
        )
        .await?;

    if let Some(row) = existing.next().await? {
        let confirmed: i64 = row.get(1)?;
        let unsubscribed_at: Option<String> = row.get(2)?;
        let existing_token: Option<String> = row.get(3)?;

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
                params![confirmation_token.clone(), Utc::now().to_rfc3339(), email.clone()],
            )
            .await?;

            // Send confirmation email
            if let Err(e) = state
                .email
                .send_newsletter_confirmation(
                    &email,
                    name.as_deref(),
                    &confirmation_token,
                    &base_url,
                )
                .await
            {
                warn!(
                    "Failed to send newsletter confirmation email to {}: {}",
                    email, e
                );
            } else {
                info!("Sent newsletter re-subscription confirmation to {}", email);
            }

            return Ok(ApiResponse::success(json!({
                "message": "Please check your email to confirm your subscription.",
                "email": email
            })));
        }

        // If not confirmed yet, resend confirmation email
        if let Some(token) = existing_token {
            if let Err(e) = state
                .email
                .send_newsletter_confirmation(&email, name.as_deref(), &token, &base_url)
                .await
            {
                warn!(
                    "Failed to resend newsletter confirmation email to {}: {}",
                    email, e
                );
            } else {
                info!("Resent newsletter confirmation to {}", email);
            }
        }

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
            confirmation_token.clone(),
            unsubscribe_token,
            now.clone(),
            now.clone(),
            now.clone()
        ],
    )
    .await?;

    // Send confirmation email
    if let Err(e) = state
        .email
        .send_newsletter_confirmation(&email, name.as_deref(), &confirmation_token, &base_url)
        .await
    {
        warn!(
            "Failed to send newsletter confirmation email to {}: {}",
            email, e
        );
    } else {
        info!("Sent newsletter confirmation email to {}", email);
    }

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
) -> Result<impl IntoResponse, ApiError> {
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
        .ok_or_else(|| ApiError::NotFound("Invalid confirmation token".to_string()))?;

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
) -> Result<impl IntoResponse, ApiError> {
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
        .ok_or_else(|| ApiError::NotFound("Invalid unsubscribe token".to_string()))?;

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
) -> Result<impl IntoResponse, ApiError> {
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
) -> Result<impl IntoResponse, ApiError> {
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
) -> Result<impl IntoResponse, ApiError> {
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
        .ok_or_else(|| ApiError::NotFound("Newsletter not found".to_string()))?;

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
) -> Result<impl IntoResponse, ApiError> {
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
        return Err(ApiError::NotFound("Newsletter not found".to_string()));
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
        return Err(ApiError::BadRequest("No fields to update".to_string()));
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
) -> Result<impl IntoResponse, ApiError> {
    let conn = state.db.connect()?;

    let result = conn
        .execute(
            "DELETE FROM newsletter_issues WHERE id = ?",
            params![id.clone()],
        )
        .await?;

    if result == 0 {
        return Err(ApiError::NotFound("Newsletter not found".to_string()));
    }

    Ok(ApiResponse::success(json!({
        "message": "Newsletter deleted successfully"
    })))
}

/// List all newsletter subscribers (admin only)
/// GET /api/admin/newsletters/subscribers
pub async fn list_subscribers(
    _auth_user: AuthorUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let conn = state.db.connect()?;

    let mut rows = conn
        .query(
            r"SELECT id, email, name, confirmed, subscribed_at, confirmed_at, unsubscribed_at
              FROM newsletter_subscribers
              ORDER BY subscribed_at DESC",
            params![],
        )
        .await?;

    let mut subscribers = Vec::new();
    while let Some(row) = rows.next().await? {
        let id: String = row.get(0)?;
        let email: String = row.get(1)?;
        let name: Option<String> = row.get(2)?;
        let confirmed: i64 = row.get(3)?;
        let subscribed_at: String = row.get(4)?;
        let confirmed_at: Option<String> = row.get(5)?;
        let unsubscribed_at: Option<String> = row.get(6)?;

        subscribers.push(crate::models::SubscriberResponse {
            id: Uuid::parse_str(&id)?,
            email,
            name,
            confirmed: confirmed == 1,
            subscribed_at: subscribed_at.parse()?,
            confirmed_at: confirmed_at.and_then(|s| s.parse().ok()),
            unsubscribed_at: unsubscribed_at.and_then(|s| s.parse().ok()),
        });
    }

    Ok(ApiResponse::success(json!({ "subscribers": subscribers })))
}

/// Delete a subscriber (admin only)
/// DELETE /api/admin/newsletters/subscribers/:id
pub async fn delete_subscriber(
    _auth_user: AuthorUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Attempting to delete subscriber with id: {}", id);

    let conn = state.db.connect()?;

    // Delete the subscriber directly (delivery logs will be deleted by CASCADE)
    let result = match conn
        .execute(
            "DELETE FROM newsletter_subscribers WHERE id = ?",
            params![id.clone()],
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            error!("Database error deleting subscriber {}: {:?}", id, e);
            return Err(ApiError::from(e));
        }
    };

    if result == 0 {
        return Err(ApiError::NotFound("Subscriber not found".to_string()));
    }

    info!("Subscriber {} deleted by admin", id);

    Ok(ApiResponse::success(json!({
        "message": "Subscriber deleted successfully"
    })))
}

/// Subscriber info for sending newsletters
struct SubscriberInfo {
    id: String,
    email: String,
    name: Option<String>,
    unsubscribe_token: String,
}

/// Send newsletter to all confirmed subscribers (author or admin)
/// POST /api/admin/newsletters/:id/send
pub async fn send_newsletter(
    _auth_user: AuthorUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let conn = state.db.connect()?;
    let base_url = get_base_url(&headers);

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
        .ok_or_else(|| ApiError::NotFound("Newsletter not found".to_string()))?;

    let newsletter_title: String = row.get(1)?;
    let newsletter_subject: String = row.get(2)?;
    let newsletter_body: String = row.get(3)?;
    let status: String = row.get(4)?;

    // Check if already sent
    if status == "sent" {
        return Err(ApiError::BadRequest(
            "Newsletter has already been sent".to_string(),
        ));
    }

    // Update newsletter status to "sending"
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE newsletter_issues SET status = 'sending', updated_at = ? WHERE id = ?",
        params![now.clone(), id.clone()],
    )
    .await?;

    // Get all confirmed subscribers who haven't received this newsletter yet (idempotency)
    let mut subscribers_rows = conn
        .query(
            r"SELECT ns.id, ns.email, ns.name, ns.unsubscribe_token
              FROM newsletter_subscribers ns
              WHERE ns.confirmed = 1
              AND ns.unsubscribed_at IS NULL
              AND NOT EXISTS (
                  SELECT 1 FROM newsletter_delivery_logs dl
                  WHERE dl.subscriber_id = ns.id
                  AND dl.issue_id = ?
              )",
            params![id.clone()],
        )
        .await?;

    // Collect subscriber info
    let mut subscribers: Vec<SubscriberInfo> = Vec::new();
    while let Some(row) = subscribers_rows.next().await? {
        subscribers.push(SubscriberInfo {
            id: row.get(0)?,
            email: row.get(1)?,
            name: row.get(2).ok(),
            unsubscribe_token: row.get(3)?,
        });
    }

    let total_subscribers = subscribers.len();
    let mut sent_count = 0;
    let mut failed_count = 0;

    // Send emails to each subscriber
    for subscriber in subscribers {
        // Get personalized content: recent articles from authors this subscriber's user follows
        // Note: Newsletter subscribers may not have a user account, so this is optional
        let author_articles = get_followed_author_articles(&conn, &subscriber.email, &base_url)
            .await
            .unwrap_or_default();

        let author_articles_ref: Option<&[AuthorArticleSummary]> = if author_articles.is_empty() {
            None
        } else {
            Some(&author_articles)
        };

        // Create delivery log entry with "pending" status
        let log_id = Uuid::new_v4();
        let send_time = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO newsletter_delivery_logs (id, issue_id, subscriber_id, status, created_at) VALUES (?, ?, ?, 'pending', ?)",
            params![log_id.to_string(), id.clone(), subscriber.id.clone(), send_time.clone()],
        )
        .await?;

        // Send the email
        match state
            .email
            .send_newsletter_issue(NewsletterIssueParams {
                to_email: &subscriber.email,
                subscriber_name: subscriber.name.as_deref(),
                subject: &newsletter_subject,
                newsletter_title: &newsletter_title,
                newsletter_body: &newsletter_body,
                unsubscribe_token: &subscriber.unsubscribe_token,
                base_url: &base_url,
                author_articles: author_articles_ref,
            })
            .await
        {
            Ok(_) => {
                // Update delivery log to "sent"
                conn.execute(
                    "UPDATE newsletter_delivery_logs SET status = 'sent', sent_at = ? WHERE id = ?",
                    params![Utc::now().to_rfc3339(), log_id.to_string()],
                )
                .await?;
                sent_count += 1;
                info!("Newsletter {} sent to {}", id, subscriber.email);
            }
            Err(e) => {
                // Update delivery log to "failed" with error message
                conn.execute(
                    "UPDATE newsletter_delivery_logs SET status = 'failed', error_message = ? WHERE id = ?",
                    params![e.to_string(), log_id.to_string()],
                )
                .await?;
                failed_count += 1;
                error!(
                    "Failed to send newsletter {} to {}: {}",
                    id, subscriber.email, e
                );
            }
        }
    }

    // Update newsletter status based on results
    let final_status = if failed_count == 0 {
        "sent"
    } else if sent_count == 0 {
        "failed"
    } else {
        "sent" // Partial success still counts as sent
    };

    conn.execute(
        "UPDATE newsletter_issues SET status = ?, sent_at = ?, recipient_count = ?, updated_at = ? WHERE id = ?",
        params![final_status, Utc::now().to_rfc3339(), sent_count, Utc::now().to_rfc3339(), id.clone()],
    )
    .await?;

    let message = if failed_count > 0 {
        format!(
            "Newsletter sent to {} of {} subscribers ({} failed)",
            sent_count, total_subscribers, failed_count
        )
    } else {
        format!("Newsletter sent to {} subscribers", sent_count)
    };

    info!("{}", message);

    Ok(ApiResponse::success(json!({
        "message": message,
        "recipient_count": sent_count,
        "failed_count": failed_count
    })))
}

/// Get recent articles from authors that a subscriber follows (via their user account)
/// Returns empty vec if subscriber has no linked user account or doesn't follow anyone
async fn get_followed_author_articles(
    conn: &libsql::Connection,
    subscriber_email: &str,
    base_url: &str,
) -> Result<Vec<AuthorArticleSummary>, libsql::Error> {
    // Try to find a user account with this email that follows some authors
    // Then get recent articles from those followed authors (last 30 days)
    let mut rows = conn
        .query(
            r"SELECT a.title, a.description, a.slug, u.username
              FROM articles a
              JOIN users u ON a.author_id = u.id
              JOIN user_follows uf ON uf.following_id = u.id
              JOIN users subscriber_user ON uf.follower_id = subscriber_user.id
              WHERE subscriber_user.email = ?
              AND a.draft = 0
              AND datetime(a.created_at) >= datetime('now', '-30 days')
              ORDER BY a.created_at DESC
              LIMIT 5",
            params![subscriber_email],
        )
        .await?;

    let mut articles = Vec::new();
    while let Some(row) = rows.next().await? {
        let title: String = row.get(0)?;
        let description: String = row.get(1)?;
        let slug: String = row.get(2)?;
        let author_name: String = row.get(3)?;

        articles.push(AuthorArticleSummary {
            title,
            description,
            author_name,
            url: format!("{}/articles/{}", base_url, slug),
        });
    }

    Ok(articles)
}

/// Get newsletter statistics (author or admin)
/// GET /api/admin/newsletters/stats
pub async fn get_newsletter_stats(
    _auth_user: AuthorUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
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
pub async fn newsletter_page(State(state): State<AppState>) -> Result<Html<String>, ApiError> {
    let mut context = tera::Context::new();
    context.insert("title", "Newsletter Subscription");

    let html = state
        .templates
        .render("newsletter/subscribe.html", &context)?;

    Ok(Html(html))
}

/// Newsletter confirmation success page
/// GET /newsletter/confirmed/{token}
/// This page both confirms the subscription AND displays the success page
pub async fn newsletter_confirmed_page(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Html<String>, ApiError> {
    let conn = state.db.connect()?;

    // Get subscriber info from token
    let mut rows = conn
        .query(
            "SELECT id, email, confirmed FROM newsletter_subscribers WHERE confirmation_token = ?",
            params![token],
        )
        .await?;

    let row = rows
        .next()
        .await?
        .ok_or_else(|| ApiError::NotFound("Invalid confirmation link".to_string()))?;

    let id: String = row.get(0)?;
    let email: String = row.get(1)?;
    let confirmed: i64 = row.get(2)?;

    // If not already confirmed, confirm now
    if confirmed != 1 {
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE newsletter_subscribers SET confirmed = 1, confirmed_at = ?, updated_at = ? WHERE id = ?",
            params![now.clone(), now, id],
        )
        .await?;
        info!("Newsletter subscription confirmed for {}", email);
    }

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
) -> Result<Html<String>, ApiError> {
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
        return Err(ApiError::NotFound("Invalid unsubscribe link".to_string()));
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
) -> Result<Html<String>, ApiError> {
    let conn = state
        .db
        .connect()
        .map_err(ApiError::from_connection_error)?;

    // Get user info for template
    let mut user_rows = conn
        .query(
            "SELECT username, email, bio, image FROM users WHERE id = ?",
            libsql::params![auth_user.user_id.to_string()],
        )
        .await?;

    let user_info = if let Some(row) = user_rows.next().await? {
        let username: String = row.get(0)?;
        let email: String = row.get(1)?;
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
