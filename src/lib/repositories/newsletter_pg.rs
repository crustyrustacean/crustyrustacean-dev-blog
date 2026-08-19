// src/lib/repositories/newsletter_pg.rs

//! PostgreSQL implementation of the NewsletterRepository trait.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::newsletter::{NewsletterIssue, NewsletterStatus, NewsletterSubscriber};

use super::error::{RepoResult, RepositoryError};
use super::newsletter::{
    NewNewsletterIssue, NewSubscriber, NewsletterRepository, NewsletterStatsData,
    UpdateNewsletterIssueData,
};

/// PostgreSQL implementation of the NewsletterRepository.
#[derive(Debug, Clone)]
pub struct PgNewsletterRepository {
    db: sqlx::PgPool,
}

impl PgNewsletterRepository {
    /// Create a new PostgreSQL newsletter repository.
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }
}

fn status_from_str(s: &str) -> RepoResult<NewsletterStatus> {
    match s {
        "draft" => Ok(NewsletterStatus::Draft),
        "scheduled" => Ok(NewsletterStatus::Scheduled),
        "sending" => Ok(NewsletterStatus::Sending),
        "sent" => Ok(NewsletterStatus::Sent),
        "failed" => Ok(NewsletterStatus::Failed),
        other => Err(RepositoryError::InternalError(format!(
            "Invalid newsletter status: {}",
            other
        ))),
    }
}

fn status_as_str(status: &NewsletterStatus) -> &str {
    match status {
        NewsletterStatus::Draft => "draft",
        NewsletterStatus::Scheduled => "scheduled",
        NewsletterStatus::Sending => "sending",
        NewsletterStatus::Sent => "sent",
        NewsletterStatus::Failed => "failed",
    }
}

// (id, email, name, confirmation_token, unsubscribe_token, confirmed,
//  subscribed_at, confirmed_at, unsubscribed_at, created_at, updated_at)
type SubscriberRow = (
    Uuid,
    String,
    Option<String>,
    Option<String>,
    String,
    bool,
    DateTime<Utc>,
    Option<DateTime<Utc>>,
    Option<DateTime<Utc>>,
    DateTime<Utc>,
    DateTime<Utc>,
);

fn subscriber_from_row(row: SubscriberRow) -> NewsletterSubscriber {
    NewsletterSubscriber {
        id: row.0,
        email: row.1,
        name: row.2,
        confirmation_token: row.3,
        unsubscribe_token: row.4,
        confirmed: row.5,
        subscribed_at: row.6,
        confirmed_at: row.7,
        unsubscribed_at: row.8,
        created_at: row.9,
        updated_at: row.10,
    }
}

const SUBSCRIBER_COLUMNS: &str = "id, email, name, confirmation_token, unsubscribe_token, \
                                 confirmed, subscribed_at, confirmed_at, unsubscribed_at, \
                                 created_at, updated_at";

// (id, title, subject, body, status, author_id, scheduled_at, sent_at,
//  recipient_count, created_at, updated_at)
type IssueRow = (
    Uuid,
    String,
    String,
    String,
    String,
    Uuid,
    Option<DateTime<Utc>>,
    Option<DateTime<Utc>>,
    i32,
    DateTime<Utc>,
    DateTime<Utc>,
);

fn issue_from_row(row: IssueRow) -> RepoResult<NewsletterIssue> {
    Ok(NewsletterIssue {
        id: row.0,
        title: row.1,
        subject: row.2,
        body: row.3,
        status: status_from_str(&row.4)?,
        author_id: row.5,
        scheduled_at: row.6,
        sent_at: row.7,
        recipient_count: row.8,
        created_at: row.9,
        updated_at: row.10,
    })
}

const ISSUE_COLUMNS: &str =
    "id, title, subject, body, status, author_id, scheduled_at, sent_at, \
     recipient_count, created_at, updated_at";

#[async_trait]
impl NewsletterRepository for PgNewsletterRepository {
    // ------------------------------------------------------------------
    // Subscribers
    // ------------------------------------------------------------------

    async fn subscribe(&self, subscriber: &NewSubscriber) -> RepoResult<NewsletterSubscriber> {
        // Tokens are generated here; re-subscribing refreshes them and
        // restarts the confirmation flow.
        let confirmation_token = Uuid::new_v4().to_string();
        let unsubscribe_token = Uuid::new_v4().to_string();

        let row = sqlx::query_as::<_, SubscriberRow>(&format!(
            r"INSERT INTO newsletter_subscribers
                   (email, name, confirmation_token, unsubscribe_token)
              VALUES ($1, $2, $3, $4)
              ON CONFLICT (email) DO UPDATE SET
                  name = EXCLUDED.name,
                  confirmation_token = EXCLUDED.confirmation_token,
                  unsubscribe_token = EXCLUDED.unsubscribe_token,
                  confirmed = FALSE,
                  confirmed_at = NULL,
                  unsubscribed_at = NULL,
                  updated_at = NOW()
              RETURNING {SUBSCRIBER_COLUMNS}",
            SUBSCRIBER_COLUMNS = SUBSCRIBER_COLUMNS
        ))
        .bind(&subscriber.email)
        .bind(&subscriber.name)
        .bind(&confirmation_token)
        .bind(&unsubscribe_token)
        .fetch_one(&self.db)
        .await?;

        Ok(subscriber_from_row(row))
    }

