// src/lib/routes/newsletters.rs

use crate::auth::AuthorUser;
use crate::email::{AuthorArticleSummary, NewsletterIssueParams};
use crate::errors::ApiError;
use crate::models::{
    CreateNewsletterIssue, NewsletterIssueResponse, NewsletterStats, SubscribeRequest,
    SubscriberResponse, UpdateNewsletterIssue,
};
use crate::repositories::{NewNewsletterIssue, NewSubscriber, UpdateNewsletterIssueData};
use crate::response::ApiResponse;
use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
};
use chrono::Datelike;
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

    let email = payload.email.to_lowercase();
    let name = payload.name;
    let base_url = get_base_url(&headers);

    // Check if email already subscribed
    if let Some(existing) = state.newsletters.find_subscriber_by_email(&email).await? {
        // If already confirmed and not unsubscribed
        if existing.confirmed && existing.unsubscribed_at.is_none() {
            return Ok(ApiResponse::success(json!({
                "message": "You are already subscribed to our newsletter!",
                "email": email
            })));
        }

        // If unsubscribed, allow re-subscription (requires direct DB for updating tokens)
        if existing.unsubscribed_at.is_some() {
            let conn = state.db.connect()?;
            let confirmation_token = Uuid::new_v4().to_string();
            conn.execute(
                "UPDATE newsletter_subscribers SET confirmation_token = ?, unsubscribed_at = NULL, confirmed = 0, updated_at = ? WHERE email = ?",
                params![confirmation_token.clone(), chrono::Utc::now().to_rfc3339(), email.clone()],
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
        if let Some(token) = existing.confirmation_token {
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
    let new_sub = NewSubscriber {
        email: email.clone(),
        name: name.clone(),
    };

    let subscriber = state.newsletters.subscribe(&new_sub).await?;

    // Send confirmation email
    if let Some(token) = &subscriber.confirmation_token {
        if let Err(e) = state
            .email
            .send_newsletter_confirmation(&email, name.as_deref(), token, &base_url)
            .await
        {
            warn!("Failed to send newsletter confirmation email to {}: {}", email, e);
        } else {
            info!("Sent newsletter confirmation email to {}", email);
        }
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
    // The repository clears the token, but we need to keep it for idempotency
    // Use direct DB access for this specific case
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
    let now = chrono::Utc::now().to_rfc3339();
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
    let now = chrono::Utc::now().to_rfc3339();
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
    let issues = state.newsletters.list_issues().await?;

    let response: Vec<NewsletterIssueResponse> = issues
        .into_iter()
        .map(|i| NewsletterIssueResponse {
            id: i.id,
            title: i.title,
            subject: i.subject,
            body: i.body,
            status: i.status.as_str().to_string(),
            author_id: i.author_id,
            scheduled_at: i.scheduled_at,
            sent_at: i.sent_at,
            recipient_count: i.recipient_count,
            created_at: i.created_at,
            updated_at: i.updated_at,
        })
        .collect();

    Ok(ApiResponse::success(json!({ "newsletters": response })))
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

    let new_issue = NewNewsletterIssue {
        title: payload.title,
        subject: payload.subject,
        body: payload.body,
        author_id: auth_user.user_id,
        scheduled_at: payload.scheduled_at,
    };

    let issue = state.newsletters.create_issue(&new_issue).await?;

    Ok(ApiResponse::success_with_status(
        StatusCode::CREATED,
        json!({
            "message": "Newsletter created successfully",
            "id": issue.id
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
    let id_uuid = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid newsletter ID".to_string()))?;

    let issue = state
        .newsletters
        .find_issue_by_id(id_uuid)
        .await?
        .ok_or_else(|| ApiError::NotFound("Newsletter not found".to_string()))?;

    let response = NewsletterIssueResponse {
        id: issue.id,
        title: issue.title,
        subject: issue.subject,
        body: issue.body,
        status: issue.status.as_str().to_string(),
        author_id: issue.author_id,
        scheduled_at: issue.scheduled_at,
        sent_at: issue.sent_at,
        recipient_count: issue.recipient_count,
        created_at: issue.created_at,
        updated_at: issue.updated_at,
    };

    Ok(ApiResponse::success(json!({ "newsletter": response })))
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

    let id_uuid = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid newsletter ID".to_string()))?;

    // Check if at least one field is being updated
    if payload.title.is_none()
        && payload.subject.is_none()
        && payload.body.is_none()
        && payload.status.is_none()
        && payload.scheduled_at.is_none()
    {
        return Err(ApiError::BadRequest("No fields to update".to_string()));
    }

    let update_data = UpdateNewsletterIssueData {
        title: payload.title,
        subject: payload.subject,
        body: payload.body,
        status: payload.status.and_then(|s| s.parse().ok()),
        scheduled_at: payload.scheduled_at.map(Some),
    };

    state.newsletters.update_issue(id_uuid, &update_data).await?;

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
    let id_uuid = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid newsletter ID".to_string()))?;

    state.newsletters.delete_issue(id_uuid).await?;

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
    let subscribers = state.newsletters.list_all_subscribers().await?;

    let response: Vec<SubscriberResponse> = subscribers
        .into_iter()
        .map(|s| SubscriberResponse {
            id: s.id,
            email: s.email,
            name: s.name,
            confirmed: s.confirmed,
            subscribed_at: s.subscribed_at,
            confirmed_at: s.confirmed_at,
            unsubscribed_at: s.unsubscribed_at,
        })
        .collect();

    Ok(ApiResponse::success(json!({ "subscribers": response })))
}

/// Delete a subscriber (admin only)
/// DELETE /api/admin/newsletters/subscribers/:id
pub async fn delete_subscriber(
    _auth_user: AuthorUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Attempting to delete subscriber with id: {}", id);

    let id_uuid = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid subscriber ID".to_string()))?;

    state.newsletters.delete_subscriber(id_uuid).await?;

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

    let id_uuid = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid newsletter ID".to_string()))?;

    // Get newsletter
    let issue = state
        .newsletters
        .find_issue_by_id(id_uuid)
        .await?
        .ok_or_else(|| ApiError::NotFound("Newsletter not found".to_string()))?;

    // Check if already sent
    if issue.status.as_str() == "sent" {
        return Err(ApiError::BadRequest(
            "Newsletter has already been sent".to_string(),
        ));
    }

    // Update newsletter status to "sending"
    let now = chrono::Utc::now().to_rfc3339();
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
        let send_time = chrono::Utc::now().to_rfc3339();

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
                subject: &issue.subject,
                newsletter_title: &issue.title,
                newsletter_body: &issue.body,
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
                    params![chrono::Utc::now().to_rfc3339(), log_id.to_string()],
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
    state
        .newsletters
        .mark_issue_sent(id_uuid, sent_count)
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
    let stats_data = state.newsletters.get_stats().await?;

    let stats = NewsletterStats {
        total_subscribers: stats_data.total_subscribers,
        confirmed_subscribers: stats_data.confirmed_subscribers,
        total_issues: stats_data.total_issues,
        sent_issues: stats_data.sent_issues,
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
        let now = chrono::Utc::now().to_rfc3339();
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
    // Get user info for template
    let user_info = state.users.find_by_id(auth_user.user_id).await?.map(|u| {
        json!({
            "username": u.username,
            "email": u.email,
            "bio": u.bio,
            "image": u.image,
        })
    });

    let mut context = tera::Context::new();
    context.insert("title", "Newsletter Management");
    context.insert("user", &user_info);
    context.insert("current_year", &chrono::Utc::now().year());

    let html = state.templates.render("admin/newsletters.html", &context)?;

    Ok(Html(html))
}
