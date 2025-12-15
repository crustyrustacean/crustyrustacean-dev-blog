// src/lib/repositories/article.rs

//! Article repository trait and related types.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::{ArticleQuery, ArticleResponse, FeedQuery, UserProfile};

use super::error::RepoResult;

/// Data required to create a new article.
#[derive(Debug, Clone)]
pub struct NewArticle {
    pub title: String,
    pub description: String,
    pub body: String,
    pub author_id: Uuid,
    pub category_id: Option<Uuid>,
    pub draft: bool,
}

/// Data for updating an existing article.
#[derive(Debug, Clone, Default)]
pub struct UpdateArticleData {
    pub title: Option<String>,
    pub description: Option<String>,
    pub body: Option<String>,
    pub category_id: Option<Option<Uuid>>, // None = don't update, Some(None) = clear, Some(Some(id)) = set
    pub draft: Option<bool>,
    pub featured_image_id: Option<Option<String>>,
}

/// Core article data from the database.
#[derive(Debug, Clone)]
pub struct ArticleRecord {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub author_id: Uuid,
    pub category_id: Option<Uuid>,
    pub category_slug: Option<String>,
    pub draft: bool,
    pub featured_image_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Full article data with author profile, tags, and favorites info.
#[derive(Debug, Clone)]
pub struct ArticleWithDetails {
    pub article: ArticleRecord,
    pub author: UserProfile,
    pub tag_list: Vec<String>,
    pub favorited: bool,
    pub favorites_count: i32,
}

impl ArticleWithDetails {
    /// Convert to API response format.
    pub fn to_response(self, rendered_body: Option<String>) -> ArticleResponse {
        ArticleResponse {
            slug: self.article.slug,
            title: self.article.title,
            description: self.article.description,
            body: self.article.body,
            rendered_body,
            tag_list: self.tag_list,
            category: self.article.category_slug,
            draft: self.article.draft,
            created_at: self.article.created_at,
            updated_at: self.article.updated_at,
            favorited: self.favorited,
            favorites_count: self.favorites_count,
            author: self.author,
        }
    }
}

/// Repository trait for article data access operations.
#[async_trait]
pub trait ArticleRepository: Send + Sync {
    // ========================================================================
    // Create Operations
    // ========================================================================

    /// Create a new article and return its slug.
    async fn create(&self, article: &NewArticle) -> RepoResult<ArticleRecord>;

    // ========================================================================
    // Read Operations
    // ========================================================================

    /// Find an article by its unique ID.
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<ArticleRecord>>;

    /// Find an article by its slug.
    async fn find_by_slug(&self, slug: &str) -> RepoResult<Option<ArticleRecord>>;

    /// Get an article with all details (author, tags, favorites) for a given user.
    async fn get_with_details(
        &self,
        slug: &str,
        current_user_id: Option<Uuid>,
    ) -> RepoResult<Option<ArticleWithDetails>>;

    /// List articles with filtering and pagination.
    async fn list(
        &self,
        query: &ArticleQuery,
        current_user_id: Option<Uuid>,
        include_drafts: bool,
    ) -> RepoResult<Vec<ArticleWithDetails>>;

    /// Count articles matching the given query.
    async fn count(&self, query: &ArticleQuery, include_drafts: bool) -> RepoResult<i32>;

    /// Get articles from followed users (feed).
    async fn get_feed(
        &self,
        user_id: Uuid,
        query: &FeedQuery,
    ) -> RepoResult<Vec<ArticleWithDetails>>;

    /// Count articles in user's feed.
    async fn count_feed(&self, user_id: Uuid) -> RepoResult<i32>;

    /// List draft articles for a specific user.
    async fn list_user_drafts(&self, user_id: Uuid) -> RepoResult<Vec<ArticleWithDetails>>;

    /// Check if a slug already exists.
    async fn slug_exists(&self, slug: &str) -> RepoResult<bool>;

    // ========================================================================
    // Update Operations
    // ========================================================================

    /// Update an article by slug.
    async fn update(&self, slug: &str, data: &UpdateArticleData) -> RepoResult<ArticleRecord>;

    // ========================================================================
    // Delete Operations
    // ========================================================================

    /// Delete an article by slug.
    async fn delete(&self, slug: &str) -> RepoResult<()>;

    // ========================================================================
    // Tag Operations
    // ========================================================================

    /// Get tags for an article.
    async fn get_tags(&self, article_id: Uuid) -> RepoResult<Vec<String>>;

    /// Set tags for an article (replaces existing tags).
    async fn set_tags(&self, article_id: Uuid, tags: &[String]) -> RepoResult<Vec<String>>;

    /// Clear all tags from an article.
    async fn clear_tags(&self, article_id: Uuid) -> RepoResult<()>;

    // ========================================================================
    // Favorite Operations
    // ========================================================================

    /// Add article to user's favorites.
    async fn favorite(&self, article_id: Uuid, user_id: Uuid) -> RepoResult<()>;

    /// Remove article from user's favorites.
    async fn unfavorite(&self, article_id: Uuid, user_id: Uuid) -> RepoResult<()>;

    /// Check if user has favorited an article.
    async fn is_favorited(&self, article_id: Uuid, user_id: Uuid) -> RepoResult<bool>;

    /// Get favorites count for an article.
    async fn favorites_count(&self, article_id: Uuid) -> RepoResult<i32>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_article_creation() {
        let new_article = NewArticle {
            title: "Test Article".to_string(),
            description: "A test description".to_string(),
            body: "The article body".to_string(),
            author_id: Uuid::new_v4(),
            category_id: None,
            draft: false,
        };

        assert_eq!(new_article.title, "Test Article");
        assert!(!new_article.draft);
    }

    #[test]
    fn test_update_article_default() {
        let update = UpdateArticleData::default();

        assert!(update.title.is_none());
        assert!(update.description.is_none());
        assert!(update.body.is_none());
        assert!(update.draft.is_none());
    }
}
