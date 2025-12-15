// src/lib/repositories/newsletter.rs

//! Newsletter repository trait and related types.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::{NewsletterIssue, NewsletterStatus, NewsletterSubscriber};

use super::error::RepoResult;

/// Data for creating a new subscriber.
#[derive(Debug, Clone)]
pub struct NewSubscriber {
    pub email: String,
    pub name: Option<String>,
}

/// Data for creating a new newsletter issue.
#[derive(Debug, Clone)]
pub struct NewNewsletterIssue {
    pub title: String,
    pub subject: String,
    pub body: String,
    pub author_id: Uuid,
    pub scheduled_at: Option<DateTime<Utc>>,
}

/// Data for updating a newsletter issue.
#[derive(Debug, Clone, Default)]
pub struct UpdateNewsletterIssueData {
    pub title: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub status: Option<NewsletterStatus>,
    pub scheduled_at: Option<Option<DateTime<Utc>>>,
}

/// Newsletter statistics.
#[derive(Debug, Clone)]
pub struct NewsletterStatsData {
    pub total_subscribers: i32,
    pub confirmed_subscribers: i32,
    pub total_issues: i32,
    pub sent_issues: i32,
}

/// Repository trait for newsletter data access operations.
#[async_trait]
pub trait NewsletterRepository: Send + Sync {
    // ========================================================================
    // Subscriber Operations
    // ========================================================================

    /// Subscribe a new email address.
    async fn subscribe(&self, subscriber: &NewSubscriber) -> RepoResult<NewsletterSubscriber>;

    /// Confirm a subscription using the confirmation token.
    async fn confirm_subscription(&self, token: &str) -> RepoResult<NewsletterSubscriber>;

    /// Unsubscribe using the unsubscribe token.
    async fn unsubscribe(&self, token: &str) -> RepoResult<()>;

    /// Find a subscriber by email.
    async fn find_subscriber_by_email(&self, email: &str) -> RepoResult<Option<NewsletterSubscriber>>;

    /// Find a subscriber by ID.
    async fn find_subscriber_by_id(&self, id: Uuid) -> RepoResult<Option<NewsletterSubscriber>>;

    /// List all confirmed subscribers.
    async fn list_confirmed_subscribers(&self) -> RepoResult<Vec<NewsletterSubscriber>>;

    /// List all subscribers (for admin).
    async fn list_all_subscribers(&self) -> RepoResult<Vec<NewsletterSubscriber>>;

    /// Delete a subscriber by ID.
    async fn delete_subscriber(&self, id: Uuid) -> RepoResult<()>;

    // ========================================================================
    // Issue Operations
    // ========================================================================

    /// Create a new newsletter issue.
    async fn create_issue(&self, issue: &NewNewsletterIssue) -> RepoResult<NewsletterIssue>;

    /// Find an issue by ID.
    async fn find_issue_by_id(&self, id: Uuid) -> RepoResult<Option<NewsletterIssue>>;

    /// List all issues.
    async fn list_issues(&self) -> RepoResult<Vec<NewsletterIssue>>;

    /// Update an issue.
    async fn update_issue(&self, id: Uuid, data: &UpdateNewsletterIssueData) -> RepoResult<NewsletterIssue>;

    /// Delete an issue.
    async fn delete_issue(&self, id: Uuid) -> RepoResult<()>;

    /// Mark an issue as sent with recipient count.
    async fn mark_issue_sent(&self, id: Uuid, recipient_count: i32) -> RepoResult<()>;

    // ========================================================================
    // Delivery Operations
    // ========================================================================

    /// Log a delivery attempt.
    async fn log_delivery(&self, issue_id: Uuid, subscriber_id: Uuid, status: &str, error: Option<&str>) -> RepoResult<()>;

    // ========================================================================
    // Statistics
    // ========================================================================

    /// Get newsletter statistics.
    async fn get_stats(&self) -> RepoResult<NewsletterStatsData>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_subscriber() {
        let subscriber = NewSubscriber {
            email: "test@example.com".to_string(),
            name: Some("Test User".to_string()),
        };

        assert_eq!(subscriber.email, "test@example.com");
        assert!(subscriber.name.is_some());
    }
}
