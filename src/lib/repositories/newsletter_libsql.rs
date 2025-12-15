// src/lib/repositories/newsletter_libsql.rs

//! LibSQL implementation of the NewsletterRepository trait.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::database::DatabaseConnection;
use crate::models::{NewsletterIssue, NewsletterStatus, NewsletterSubscriber};

use super::error::{RepoResult, RepositoryError};
use super::newsletter::{
    NewNewsletterIssue, NewSubscriber, NewsletterRepository, NewsletterStatsData,
    UpdateNewsletterIssueData,
};

/// LibSQL implementation of the NewsletterRepository.
#[derive(Debug, Clone)]
pub struct LibSqlNewsletterRepository {
    db: DatabaseConnection,
}

impl LibSqlNewsletterRepository {
    /// Create a new LibSQL newsletter repository.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn parse_datetime(s: &str) -> RepoResult<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| ndt.and_utc())
            })
            .map_err(|e| RepositoryError::InternalError(format!("Failed to parse datetime: {}", e)))
    }
}

#[async_trait]
impl NewsletterRepository for LibSqlNewsletterRepository {
    async fn subscribe(&self, subscriber: &NewSubscriber) -> RepoResult<NewsletterSubscriber> {
        let conn = self.db.connect().map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let id = Uuid::new_v4();
        let confirmation_token = Uuid::new_v4().to_string();
        let unsubscribe_token = Uuid::new_v4().to_string();
        let now = Utc::now();

        let name_param = match &subscriber.name {
            Some(n) => libsql::Value::Text(n.clone()),
            None => libsql::Value::Null,
        };

        conn.execute(
            r"INSERT INTO newsletter_subscribers (id, email, name, confirmation_token, unsubscribe_token, confirmed, subscribed_at, created_at, updated_at)
              VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            vec![
                libsql::Value::Text(id.to_string()),
                libsql::Value::Text(subscriber.email.clone()),
                name_param,
                libsql::Value::Text(confirmation_token),
                libsql::Value::Text(unsubscribe_token),
                libsql::Value::Integer(0),
                libsql::Value::Text(now.to_rfc3339()),
                libsql::Value::Text(now.to_rfc3339()),
                libsql::Value::Text(now.to_rfc3339()),
            ],
        ).await?;

        self.find_subscriber_by_id(id).await?.ok_or_else(|| {
            RepositoryError::InternalError("Failed to create subscriber".to_string())
        })
    }

    async fn confirm_subscription(&self, token: &str) -> RepoResult<NewsletterSubscriber> {
        let conn = self.db.connect().map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let now = Utc::now();

        let result = conn.execute(
            r"UPDATE newsletter_subscribers
              SET confirmed = 1, confirmed_at = ?, confirmation_token = NULL, updated_at = ?
              WHERE confirmation_token = ? AND confirmed = 0",
            libsql::params![now.to_rfc3339(), now.to_rfc3339(), token],
        ).await?;

        if result == 0 {
            return Err(RepositoryError::NotFound("Invalid or already used confirmation token".to_string()));
        }

        // Find the subscriber that was just confirmed
        let mut rows = conn.query(
            "SELECT id FROM newsletter_subscribers WHERE confirmed_at = ?",
            libsql::params![now.to_rfc3339()],
        ).await?;

        if let Some(row) = rows.next().await? {
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str).map_err(|e| RepositoryError::InternalError(e.to_string()))?;
            return self.find_subscriber_by_id(id).await?.ok_or_else(|| {
                RepositoryError::InternalError("Subscriber not found after confirmation".to_string())
            });
        }

