// src/lib/repositories/tag_libsql.rs

//! LibSQL implementation of the TagRepository trait.

use async_trait::async_trait;
use uuid::Uuid;

use crate::database::DatabaseConnection;

use super::error::{RepoResult, RepositoryError};
use super::tag::{TagRecord, TagRepository};

/// LibSQL implementation of the TagRepository.
#[derive(Debug, Clone)]
pub struct LibSqlTagRepository {
    db: DatabaseConnection,
}

impl LibSqlTagRepository {
    /// Create a new LibSQL tag repository.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl TagRepository for LibSqlTagRepository {
    async fn create_or_get(&self, name: &str) -> RepoResult<TagRecord> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // Try to insert, ignore if exists
        let id = Uuid::new_v4();
        conn.execute(
            "INSERT OR IGNORE INTO tags (id, name) VALUES (?, ?)",
            libsql::params![id.to_string(), name],
        )
        .await?;

        // Get the tag (either newly created or existing)
        self.find_by_name(name)
            .await?
            .ok_or_else(|| RepositoryError::InternalError("Failed to create or get tag".to_string()))
    }

    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<TagRecord>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                "SELECT id, name FROM tags WHERE id = ?",
                libsql::params![id.to_string()],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid UUID: {}", e)))?;
            let name: String = row.get(1)?;

            Ok(Some(TagRecord { id, name }))
        } else {
            Ok(None)
        }
    }

    async fn find_by_name(&self, name: &str) -> RepoResult<Option<TagRecord>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                "SELECT id, name FROM tags WHERE name = ?",
                libsql::params![name],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid UUID: {}", e)))?;
            let name: String = row.get(1)?;

            Ok(Some(TagRecord { id, name }))
        } else {
            Ok(None)
        }
    }

    async fn list_all(&self) -> RepoResult<Vec<String>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query("SELECT name FROM tags ORDER BY name", ())
            .await?;

        let mut tags = Vec::new();
        while let Some(row) = rows.next().await? {
            let name: String = row.get(0)?;
            tags.push(name);
        }

        Ok(tags)
    }

    async fn list_with_counts(&self) -> RepoResult<Vec<(String, i32)>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                r"SELECT t.name, COUNT(at.article_id) as article_count
                  FROM tags t
                  LEFT JOIN article_tags at ON t.id = at.tag_id
                  LEFT JOIN articles a ON at.article_id = a.id AND a.draft = 0
                  GROUP BY t.id, t.name
                  ORDER BY article_count DESC, t.name",
                (),
            )
            .await?;

        let mut tags = Vec::new();
        while let Some(row) = rows.next().await? {
            let name: String = row.get(0)?;
            let count: i32 = row.get(1).unwrap_or(0);
            tags.push((name, count));
        }

        Ok(tags)
    }

    async fn rename(&self, old_name: &str, new_name: &str) -> RepoResult<TagRecord> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // Check if old tag exists
        let existing = self.find_by_name(old_name).await?;
        if existing.is_none() {
            return Err(RepositoryError::NotFound(format!("Tag '{}'", old_name)));
        }

        // Check if new name already exists
        if let Some(_) = self.find_by_name(new_name).await? {
            return Err(RepositoryError::AlreadyExists(format!("Tag '{}'", new_name)));
        }

        conn.execute(
            "UPDATE tags SET name = ? WHERE name = ?",
            libsql::params![new_name, old_name],
        )
        .await?;

        self.find_by_name(new_name)
            .await?
            .ok_or_else(|| RepositoryError::InternalError("Failed to rename tag".to_string()))
    }

    async fn delete(&self, name: &str) -> RepoResult<()> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // Get tag ID first
        let tag = self.find_by_name(name).await?;
        let tag = tag.ok_or_else(|| RepositoryError::NotFound(format!("Tag '{}'", name)))?;

        // Delete article_tags associations
        conn.execute(
            "DELETE FROM article_tags WHERE tag_id = ?",
            libsql::params![tag.id.to_string()],
        )
        .await?;

        // Delete the tag
        conn.execute(
            "DELETE FROM tags WHERE id = ?",
            libsql::params![tag.id.to_string()],
        )
        .await?;

        Ok(())
    }

    async fn delete_orphaned(&self) -> RepoResult<i32> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let result = conn
            .execute(
                r"DELETE FROM tags
                  WHERE id NOT IN (SELECT DISTINCT tag_id FROM article_tags)",
                (),
            )
            .await?;

        Ok(result as i32)
    }
}
