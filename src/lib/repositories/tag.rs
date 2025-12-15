// src/lib/repositories/tag.rs

//! Tag repository trait and related types.

use async_trait::async_trait;
use uuid::Uuid;

use super::error::RepoResult;

/// A tag record from the database.
#[derive(Debug, Clone)]
pub struct TagRecord {
    pub id: Uuid,
    pub name: String,
}

/// Repository trait for tag data access operations.
#[async_trait]
pub trait TagRepository: Send + Sync {
    // ========================================================================
    // Create Operations
    // ========================================================================

    /// Create a new tag. Returns existing tag if name already exists.
    async fn create_or_get(&self, name: &str) -> RepoResult<TagRecord>;

    // ========================================================================
    // Read Operations
    // ========================================================================

    /// Find a tag by its ID.
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<TagRecord>>;

    /// Find a tag by its name.
    async fn find_by_name(&self, name: &str) -> RepoResult<Option<TagRecord>>;

    /// List all tags.
    async fn list_all(&self) -> RepoResult<Vec<String>>;

    /// List tags with article counts.
    async fn list_with_counts(&self) -> RepoResult<Vec<(String, i32)>>;

    // ========================================================================
    // Update Operations
    // ========================================================================

    /// Rename a tag.
    async fn rename(&self, old_name: &str, new_name: &str) -> RepoResult<TagRecord>;

    // ========================================================================
    // Delete Operations
    // ========================================================================

    /// Delete a tag by name.
    async fn delete(&self, name: &str) -> RepoResult<()>;

    /// Delete orphaned tags (tags with no articles).
    async fn delete_orphaned(&self) -> RepoResult<i32>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_record() {
        let tag = TagRecord {
            id: Uuid::new_v4(),
            name: "rust".to_string(),
        };

        assert_eq!(tag.name, "rust");
    }
}
