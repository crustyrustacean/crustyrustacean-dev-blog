// src/lib/repositories/tag_pg.rs

//! PostgreSQL implementation of the TagRepository trait.

use async_trait::async_trait;
use uuid::Uuid;

use super::error::{RepoResult, RepositoryError};
use super::tag::{TagRecord, TagRepository};

/// PostgreSQL implementation of the TagRepository.
#[derive(Debug, Clone)]
pub struct PgTagRepository {
    db: sqlx::PgPool,
}

impl PgTagRepository {
    /// Create a new PostgreSQL tag repository.
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl TagRepository for PgTagRepository {
    async fn create_or_get(&self, name: &str) -> RepoResult<TagRecord> {
        // INSERT ... ON CONFLICT ensures the tag exists; then select it back.
        let tag = sqlx::query_as::<_, (Uuid, String)>(
            "INSERT INTO tags (name) VALUES ($1) \
             ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name \
             RETURNING id, name",
        )
        .bind(name)
        .fetch_one(&self.db)
        .await
        .map(|(id, name)| TagRecord { id, name })?;

        Ok(tag)
    }

    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<TagRecord>> {
        let tag = sqlx::query_as::<_, (Uuid, String)>("SELECT id, name FROM tags WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.db)
            .await?
            .map(|(id, name)| TagRecord { id, name });

        Ok(tag)
    }

    async fn find_by_name(&self, name: &str) -> RepoResult<Option<TagRecord>> {
        let tag =
            sqlx::query_as::<_, (Uuid, String)>("SELECT id, name FROM tags WHERE name = $1")
                .bind(name)
                .fetch_optional(&self.db)
                .await?
                .map(|(id, name)| TagRecord { id, name });

        Ok(tag)
    }

    async fn list_all(&self) -> RepoResult<Vec<String>> {
        let tags: Vec<String> = sqlx::query_scalar("SELECT name FROM tags ORDER BY name")
            .fetch_all(&self.db)
            .await?;

        Ok(tags)
    }

    async fn list_with_counts(&self) -> RepoResult<Vec<(String, i32)>> {
        let rows = sqlx::query_as::<_, (String, i32)>(
            r"SELECT t.name, COUNT(at.article_id)::INT as article_count
              FROM tags t
              LEFT JOIN article_tags at ON t.id = at.tag_id
              GROUP BY t.id, t.name
              ORDER BY article_count DESC, t.name",
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows)
    }

    async fn rename(&self, old_name: &str, new_name: &str) -> RepoResult<TagRecord> {
        // Check the target name is not already taken.
        if self.find_by_name(new_name).await?.is_some() {
            return Err(RepositoryError::AlreadyExists(format!("Tag '{}'", new_name)));
        }

        let result = sqlx::query_as::<_, (Uuid, String)>(
            "UPDATE tags SET name = $2 WHERE name = $1 RETURNING id, name",
        )
        .bind(old_name)
        .bind(new_name)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| {
            if matches!(e, sqlx::Error::RowNotFound) {
                RepositoryError::NotFound(format!("Tag '{}'", old_name))
            } else {
                e.into()
            }
        })?;

        match result {
            Some((id, name)) => Ok(TagRecord { id, name }),
            None => Err(RepositoryError::NotFound(format!("Tag '{}'", old_name))),
        }
    }

    async fn delete(&self, name: &str) -> RepoResult<()> {
        // Delete links first, then the tag itself.
        let tag = self
            .find_by_name(name)
            .await?
            .ok_or_else(|| RepositoryError::NotFound(format!("Tag '{}'", name)))?;

        sqlx::query("DELETE FROM article_tags WHERE tag_id = $1")
            .bind(tag.id)
            .execute(&self.db)
            .await?;

        sqlx::query("DELETE FROM tags WHERE id = $1")
            .bind(tag.id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    async fn delete_orphaned(&self) -> RepoResult<i32> {
        let result = sqlx::query(
            r"DELETE FROM tags
              WHERE id NOT IN (SELECT DISTINCT tag_id FROM article_tags)",
        )
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected() as i32)
    }
}
