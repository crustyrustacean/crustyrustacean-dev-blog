// src/lib/repositories/error.rs

//! Repository error types and conversions.
//!
//! This module defines the error types used throughout the repository layer,
//! providing a clean abstraction over database-specific errors.

use thiserror::Error;

/// Error type for repository operations.
///
/// This enum provides a database-agnostic way to handle errors from the
/// repository layer, mapping database-specific errors to domain concepts.
#[derive(Debug, Error)]
pub enum RepositoryError {
    /// The requested entity was not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// A unique constraint was violated (e.g., duplicate email or username).
    #[error("Already exists: {0}")]
    AlreadyExists(String),

    /// A foreign key constraint was violated.
    #[error("Invalid reference: {0}")]
    InvalidReference(String),

    /// The provided data failed validation.
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// A database connection error occurred.
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// A general database error occurred.
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// An internal error occurred.
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type alias for repository operations.
pub type RepoResult<T> = Result<T, RepositoryError>;

// Convert from sqlx (Postgres) errors to repository errors
impl From<sqlx::Error> for RepositoryError {
    fn from(err: sqlx::Error) -> Self {
        let err_string = err.to_string();

        // Check for UNIQUE constraint violations (Postgres error code 23505)
        if let sqlx::Error::Database(db_err) = &err {
            if db_err.code().as_deref() == Some("23505") {
                // Extract constraint name if possible for better messages
                let constraint = db_err.constraint().unwrap_or("");
                let message = match constraint {
                    "users_email_key" | "idx_users_email" => "Email already in use",
                    "users_username_key" | "idx_users_username" => "Username already taken",
                    "categories_slug_key" | "idx_categories_slug" => "Category slug already exists",
                    "categories_name_key" => "Category name already exists",
                    "newsletter_subscribers_email_key" => "Email already subscribed",
                    "tags_name_key" | "idx_tags_name" => "Tag already exists",
                    "articles_slug_key" | "idx_articles_slug" => "Article slug already exists",
                    "password_reset_tokens_token_key" | "idx_password_reset_tokens_token" => {
                        "Token already exists"
                    }
                    "email_verification_tokens_token_key" | "idx_email_verification_tokens_token" => {
                        "Token already exists"
                    }
                    "media_usage_article_slug_fkey" => "Media usage already exists",
                    _ => "Duplicate entry",
                };
                return RepositoryError::AlreadyExists(message.to_string());
            }

            // Check for foreign key violations (Postgres error code 23503)
            if db_err.code().as_deref() == Some("23503") {
                return RepositoryError::InvalidReference(
                    "Referenced entity does not exist".to_string(),
                );
            }
        }

        let _ = err_string;
        RepositoryError::DatabaseError(err.to_string())
    }
}

// Convert from libsql errors to repository errors
// (kept for backwards compatibility during the migration)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_error_display() {
        let err = RepositoryError::NotFound("User with id 123".to_string());
        assert_eq!(err.to_string(), "Not found: User with id 123");

        let err = RepositoryError::AlreadyExists("Username 'test'".to_string());
        assert_eq!(err.to_string(), "Already exists: Username 'test'");

        let err = RepositoryError::ValidationError("Email is required".to_string());
        assert_eq!(err.to_string(), "Validation error: Email is required");
    }

    #[test]
    fn test_repo_result_type() {
        let ok_result: RepoResult<i32> = Ok(42);
        assert!(ok_result.is_ok());

        let err_result: RepoResult<i32> = Err(RepositoryError::NotFound("test".to_string()));
        assert!(err_result.is_err());
    }
}
