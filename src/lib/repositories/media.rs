// src/lib/repositories/media.rs

//! Media repository trait and related types.

use async_trait::async_trait;

use crate::models::{Media, MediaQuery};

use super::error::RepoResult;

/// Data for creating a new media record.
#[derive(Debug, Clone)]
pub struct NewMedia {
    pub user_id: String,
    pub filename: String,
    pub storage_path: String,
    pub title: Option<String>,
    pub alt_text: Option<String>,
    pub mime_type: String,
    pub file_size: i64,
    pub width: Option<i64>,
    pub height: Option<i64>,
}

/// Data for updating media metadata.
#[derive(Debug, Clone, Default)]
pub struct UpdateMediaData {
    pub title: Option<String>,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
    pub description: Option<String>,
}

/// Repository trait for media data access operations.
#[async_trait]
pub trait MediaRepository: Send + Sync {
    /// Create a new media record.
    async fn create(&self, media: &NewMedia) -> RepoResult<Media>;

    /// Find media by ID.
    async fn find_by_id(&self, id: &str) -> RepoResult<Option<Media>>;

    /// List media for a user with optional filtering.
    async fn list_for_user(&self, user_id: &str, query: &MediaQuery) -> RepoResult<Vec<Media>>;

    /// Count media for a user with optional filtering.
    async fn count_for_user(&self, user_id: &str, query: &MediaQuery) -> RepoResult<i64>;

    /// Update media metadata.
    async fn update(&self, id: &str, data: &UpdateMediaData) -> RepoResult<Media>;

    /// Delete media by ID.
    async fn delete(&self, id: &str) -> RepoResult<()>;

    /// Track media usage in an article.
    async fn track_usage(
        &self,
        media_id: &str,
        article_slug: &str,
        context: Option<&str>,
    ) -> RepoResult<()>;

    /// Remove usage tracking for an article.
    async fn remove_usage(&self, article_slug: &str) -> RepoResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_media() {
        let media = NewMedia {
            user_id: "user-123".to_string(),
            filename: "image.jpg".to_string(),
            storage_path: "media/user-123/2024/01/image.jpg".to_string(),
            title: Some("My Image".to_string()),
            alt_text: None,
            mime_type: "image/jpeg".to_string(),
            file_size: 1024,
            width: Some(800),
            height: Some(600),
        };

        assert_eq!(media.filename, "image.jpg");
    }
}