    async fn confirm_subscription(&self, token: &str) -> RepoResult<NewsletterSubscriber> {
        let row = sqlx::query_as::<_, SubscriberRow>(&format!(
            r"UPDATE newsletter_subscribers
              SET confirmed = TRUE, confirmed_at = NOW(),
                  confirmation_token = NULL, updated_at = NOW()
              WHERE confirmation_token = $1 AND confirmed = FALSE
              RETURNING {SUBSCRIBER_COLUMNS}",
            SUBSCRIBER_COLUMNS = SUBSCRIBER_COLUMNS
        ))
        .bind(token)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| {
            RepositoryError::NotFound("Invalid or already used confirmation token".to_string())
        })?;

        Ok(subscriber_from_row(row))
    }

    async fn unsubscribe(&self, token: &str) -> RepoResult<()> {
        let result = sqlx::query(
            "UPDATE newsletter_subscribers \
             SET confirmed = FALSE, unsubscribed_at = NOW(), updated_at = NOW() \
             WHERE unsubscribe_token = $1 AND confirmed = TRUE",
        )
        .bind(token)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(
                "Invalid unsubscribe token or already unsubscribed".to_string(),
            ));
        }

        Ok(())
    }

    async fn find_subscriber_by_email(
        &self,
        email: &str,
    ) -> RepoResult<Option<NewsletterSubscriber>> {
        let row = sqlx::query_as::<_, SubscriberRow>(&format!(
            "SELECT {SUBSCRIBER_COLUMNS} FROM newsletter_subscribers WHERE email = $1",
            SUBSCRIBER_COLUMNS = SUBSCRIBER_COLUMNS
        ))
        .bind(email)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(subscriber_from_row))
    }

    async fn find_subscriber_by_id(&self, id: Uuid) -> RepoResult<Option<NewsletterSubscriber>> {
        let row = sqlx::query_as::<_, SubscriberRow>(&format!(
            "SELECT {SUBSCRIBER_COLUMNS} FROM newsletter_subscribers WHERE id = $1",
            SUBSCRIBER_COLUMNS = SUBSCRIBER_COLUMNS
        ))
        .bind(id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(subscriber_from_row))
    }

    async fn list_confirmed_subscribers(&self) -> RepoResult<Vec<NewsletterSubscriber>> {
        let rows = sqlx::query_as::<_, SubscriberRow>(&format!(
            "SELECT {SUBSCRIBER_COLUMNS} FROM newsletter_subscribers \
             WHERE confirmed = TRUE AND unsubscribed_at IS NULL \
             ORDER BY subscribed_at ASC",
            SUBSCRIBER_COLUMNS = SUBSCRIBER_COLUMNS
        ))
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().map(subscriber_from_row).collect())
    }

    async fn list_all_subscribers(&self) -> RepoResult<Vec<NewsletterSubscriber>> {
        let rows = sqlx::query_as::<_, SubscriberRow>(&format!(
            "SELECT {SUBSCRIBER_COLUMNS} FROM newsletter_subscribers ORDER BY subscribed_at DESC",
            SUBSCRIBER_COLUMNS = SUBSCRIBER_COLUMNS
        ))
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().map(subscriber_from_row).collect())
    }

    async fn delete_subscriber(&self, id: Uuid) -> RepoResult<()> {
        let result = sqlx::query("DELETE FROM newsletter_subscribers WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!("Subscriber {}", id)));
        }

        Ok(())
    }

    // ------------------------------------------------------------------
    // Issues
    // ------------------------------------------------------------------

    async fn create_issue(&self, issue: &NewNewsletterIssue) -> RepoResult<NewsletterIssue> {
        let row = sqlx::query_as::<_, IssueRow>(&format!(
            r"INSERT INTO newsletter_issues (author_id, title, subject, body, scheduled_at)
              VALUES ($1, $2, $3, $4, $5)
              RETURNING {ISSUE_COLUMNS}",
            ISSUE_COLUMNS = ISSUE_COLUMNS
        ))
        .bind(issue.author_id)
        .bind(&issue.title)
        .bind(&issue.subject)
        .bind(&issue.body)
        .bind(issue.scheduled_at)
        .fetch_one(&self.db)
        .await?;

        issue_from_row(row)
    }

    async fn find_issue_by_id(&self, id: Uuid) -> RepoResult<Option<NewsletterIssue>> {
        let row = sqlx::query_as::<_, IssueRow>(&format!(
            "SELECT {ISSUE_COLUMNS} FROM newsletter_issues WHERE id = $1",
            ISSUE_COLUMNS = ISSUE_COLUMNS
        ))
        .bind(id)
        .fetch_optional(&self.db)
        .await?;

        row.map(issue_from_row).transpose()
    }

    async fn list_issues(&self) -> RepoResult<Vec<NewsletterIssue>> {
        let rows = sqlx::query_as::<_, IssueRow>(&format!(
            "SELECT {ISSUE_COLUMNS} FROM newsletter_issues ORDER BY created_at DESC",
            ISSUE_COLUMNS = ISSUE_COLUMNS
        ))
        .fetch_all(&self.db)
        .await?;

        rows.into_iter().map(issue_from_row).collect()
    }

    async fn update_issue(
        &self,
        id: Uuid,
        data: &UpdateNewsletterIssueData,
    ) -> RepoResult<NewsletterIssue> {
        // Each SET clause gets a sequential bind slot; we bind values in the
        // same order the clauses were pushed.
        enum Bind {
            Text(String),
            Status(String),
            ScheduledAt(Option<DateTime<Utc>>),
        }

        let mut updates: Vec<String> = Vec::new();
        let mut binds: Vec<Bind> = Vec::new();

        let push = |fragment: &str, bind: Bind, updates: &mut Vec<String>, binds: &mut Vec<Bind>| {
            let n = binds.len() + 2; // $1 is the id
            updates.push(format!("{} = ${}", fragment, n));
            binds.push(bind);
        };

        if let Some(title) = &data.title {
            push("title", Bind::Text(title.clone()), &mut updates, &mut binds);
        }
        if let Some(subject) = &data.subject {
            push("subject", Bind::Text(subject.clone()), &mut updates, &mut binds);
        }
        if let Some(body) = &data.body {
            push("body", Bind::Text(body.clone()), &mut updates, &mut binds);
        }
        if let Some(status) = &data.status {
            push(
                "status",
                Bind::Status(status_as_str(status).to_string()),
                &mut updates,
                &mut binds,
            );
        }
        if let Some(scheduled_at) = data.scheduled_at {
            push(
                "scheduled_at",
                Bind::ScheduledAt(scheduled_at),
                &mut updates,
                &mut binds,
            );
        }

        if updates.is_empty() {
            return self
                .find_issue_by_id(id)
                .await?
                .ok_or_else(|| RepositoryError::NotFound(format!("Newsletter issue {}", id)));
        }

        updates.push("updated_at = NOW()".to_string());

        let sql = format!(
            "UPDATE newsletter_issues SET {} WHERE id = $1 RETURNING {ISSUE_COLUMNS}",
            updates.join(", "),
            ISSUE_COLUMNS = ISSUE_COLUMNS
        );

        let mut query = sqlx::query_as::<_, IssueRow>(&sql).bind(id);
        for bind in binds {
            query = match bind {
                Bind::Text(v) => query.bind(v),
                Bind::Status(v) => query.bind(v),
                Bind::ScheduledAt(v) => query.bind(v),
            };
        }

        let row = query
            .fetch_optional(&self.db)
            .await?
            .ok_or_else(|| RepositoryError::NotFound(format!("Newsletter issue {}", id)))?;

        issue_from_row(row)
    }

    async fn delete_issue(&self, id: Uuid) -> RepoResult<()> {
        let result = sqlx::query("DELETE FROM newsletter_issues WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!("Newsletter issue {}", id)));
        }

        Ok(())
    }

    async fn mark_issue_sent(&self, id: Uuid, recipient_count: i32) -> RepoResult<()> {
        let result = sqlx::query(
            "UPDATE newsletter_issues \
             SET status = 'sent', sent_at = NOW(), recipient_count = $2, updated_at = NOW() \
             WHERE id = $1",
        )
        .bind(id)
        .bind(recipient_count)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!("Newsletter issue {}", id)));
        }

        Ok(())
    }

    // ------------------------------------------------------------------
    // Delivery
    // ------------------------------------------------------------------

    async fn log_delivery(
        &self,
        issue_id: Uuid,
        subscriber_id: Uuid,
        status: &str,
        error: Option<&str>,
    ) -> RepoResult<()> {
        sqlx::query(
            "INSERT INTO newsletter_delivery_logs (issue_id, subscriber_id, status, error_message) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(issue_id)
        .bind(subscriber_id)
        .bind(status)
        .bind(error)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    // ------------------------------------------------------------------
    // Stats
    // ------------------------------------------------------------------

    async fn get_stats(&self) -> RepoResult<NewsletterStatsData> {
        let stats = sqlx::query_as::<_, (i32, i32, i32, i32)>(
            r"SELECT
                 (SELECT COUNT(*)::INT FROM newsletter_subscribers) AS total_subscribers,
                 (SELECT COUNT(*)::INT FROM newsletter_subscribers
                  WHERE confirmed = TRUE AND unsubscribed_at IS NULL) AS confirmed_subscribers,
                 (SELECT COUNT(*)::INT FROM newsletter_issues) AS total_issues,
                 (SELECT COUNT(*)::INT FROM newsletter_issues WHERE status = 'sent') AS sent_issues",
        )
        .fetch_one(&self.db)
        .await?;

        Ok(NewsletterStatsData {
            total_subscribers: stats.0,
            confirmed_subscribers: stats.1,
            total_issues: stats.2,
            sent_issues: stats.3,
        })
    }
}
