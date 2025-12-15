// src/lib/repositories/token_libsql.rs

//! LibSQL implementation of the TokenRepository trait.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::database::DatabaseConnection;

use super::error::{RepoResult, RepositoryError};
use super::token::{EmailVerificationToken, PasswordResetToken, TokenRepository};

/// LibSQL implementation of the TokenRepository.
#[derive(Debug, Clone)]
pub struct LibSqlTokenRepository {
    db: DatabaseConnection,
}

impl LibSqlTokenRepository {
    /// Create a new LibSQL token repository.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Parse a datetime string from the database.
    fn parse_datetime(s: &str) -> Result<DateTime<Utc>, RepositoryError> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| RepositoryError::InternalError(format!("Invalid datetime: {}", e)))
    }
}

#[async_trait]
impl TokenRepository for LibSqlTokenRepository {
    // ========================================================================
    // Password Reset Tokens
    // ========================================================================

    async fn create_password_reset_token(
        &self,
        user_id: Uuid,
        token: &str,
        expires_at: DateTime<Utc>,
    ) -> RepoResult<PasswordResetToken> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let id = Uuid::new_v4();
        let now = Utc::now();

        conn.execute(
            r"INSERT INTO password_reset_tokens (id, user_id, token, expires_at, used, created_at)
              VALUES (?, ?, ?, ?, 0, ?)",
            vec![
                libsql::Value::Text(id.to_string()),
                libsql::Value::Text(user_id.to_string()),
                libsql::Value::Text(token.to_string()),
                libsql::Value::Text(expires_at.to_rfc3339()),
                libsql::Value::Text(now.to_rfc3339()),
            ],
        )
        .await?;

        Ok(PasswordResetToken {
            id,
            user_id,
            token: token.to_string(),
            expires_at,
            used: false,
            created_at: now,
        })
    }

    async fn find_password_reset_token(
        &self,
        token: &str,
    ) -> RepoResult<Option<PasswordResetToken>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                r"SELECT id, user_id, token, expires_at, used, created_at
                  FROM password_reset_tokens WHERE token = ?",
                libsql::params![token],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid UUID: {}", e)))?;

            let user_id_str: String = row.get(1)?;
            let user_id = Uuid::parse_str(&user_id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid UUID: {}", e)))?;

            let token: String = row.get(2)?;

            let expires_at_str: String = row.get(3)?;
            let expires_at = Self::parse_datetime(&expires_at_str)?;

            let used: i64 = row.get(4)?;

            let created_at_str: String = row.get(5)?;
            let created_at = Self::parse_datetime(&created_at_str)?;

            Ok(Some(PasswordResetToken {
                id,
                user_id,
                token,
                expires_at,
                used: used != 0,
                created_at,
            }))
        } else {
            Ok(None)
        }
    }

    async fn mark_password_reset_token_used(&self, token: &str) -> RepoResult<()> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let result = conn
            .execute(
                "UPDATE password_reset_tokens SET used = 1 WHERE token = ?",
                libsql::params![token],
            )
            .await?;

        if result == 0 {
            return Err(RepositoryError::NotFound(
                "Password reset token".to_string(),
            ));
        }

        Ok(())
    }

    async fn delete_expired_password_reset_tokens(&self) -> RepoResult<i32> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let now = Utc::now();

        let result = conn
            .execute(
                "DELETE FROM password_reset_tokens WHERE expires_at < ? OR used = 1",
                libsql::params![now.to_rfc3339()],
            )
            .await?;

        Ok(result as i32)
    }

    // ========================================================================
    // Email Verification Tokens
    // ========================================================================

    async fn create_email_verification_token(
        &self,
        user_id: Uuid,
        token: &str,
        expires_at: DateTime<Utc>,
    ) -> RepoResult<EmailVerificationToken> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let id = Uuid::new_v4();
        let now = Utc::now();

        conn.execute(
            r"INSERT INTO email_verification_tokens (id, user_id, token, expires_at, created_at)
              VALUES (?, ?, ?, ?, ?)",
            vec![
                libsql::Value::Text(id.to_string()),
                libsql::Value::Text(user_id.to_string()),
                libsql::Value::Text(token.to_string()),
                libsql::Value::Text(expires_at.to_rfc3339()),
                libsql::Value::Text(now.to_rfc3339()),
            ],
        )
        .await?;

        Ok(EmailVerificationToken {
            id,
            user_id,
            token: token.to_string(),
            expires_at,
            created_at: now,
        })
    }

    async fn find_email_verification_token(
        &self,
        token: &str,
    ) -> RepoResult<Option<EmailVerificationToken>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                r"SELECT id, user_id, token, expires_at, created_at
                  FROM email_verification_tokens WHERE token = ?",
                libsql::params![token],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid UUID: {}", e)))?;

            let user_id_str: String = row.get(1)?;
            let user_id = Uuid::parse_str(&user_id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid UUID: {}", e)))?;

            let token: String = row.get(2)?;

            let expires_at_str: String = row.get(3)?;
            let expires_at = Self::parse_datetime(&expires_at_str)?;

            let created_at_str: String = row.get(4)?;
            let created_at = Self::parse_datetime(&created_at_str)?;

            Ok(Some(EmailVerificationToken {
                id,
                user_id,
                token,
                expires_at,
                created_at,
            }))
        } else {
            Ok(None)
        }
    }

    async fn delete_email_verification_tokens(&self, user_id: Uuid) -> RepoResult<()> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        conn.execute(
            "DELETE FROM email_verification_tokens WHERE user_id = ?",
            libsql::params![user_id.to_string()],
        )
        .await?;

        Ok(())
    }

    async fn delete_expired_email_verification_tokens(&self) -> RepoResult<i32> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let now = Utc::now();

        let result = conn
            .execute(
                "DELETE FROM email_verification_tokens WHERE expires_at < ?",
                libsql::params![now.to_rfc3339()],
            )
            .await?;

        Ok(result as i32)
    }
}
