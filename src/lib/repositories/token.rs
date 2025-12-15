// src/lib/repositories/token.rs

//! Token repository trait for password reset and email verification tokens.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::error::RepoResult;

/// A password reset token record.
#[derive(Debug, Clone)]
pub struct PasswordResetToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
    pub created_at: DateTime<Utc>,
}

/// An email verification token record.
#[derive(Debug, Clone)]
pub struct EmailVerificationToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Repository trait for token data access operations.
#[async_trait]
pub trait TokenRepository: Send + Sync {
    // ========================================================================
    // Password Reset Tokens
    // ========================================================================

    /// Create a new password reset token.
    async fn create_password_reset_token(&self, user_id: Uuid, token: &str, expires_at: DateTime<Utc>) -> RepoResult<PasswordResetToken>;

    /// Find a password reset token by token string.
    async fn find_password_reset_token(&self, token: &str) -> RepoResult<Option<PasswordResetToken>>;

    /// Mark a password reset token as used.
    async fn mark_password_reset_token_used(&self, token: &str) -> RepoResult<()>;

    /// Delete expired password reset tokens.
    async fn delete_expired_password_reset_tokens(&self) -> RepoResult<i32>;

    // ========================================================================
    // Email Verification Tokens
    // ========================================================================

    /// Create a new email verification token.
    async fn create_email_verification_token(&self, user_id: Uuid, token: &str, expires_at: DateTime<Utc>) -> RepoResult<EmailVerificationToken>;

    /// Find an email verification token by token string.
    async fn find_email_verification_token(&self, token: &str) -> RepoResult<Option<EmailVerificationToken>>;

    /// Delete email verification tokens for a user (after verification).
    async fn delete_email_verification_tokens(&self, user_id: Uuid) -> RepoResult<()>;

    /// Delete expired email verification tokens.
    async fn delete_expired_email_verification_tokens(&self) -> RepoResult<i32>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_reset_token() {
        let token = PasswordResetToken {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            token: "abc123".to_string(),
            expires_at: Utc::now(),
            used: false,
            created_at: Utc::now(),
        };

        assert!(!token.used);
    }
}