        Err(RepositoryError::InternalError("Failed to find confirmed subscriber".to_string()))
    }

    async fn unsubscribe(&self, token: &str) -> RepoResult<()> {
        let conn = self.db.connect().map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let now = Utc::now();

        let result = conn.execute(
            r"UPDATE newsletter_subscribers SET unsubscribed_at = ?, updated_at = ? WHERE unsubscribe_token = ?",
            libsql::params![now.to_rfc3339(), now.to_rfc3339(), token],
        ).await?;

        if result == 0 {
            return Err(RepositoryError::NotFound("Invalid unsubscribe token".to_string()));
        }

        Ok(())
    }

    async fn find_subscriber_by_email(&self, email: &str) -> RepoResult<Option<NewsletterSubscriber>> {
        let conn = self.db.connect().map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn.query(
            r"SELECT id, email, name, confirmation_token, unsubscribe_token, confirmed,
                     subscribed_at, confirmed_at, unsubscribed_at, created_at, updated_at
              FROM newsletter_subscribers WHERE email = ?",
            libsql::params![email],
        ).await?;

        if let Some(row) = rows.next().await? {
            let id_str: String = row.get(0)?;
            let confirmed: i64 = row.get(5)?;
            let subscribed_at_str: String = row.get(6)?;
            let created_at_str: String = row.get(9)?;
            let updated_at_str: String = row.get(10)?;

            Ok(Some(NewsletterSubscriber {
                id: Uuid::parse_str(&id_str).map_err(|e| RepositoryError::InternalError(e.to_string()))?,
                email: row.get(1)?,
                name: row.get(2).ok(),
                confirmation_token: row.get(3).ok(),
                unsubscribe_token: row.get(4)?,
                confirmed: confirmed != 0,
                subscribed_at: Self::parse_datetime(&subscribed_at_str)?,
                confirmed_at: row.get::<String>(7).ok().and_then(|s| Self::parse_datetime(&s).ok()),
                unsubscribed_at: row.get::<String>(8).ok().and_then(|s| Self::parse_datetime(&s).ok()),
                created_at: Self::parse_datetime(&created_at_str)?,
                updated_at: Self::parse_datetime(&updated_at_str)?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn find_subscriber_by_id(&self, id: Uuid) -> RepoResult<Option<NewsletterSubscriber>> {
        let conn = self.db.connect().map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn.query(
            r"SELECT id, email, name, confirmation_token, unsubscribe_token, confirmed,
                     subscribed_at, confirmed_at, unsubscribed_at, created_at, updated_at
              FROM newsletter_subscribers WHERE id = ?",
            libsql::params![id.to_string()],
        ).await?;

        if let Some(row) = rows.next().await? {
            let id_str: String = row.get(0)?;
            let confirmed: i64 = row.get(5)?;
            let subscribed_at_str: String = row.get(6)?;
            let created_at_str: String = row.get(9)?;
            let updated_at_str: String = row.get(10)?;

            Ok(Some(NewsletterSubscriber {
                id: Uuid::parse_str(&id_str).map_err(|e| RepositoryError::InternalError(e.to_string()))?,
                email: row.get(1)?,
                name: row.get(2).ok(),
                confirmation_token: row.get(3).ok(),
                unsubscribe_token: row.get(4)?,
                confirmed: confirmed != 0,
                subscribed_at: Self::parse_datetime(&subscribed_at_str)?,
                confirmed_at: row.get::<String>(7).ok().and_then(|s| Self::parse_datetime(&s).ok()),
                unsubscribed_at: row.get::<String>(8).ok().and_then(|s| Self::parse_datetime(&s).ok()),
                created_at: Self::parse_datetime(&created_at_str)?,
                updated_at: Self::parse_datetime(&updated_at_str)?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_confirmed_subscribers(&self) -> RepoResult<Vec<NewsletterSubscriber>> {
        let conn = self.db.connect().map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn.query(
            r"SELECT id, email, name, confirmation_token, unsubscribe_token, confirmed,
                     subscribed_at, confirmed_at, unsubscribed_at, created_at, updated_at
              FROM newsletter_subscribers
              WHERE confirmed = 1 AND unsubscribed_at IS NULL
              ORDER BY subscribed_at DESC",
            (),
        ).await?;

        let mut subscribers = Vec::new();
        while let Some(row) = rows.next().await? {
            let id_str: String = row.get(0)?;
            let confirmed: i64 = row.get(5)?;
            let subscribed_at_str: String = row.get(6)?;
            let created_at_str: String = row.get(9)?;
            let updated_at_str: String = row.get(10)?;

            subscribers.push(NewsletterSubscriber {
                id: Uuid::parse_str(&id_str).map_err(|e| RepositoryError::InternalError(e.to_string()))?,
                email: row.get(1)?,
                name: row.get(2).ok(),
                confirmation_token: row.get(3).ok(),
                unsubscribe_token: row.get(4)?,
                confirmed: confirmed != 0,
                subscribed_at: Self::parse_datetime(&subscribed_at_str)?,
                confirmed_at: row.get::<String>(7).ok().and_then(|s| Self::parse_datetime(&s).ok()),
                unsubscribed_at: row.get::<String>(8).ok().and_then(|s| Self::parse_datetime(&s).ok()),
                created_at: Self::parse_datetime(&created_at_str)?,
                updated_at: Self::parse_datetime(&updated_at_str)?,
            });
        }

        Ok(subscribers)
    }

    async fn list_all_subscribers(&self) -> RepoResult<Vec<NewsletterSubscriber>> {
        let conn = self.db.connect().map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn.query(
            r"SELECT id, email, name, confirmation_token, unsubscribe_token, confirmed,
                     subscribed_at, confirmed_at, unsubscribed_at, created_at, updated_at
              FROM newsletter_subscribers ORDER BY created_at DESC",
            (),
        ).await?;

        let mut subscribers = Vec::new();
        while let Some(row) = rows.next().await? {
            let id_str: String = row.get(0)?;
            let confirmed: i64 = row.get(5)?;
            let subscribed_at_str: String = row.get(6)?;
            let created_at_str: String = row.get(9)?;
            let updated_at_str: String = row.get(10)?;

            subscribers.push(NewsletterSubscriber {
                id: Uuid::parse_str(&id_str).map_err(|e| RepositoryError::InternalError(e.to_string()))?,
                email: row.get(1)?,
                name: row.get(2).ok(),
                confirmation_token: row.get(3).ok(),
                unsubscribe_token: row.get(4)?,
                confirmed: confirmed != 0,
                subscribed_at: Self::parse_datetime(&subscribed_at_str)?,
                confirmed_at: row.get::<String>(7).ok().and_then(|s| Self::parse_datetime(&s).ok()),
                unsubscribed_at: row.get::<String>(8).ok().and_then(|s| Self::parse_datetime(&s).ok()),
                created_at: Self::parse_datetime(&created_at_str)?,
                updated_at: Self::parse_datetime(&updated_at_str)?,
            });
        }

        Ok(subscribers)
    }

    async fn delete_subscriber(&self, id: Uuid) -> RepoResult<()> {
        let conn = self.db.connect().map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let result = conn.execute(
            "DELETE FROM newsletter_subscribers WHERE id = ?",
            libsql::params![id.to_string()],
        ).await?;

        if result == 0 {
            return Err(RepositoryError::NotFound(format!("Subscriber with id {}", id)));
        }

        Ok(())
    }

    async fn create_issue(&self, issue: &NewNewsletterIssue) -> RepoResult<NewsletterIssue> {
        let conn = self.db.connect().map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let id = Uuid::new_v4();
        let now = Utc::now();

        let scheduled_param = match &issue.scheduled_at {
            Some(dt) => libsql::Value::Text(dt.to_rfc3339()),
            None => libsql::Value::Null,
        };

        conn.execute(
            r"INSERT INTO newsletter_issues (id, title, subject, body, status, author_id, scheduled_at, created_at, updated_at)
              VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            vec![
                libsql::Value::Text(id.to_string()),
                libsql::Value::Text(issue.title.clone()),
                libsql::Value::Text(issue.subject.clone()),
                libsql::Value::Text(issue.body.clone()),
                libsql::Value::Text("draft".to_string()),
                libsql::Value::Text(issue.author_id.to_string()),
                scheduled_param,
                libsql::Value::Text(now.to_rfc3339()),
                libsql::Value::Text(now.to_rfc3339()),
            ],
        ).await?;

        self.find_issue_by_id(id).await?.ok_or_else(|| {
            RepositoryError::InternalError("Failed to create newsletter issue".to_string())
        })
    }

    async fn find_issue_by_id(&self, id: Uuid) -> RepoResult<Option<NewsletterIssue>> {
        let conn = self.db.connect().map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn.query(
            r"SELECT id, title, subject, body, status, author_id, scheduled_at, sent_at, recipient_count, created_at, updated_at
              FROM newsletter_issues WHERE id = ?",
            libsql::params![id.to_string()],
        ).await?;

        if let Some(row) = rows.next().await? {
            let id_str: String = row.get(0)?;
            let status_str: String = row.get(4)?;
            let author_id_str: String = row.get(5)?;
            let recipient_count: i32 = row.get(8).unwrap_or(0);
            let created_at_str: String = row.get(9)?;
            let updated_at_str: String = row.get(10)?;

            Ok(Some(NewsletterIssue {
                id: Uuid::parse_str(&id_str).map_err(|e| RepositoryError::InternalError(e.to_string()))?,
                title: row.get(1)?,
                subject: row.get(2)?,
                body: row.get(3)?,
                status: status_str.parse().unwrap_or(NewsletterStatus::Draft),
                author_id: Uuid::parse_str(&author_id_str).map_err(|e| RepositoryError::InternalError(e.to_string()))?,
                scheduled_at: row.get::<String>(6).ok().and_then(|s| Self::parse_datetime(&s).ok()),
                sent_at: row.get::<String>(7).ok().and_then(|s| Self::parse_datetime(&s).ok()),
                recipient_count,
                created_at: Self::parse_datetime(&created_at_str)?,
                updated_at: Self::parse_datetime(&updated_at_str)?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_issues(&self) -> RepoResult<Vec<NewsletterIssue>> {
        let conn = self.db.connect().map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn.query(
            r"SELECT id, title, subject, body, status, author_id, scheduled_at, sent_at, recipient_count, created_at, updated_at
              FROM newsletter_issues ORDER BY created_at DESC",
            (),
        ).await?;

        let mut issues = Vec::new();
        while let Some(row) = rows.next().await? {
            let id_str: String = row.get(0)?;
            let status_str: String = row.get(4)?;
            let author_id_str: String = row.get(5)?;
            let recipient_count: i32 = row.get(8).unwrap_or(0);
            let created_at_str: String = row.get(9)?;
            let updated_at_str: String = row.get(10)?;

            issues.push(NewsletterIssue {
                id: Uuid::parse_str(&id_str).map_err(|e| RepositoryError::InternalError(e.to_string()))?,
                title: row.get(1)?,
                subject: row.get(2)?,
                body: row.get(3)?,
                status: status_str.parse().unwrap_or(NewsletterStatus::Draft),
                author_id: Uuid::parse_str(&author_id_str).map_err(|e| RepositoryError::InternalError(e.to_string()))?,
                scheduled_at: row.get::<String>(6).ok().and_then(|s| Self::parse_datetime(&s).ok()),
                sent_at: row.get::<String>(7).ok().and_then(|s| Self::parse_datetime(&s).ok()),
                recipient_count,
                created_at: Self::parse_datetime(&created_at_str)?,
                updated_at: Self::parse_datetime(&updated_at_str)?,
            });
        }

        Ok(issues)
    }

    async fn update_issue(&self, id: Uuid, data: &UpdateNewsletterIssueData) -> RepoResult<NewsletterIssue> {
        let conn = self.db.connect().map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut updates = Vec::new();
        let mut params: Vec<libsql::Value> = Vec::new();

        if let Some(title) = &data.title {
            updates.push("title = ?");
            params.push(libsql::Value::Text(title.clone()));
        }
        if let Some(subject) = &data.subject {
            updates.push("subject = ?");
            params.push(libsql::Value::Text(subject.clone()));
        }
        if let Some(body) = &data.body {
            updates.push("body = ?");
            params.push(libsql::Value::Text(body.clone()));
        }
        if let Some(status) = &data.status {
            updates.push("status = ?");
            params.push(libsql::Value::Text(status.as_str().to_string()));
        }
        if let Some(scheduled) = &data.scheduled_at {
            updates.push("scheduled_at = ?");
            params.push(match scheduled {
                Some(dt) => libsql::Value::Text(dt.to_rfc3339()),
                None => libsql::Value::Null,
            });
        }

        if updates.is_empty() {
            return self.find_issue_by_id(id).await?.ok_or_else(|| {
                RepositoryError::NotFound(format!("Newsletter issue with id {}", id))
            });
        }

        updates.push("updated_at = ?");
        params.push(libsql::Value::Text(Utc::now().to_rfc3339()));
        params.push(libsql::Value::Text(id.to_string()));

        let query = format!("UPDATE newsletter_issues SET {} WHERE id = ?", updates.join(", "));
        conn.execute(&query, libsql::params_from_iter(params)).await?;

        self.find_issue_by_id(id).await?.ok_or_else(|| {
            RepositoryError::InternalError("Failed to update newsletter issue".to_string())
        })
    }

    async fn delete_issue(&self, id: Uuid) -> RepoResult<()> {
        let conn = self.db.connect().map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let result = conn.execute(
            "DELETE FROM newsletter_issues WHERE id = ?",
            libsql::params![id.to_string()],
        ).await?;

        if result == 0 {
            return Err(RepositoryError::NotFound(format!("Newsletter issue with id {}", id)));
        }

        Ok(())
    }

    async fn mark_issue_sent(&self, id: Uuid, recipient_count: i32) -> RepoResult<()> {
        let conn = self.db.connect().map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let now = Utc::now();
        conn.execute(
            "UPDATE newsletter_issues SET status = 'sent', sent_at = ?, recipient_count = ?, updated_at = ? WHERE id = ?",
            libsql::params![now.to_rfc3339(), recipient_count, now.to_rfc3339(), id.to_string()],
        ).await?;

        Ok(())
    }

    async fn log_delivery(&self, issue_id: Uuid, subscriber_id: Uuid, status: &str, error: Option<&str>) -> RepoResult<()> {
        let conn = self.db.connect().map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let id = Uuid::new_v4();
        let now = Utc::now();

        let error_param = match error {
            Some(e) => libsql::Value::Text(e.to_string()),
            None => libsql::Value::Null,
        };

        conn.execute(
            r"INSERT INTO newsletter_delivery_logs (id, issue_id, subscriber_id, status, error_message, sent_at, created_at)
              VALUES (?, ?, ?, ?, ?, ?, ?)",
            vec![
                libsql::Value::Text(id.to_string()),
                libsql::Value::Text(issue_id.to_string()),
                libsql::Value::Text(subscriber_id.to_string()),
                libsql::Value::Text(status.to_string()),
                error_param,
                libsql::Value::Text(now.to_rfc3339()),
                libsql::Value::Text(now.to_rfc3339()),
            ],
        ).await?;

        Ok(())
    }

    async fn get_stats(&self) -> RepoResult<NewsletterStatsData> {
        let conn = self.db.connect().map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut total_rows = conn.query("SELECT COUNT(*) FROM newsletter_subscribers WHERE unsubscribed_at IS NULL", ()).await?;
        let total_subscribers: i32 = if let Some(row) = total_rows.next().await? {
            row.get(0).unwrap_or(0)
        } else { 0 };

        let mut confirmed_rows = conn.query("SELECT COUNT(*) FROM newsletter_subscribers WHERE confirmed = 1 AND unsubscribed_at IS NULL", ()).await?;
        let confirmed_subscribers: i32 = if let Some(row) = confirmed_rows.next().await? {
            row.get(0).unwrap_or(0)
        } else { 0 };

        let mut issues_rows = conn.query("SELECT COUNT(*) FROM newsletter_issues", ()).await?;
        let total_issues: i32 = if let Some(row) = issues_rows.next().await? {
            row.get(0).unwrap_or(0)
        } else { 0 };

        let mut sent_rows = conn.query("SELECT COUNT(*) FROM newsletter_issues WHERE status = 'sent'", ()).await?;
        let sent_issues: i32 = if let Some(row) = sent_rows.next().await? {
            row.get(0).unwrap_or(0)
        } else { 0 };

        Ok(NewsletterStatsData {
            total_subscribers,
            confirmed_subscribers,
            total_issues,
            sent_issues,
        })
    }
}
