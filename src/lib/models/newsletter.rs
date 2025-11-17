// src/lib/models/newsletter.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// Newsletter subscriber model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsletterSubscriber {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub confirmation_token: Option<String>,
    pub unsubscribe_token: String,
    pub confirmed: bool,
    pub subscribed_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub unsubscribed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Newsletter subscription request (for creating new subscription)
#[derive(Debug, Deserialize, Validate)]
pub struct SubscribeRequest {
    #[validate(email)]
    pub email: String,
    pub name: Option<String>,
}

// Newsletter subscription response
#[derive(Debug, Serialize)]
pub struct SubscribeResponse {
    pub message: String,
    pub email: String,
}

// Newsletter confirmation response
#[derive(Debug, Serialize)]
pub struct ConfirmationResponse {
    pub message: String,
    pub email: String,
}

// Newsletter unsubscribe response
#[derive(Debug, Serialize)]
pub struct UnsubscribeResponse {
    pub message: String,
}

// Newsletter issue model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsletterIssue {
    pub id: Uuid,
    pub title: String,
    pub subject: String,
    pub body: String,
    pub status: NewsletterStatus,
    pub author_id: Uuid,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub recipient_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Newsletter status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NewsletterStatus {
    Draft,
    Scheduled,
    Sending,
    Sent,
    Failed,
}

impl NewsletterStatus {
    pub fn as_str(&self) -> &str {
        match self {
            NewsletterStatus::Draft => "draft",
            NewsletterStatus::Scheduled => "scheduled",
            NewsletterStatus::Sending => "sending",
            NewsletterStatus::Sent => "sent",
            NewsletterStatus::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "draft" => NewsletterStatus::Draft,
            "scheduled" => NewsletterStatus::Scheduled,
            "sending" => NewsletterStatus::Sending,
            "sent" => NewsletterStatus::Sent,
            "failed" => NewsletterStatus::Failed,
            _ => NewsletterStatus::Draft,
        }
    }
}

// Create newsletter issue request
#[derive(Debug, Deserialize, Validate)]
pub struct CreateNewsletterIssue {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    #[validate(length(min = 1, max = 200))]
    pub subject: String,
    #[validate(length(min = 1))]
    pub body: String,
    pub scheduled_at: Option<DateTime<Utc>>,
}

// Update newsletter issue request
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateNewsletterIssue {
    #[validate(length(min = 1, max = 200))]
    pub title: Option<String>,
    #[validate(length(min = 1, max = 200))]
    pub subject: Option<String>,
    #[validate(length(min = 1))]
    pub body: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub status: Option<String>,
}

// Newsletter delivery log model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsletterDeliveryLog {
    pub id: Uuid,
    pub issue_id: Uuid,
    pub subscriber_id: Uuid,
    pub status: DeliveryStatus,
    pub error_message: Option<String>,
    pub sent_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// Delivery status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryStatus {
    Pending,
    Sent,
    Failed,
    Bounced,
}

impl DeliveryStatus {
    pub fn as_str(&self) -> &str {
        match self {
            DeliveryStatus::Pending => "pending",
            DeliveryStatus::Sent => "sent",
            DeliveryStatus::Failed => "failed",
            DeliveryStatus::Bounced => "bounced",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "pending" => DeliveryStatus::Pending,
            "sent" => DeliveryStatus::Sent,
            "failed" => DeliveryStatus::Failed,
            "bounced" => DeliveryStatus::Bounced,
            _ => DeliveryStatus::Pending,
        }
    }
}

// Newsletter issue response (for API responses)
#[derive(Debug, Serialize)]
pub struct NewsletterIssueResponse {
    pub id: Uuid,
    pub title: String,
    pub subject: String,
    pub body: String,
    pub status: String,
    pub author_id: Uuid,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub recipient_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<NewsletterIssue> for NewsletterIssueResponse {
    fn from(issue: NewsletterIssue) -> Self {
        NewsletterIssueResponse {
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
        }
    }
}

// Newsletter statistics (for admin dashboard)
#[derive(Debug, Serialize)]
pub struct NewsletterStats {
    pub total_subscribers: i32,
    pub confirmed_subscribers: i32,
    pub total_issues: i32,
    pub sent_issues: i32,
}
