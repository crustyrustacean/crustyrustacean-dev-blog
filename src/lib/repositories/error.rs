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

// Convert from libsql errors to repository errors
impl From<libsql::Error> for RepositoryError {
    fn from(err: libsql::Error) -> Self {
        let err_string = err.to_string();

        // Check for UNIQUE constraint violations
        if err_string.contains("UNIQUE constraint failed") {
            // Extract the field name if possible
            if err_string.contains("users.email") {
                return RepositoryError::AlreadyExists("Email already in use".to_string());
            }
            if err_string.contains("users.username") {
                return RepositoryError::AlreadyExists("Username already taken".to_string());
            }
            if err_string.contains("categories.slug") {
                return RepositoryError::AlreadyExists("Category slug already exists".to_string());
            }
            if err_string.contains("categories.name") {
                return RepositoryError::AlreadyExists("Category name already exists".to_string());
            }
            if err_string.contains("newsletter_subscribers.email") {
                return RepositoryError::AlreadyExists("Email already subscribed".to_string());
            }
            return RepositoryError::AlreadyExists("Duplicate entry".to_string());
        }

        // Check for foreign key violations
        if err_string.contains("FOREIGN KEY constraint failed") {
            return RepositoryError::InvalidReference(
                "Referenced entity does not exist".to_string(),
            );
        }

        // Default to database error
        RepositoryError::DatabaseError(err_string)
    }
}

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
