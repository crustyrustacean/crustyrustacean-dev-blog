// src/lib/repositories/category.rs

//! Category repository trait and related types.

use async_trait::async_trait;
use uuid::Uuid;

use super::error::RepoResult;

/// Data required to create a new category.
#[derive(Debug, Clone)]
pub struct NewCategory {
    pub name: String,
    pub description: Option<String>,
}

/// Data for updating a category.
#[derive(Debug, Clone, Default)]
pub struct UpdateCategoryData {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// A category record from the database.
#[derive(Debug, Clone)]
pub struct CategoryRecord {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

/// Category with article count.
#[derive(Debug, Clone)]
pub struct CategoryWithCount {
    pub category: CategoryRecord,
    pub article_count: i64,
}

/// Repository trait for category data access operations.
#[async_trait]
pub trait CategoryRepository: Send + Sync {
    // ========================================================================
    // Create Operations
    // ========================================================================

    /// Create a new category.
    async fn create(&self, category: &NewCategory) -> RepoResult<CategoryRecord>;

    // ========================================================================
    // Read Operations
    // ========================================================================

    /// Find a category by its ID.
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<CategoryRecord>>;

    /// Find a category by its slug.
    async fn find_by_slug(&self, slug: &str) -> RepoResult<Option<CategoryRecord>>;

    /// Find a category by its name.
    async fn find_by_name(&self, name: &str) -> RepoResult<Option<CategoryRecord>>;

    /// List all categories with article counts.
    async fn list_with_counts(&self) -> RepoResult<Vec<CategoryWithCount>>;

    // ========================================================================
    // Update Operations
    // ========================================================================

    /// Update a category by slug.
    async fn update(&self, slug: &str, data: &UpdateCategoryData) -> RepoResult<CategoryRecord>;

    // ========================================================================
    // Delete Operations
    // ========================================================================

    /// Delete a category by slug.
    async fn delete(&self, slug: &str) -> RepoResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_category() {
        let new_cat = NewCategory {
            name: "Programming".to_string(),
            description: Some("Articles about programming".to_string()),
        };

        assert_eq!(new_cat.name, "Programming");
        assert!(new_cat.description.is_some());
    }

    #[test]
    fn test_update_category_default() {
        let update = UpdateCategoryData::default();

        assert!(update.name.is_none());
        assert!(update.description.is_none());
    }
}
