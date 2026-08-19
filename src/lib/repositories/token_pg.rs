// src/lib/repositories/token_pg.rs

//! PostgreSQL implementation of the TokenRepository trait.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::error::{RepoResult, RepositoryError};
use super::token::{EmailVerificationToken, PasswordResetToken, TokenRepository};

/// PostgreSQL implementation of the TokenRepository.
#[derive(Debug, Clone)]
pub struct PgTokenRepository {
    db: sqlx::PgPool,
}

impl PgTokenRepository {
    /// Create a new PostgreSQL token repository.
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }
}

// (id, user_id, token, expires_at, used, created_at)
type ResetRow = (Uuid, Uuid, String, DateTime<Utc>, bool, DateTime<Utc>);

// (id, user_id, token, expires_at, created_at)
type VerificationRow = (Uuid, Uuid, String, DateTime<Utc>, DateTime<Utc>);

fn reset_from_row(row: ResetRow) -> PasswordResetToken {
    PasswordResetToken {
        id: row.0,
        user_id: row.1,
        token: row.2,
        expires_at: row.3,
        used: row.4,
        created_at: row.5,
    }
}

fn verification_from_row(row: VerificationRow) -> EmailVerificationToken {
    EmailVerificationToken {
        id: row.0,
        user_id: row.1,
        token: row.2,
        expires_at: row.3,
        created_at: row.4,
    }
}

#[async_trait]
impl TokenRepository for PgTokenRepository {
    async fn create_password_reset_token(
        &self,
        user_id: Uuid,
        token: &str,
        expires_at: DateTime<Utc>,
    ) -> RepoResult<PasswordResetToken> {
        let row = sqlx::query_as::<_, ResetRow>(
            "INSERT INTO password_reset_tokens (user_id, token, expires_at) \
             VALUES ($1, $2, $3) \
             RETURNING id, user_id, token, expires_at, used, created_at",
        )
        .bind(user_id)
        .bind(token)
        .bind(expires_at)
        .fetch_one(&self.db)
        .await?;

        Ok(reset_from_row(row))
    }

    async fn find_password_reset_token(
        &self,
        token: &str,
    ) -> RepoResult<Option<PasswordResetToken>> {
        let row = sqlx::query_as::<_, ResetRow>(
            "SELECT id, user_id, token, expires_at, used, created_at \
             FROM password_reset_tokens WHERE token = $1",
        )
        .bind(token)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(reset_from_row))
    }

    async fn mark_password_reset_token_used(&self, token: &str) -> RepoResult<()> {
        let result = sqlx::query("UPDATE password_reset_tokens SET used = TRUE WHERE token = $1")
            .bind(token)
            .execute(&self.db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!(
                "Password reset token '{}'",
                token
            )));
        }

        Ok(())
    }

    async fn delete_expired_password_reset_tokens(&self) -> RepoResult<i32> {
        let result = sqlx::query("DELETE FROM password_reset_tokens WHERE expires_at < NOW()")
            .execute(&self.db)
            .await?;

        Ok(result.rows_affected() as i32)
    }

    async fn create_email_verification_token(
        &self,
        user_id: Uuid,
        token: &str,
        expires_at: DateTime<Utc>,
    ) -> RepoResult<EmailVerificationToken> {
        let row = sqlx::query_as::<_, VerificationRow>(
            "INSERT INTO email_verification_tokens (user_id, token, expires_at) \
             VALUES ($1, $2, $3) \
             RETURNING id, user_id, token, expires_at, created_at",
        )
        .bind(user_id)
        .bind(token)
        .bind(expires_at)
        .fetch_one(&self.db)
        .await?;

        Ok(verification_from_row(row))
    }

    async fn find_email_verification_token(
        &self,
        token: &str,
    ) -> RepoResult<Option<EmailVerificationToken>> {
        let row = sqlx::query_as::<_, VerificationRow>(
            "SELECT id, user_id, token, expires_at, created_at \
             FROM email_verification_tokens WHERE token = $1",
        )
        .bind(token)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(verification_from_row))
    }

    async fn delete_email_verification_tokens(&self, user_id: Uuid) -> RepoResult<()> {
        sqlx::query("DELETE FROM email_verification_tokens WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    async fn delete_expired_email_verification_tokens(&self) -> RepoResult<i32> {
        let result = sqlx::query("DELETE FROM email_verification_tokens WHERE expires_at < NOW()")
            .execute(&self.db)
            .await?;

        Ok(result.rows_affected() as i32)
    }
}
