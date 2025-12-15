// src/lib/repositories/comment.rs

//! Comment repository trait and related types.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::UserProfile;

use super::error::RepoResult;

/// Data required to create a new comment.
#[derive(Debug, Clone)]
pub struct NewComment {
    pub body: String,
    pub author_id: Uuid,
    pub article_id: Uuid,
}

/// A comment record from the database.
#[derive(Debug, Clone)]
pub struct CommentRecord {
    pub id: Uuid,
    pub body: String,
    pub author_id: Uuid,
    pub article_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Comment with author profile.
#[derive(Debug, Clone)]
pub struct CommentWithAuthor {
    pub comment: CommentRecord,
    pub author: UserProfile,
}

/// Repository trait for comment data access operations.
#[async_trait]
pub trait CommentRepository: Send + Sync {
    // ========================================================================
    // Create Operations
    // ========================================================================

    /// Create a new comment.
    async fn create(&self, comment: &NewComment) -> RepoResult<CommentRecord>;

    // ========================================================================
    // Read Operations
    // ========================================================================

    /// Find a comment by its ID.
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<CommentRecord>>;

    /// Get a comment with author profile.
    async fn get_with_author(
        &self,
        id: Uuid,
        current_user_id: Option<Uuid>,
    ) -> RepoResult<Option<CommentWithAuthor>>;

    /// List comments for an article.
    async fn list_for_article(
        &self,
        article_id: Uuid,
        current_user_id: Option<Uuid>,
    ) -> RepoResult<Vec<CommentWithAuthor>>;

    /// Count comments for an article.
    async fn count_for_article(&self, article_id: Uuid) -> RepoResult<i32>;

    // ========================================================================
    // Delete Operations
    // ========================================================================

    /// Delete a comment by ID.
    async fn delete(&self, id: Uuid) -> RepoResult<()>;

    /// Delete all comments for an article.
    async fn delete_for_article(&self, article_id: Uuid) -> RepoResult<i32>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_comment() {
        let new_comment = NewComment {
            body: "Great article!".to_string(),
            author_id: Uuid::new_v4(),
            article_id: Uuid::new_v4(),
        };

        assert_eq!(new_comment.body, "Great article!");
    }
}
