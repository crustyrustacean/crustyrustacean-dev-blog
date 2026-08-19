// src/lib/repositories/media_pg.rs

//! PostgreSQL implementation of the MediaRepository trait.

use async_trait::async_trait;
use uuid::Uuid;

use crate::models::{Media, MediaQuery};

use super::error::{RepoResult, RepositoryError};
use super::media::{MediaRepository, NewMedia, UpdateMediaData};

/// PostgreSQL implementation of the MediaRepository.
#[derive(Debug, Clone)]
pub struct PgMediaRepository {
    db: sqlx::PgPool,
}

// Row shape: everything needed to build a `Media` model. Timestamps are
// read as RFC3339 strings to match the model's String fields.
type MediaRow = (
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    String,
    i64,
    Option<i64>,
    Option<i64>,
    String,
    String,
);

const MEDIA_COLUMNS: &str = "id::TEXT, user_id::TEXT, filename, storage_path, title, alt_text, \
                            caption, description, mime_type, file_size, width, height, \
                            to_char(uploaded_at, 'YYYY-MM-DD\"T\"HH24:MI:SS.USZ'), \
                            to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS.USZ')";

fn media_from_row(row: MediaRow) -> Media {
    Media {
        id: row.0,
        user_id: row.1,
        filename: row.2,
        storage_path: row.3,
        title: row.4,
        alt_text: row.5,
        caption: row.6,
        description: row.7,
        mime_type: row.8,
        file_size: row.9,
        width: row.10,
        height: row.11,
        uploaded_at: row.12,
        updated_at: row.13,
    }
}

impl PgMediaRepository {
    /// Create a new PostgreSQL media repository.
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl MediaRepository for PgMediaRepository {
    async fn create(&self, media: &NewMedia) -> RepoResult<Media> {
        let row = sqlx::query_as::<_, MediaRow>(&format!(
            r"INSERT INTO media_library
                   (user_id, filename, storage_path, title, alt_text, caption, description,
                    mime_type, file_size, width, height)
              VALUES ($1::UUID, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
              RETURNING {MEDIA_COLUMNS}",
            MEDIA_COLUMNS = MEDIA_COLUMNS
        ))
        .bind(Uuid::parse_str(&media.user_id).map_err(|e| {
            RepositoryError::ValidationError(format!("Invalid user UUID: {}", e))
        })?)
        .bind(&media.filename)
        .bind(&media.storage_path)
        .bind(&media.title)
        .bind(&media.alt_text)
        .bind(&media.caption)
        .bind(&media.description)
        .bind(&media.mime_type)
        .bind(media.file_size)
        .bind(media.width)
        .bind(media.height)
        .fetch_one(&self.db)
        .await?;

        Ok(media_from_row(row))
    }

    async fn find_by_id(&self, id: &str) -> RepoResult<Option<Media>> {
        let uuid = Uuid::parse_str(id).map_err(|e| {
            RepositoryError::ValidationError(format!("Invalid media UUID: {}", e))
        })?;

        let row = sqlx::query_as::<_, MediaRow>(&format!(
            "SELECT {MEDIA_COLUMNS} FROM media_library WHERE id = $1",
            MEDIA_COLUMNS = MEDIA_COLUMNS
        ))
        .bind(uuid)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(media_from_row))
    }

    async fn list_for_user(
        &self,
        user_id: &str,
        query: &MediaQuery,
    ) -> RepoResult<Vec<Media>> {
        let uuid = Uuid::parse_str(user_id).map_err(|e| {
            RepositoryError::ValidationError(format!("Invalid user UUID: {}", e))
        })?;

        let limit = query.limit.unwrap_or(20);
        let offset = query.offset.unwrap_or(0);

        let sql = if let Some(_mime_type) = &query.mime_type {
            format!(
                "SELECT {MEDIA_COLUMNS} FROM media_library \
                 WHERE user_id = $1 AND mime_type LIKE $2 \
                 ORDER BY uploaded_at DESC LIMIT $3 OFFSET $4",
                MEDIA_COLUMNS = MEDIA_COLUMNS
            )
        } else {
            format!(
                "SELECT {MEDIA_COLUMNS} FROM media_library \
                 WHERE user_id = $1 ORDER BY uploaded_at DESC LIMIT $2 OFFSET $3",
                MEDIA_COLUMNS = MEDIA_COLUMNS
            )
        };

        let rows = if let Some(mime_type) = &query.mime_type {
            sqlx::query_as::<_, MediaRow>(&sql)
                .bind(uuid)
                .bind(format!("{}%", mime_type))
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.db)
                .await?
        } else {
            sqlx::query_as::<_, MediaRow>(&sql)
                .bind(uuid)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.db)
                .await?
        };

        Ok(rows.into_iter().map(media_from_row).collect())
    }

    async fn count_for_user(&self, user_id: &str, query: &MediaQuery) -> RepoResult<i64> {
        let uuid = Uuid::parse_str(user_id).map_err(|e| {
            RepositoryError::ValidationError(format!("Invalid user UUID: {}", e))
        })?;

        let count: i64 = if let Some(mime_type) = &query.mime_type {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM media_library \
                 WHERE user_id = $1 AND mime_type LIKE $2",
            )
            .bind(uuid)
            .bind(format!("{}%", mime_type))
            .fetch_one(&self.db)
            .await?
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM media_library WHERE user_id = $1")
                .bind(uuid)
                .fetch_one(&self.db)
                .await?
        };

        Ok(count)
    }

    async fn update(&self, id: &str, data: &UpdateMediaData) -> RepoResult<Media> {
        let uuid = Uuid::parse_str(id).map_err(|e| {
            RepositoryError::ValidationError(format!("Invalid media UUID: {}", e))
        })?;

        let mut updates = Vec::new();
        let mut binds: Vec<Option<String>> = Vec::new();

        if let Some(title) = &data.title {
            updates.push(format!("title = ${}", binds.len() + 1));
            binds.push(Some(title.clone()));
        }
        if let Some(alt_text) = &data.alt_text {
            updates.push(format!("alt_text = ${}", binds.len() + 1));
            binds.push(Some(alt_text.clone()));
        }
        if let Some(caption) = &data.caption {
            updates.push(format!("caption = ${}", binds.len() + 1));
            binds.push(Some(caption.clone()));
        }
        if let Some(description) = &data.description {
            updates.push(format!("description = ${}", binds.len() + 1));
            binds.push(Some(description.clone()));
        }

        if updates.is_empty() {
            return self
                .find_by_id(id)
                .await?
                .ok_or_else(|| RepositoryError::NotFound(format!("Media with id '{}'", id)));
        }

        updates.push("updated_at = NOW()".to_string());
        let id_bind = binds.len() + 1;

        let sql = format!(
            "UPDATE media_library SET {} WHERE id = ${} RETURNING {MEDIA_COLUMNS}",
            updates.join(", "),
            id_bind,
            MEDIA_COLUMNS = MEDIA_COLUMNS
        );

        let mut query = sqlx::query_as::<_, MediaRow>(&sql);
        for value in &binds {
            query = query.bind(value);
        }
        query = query.bind(uuid);

        let row = query
            .fetch_optional(&self.db)
            .await?
            .ok_or_else(|| RepositoryError::NotFound(format!("Media with id '{}'", id)))?;

        Ok(media_from_row(row))
    }

    async fn delete(&self, id: &str) -> RepoResult<()> {
        let uuid = Uuid::parse_str(id).map_err(|e| {
            RepositoryError::ValidationError(format!("Invalid media UUID: {}", e))
        })?;

        let result = sqlx::query("DELETE FROM media_library WHERE id = $1")
            .bind(uuid)
            .execute(&self.db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!(
                "Media with id '{}'",
                id
            )));
        }

        Ok(())
    }

    async fn track_usage(
        &self,
        media_id: &str,
        article_slug: &str,
        context: Option<&str>,
    ) -> RepoResult<()> {
        let uuid = Uuid::parse_str(media_id).map_err(|e| {
            RepositoryError::ValidationError(format!("Invalid media UUID: {}", e))
        })?;

        // Upsert: the same media can be re-tracked when an article is saved again.
        sqlx::query(
            r"INSERT INTO media_usage (media_id, article_slug, usage_context)
              VALUES ($1, $2, $3)
              ON CONFLICT (media_id, article_slug) DO UPDATE SET usage_context = EXCLUDED.usage_context",
        )
        .bind(uuid)
        .bind(article_slug)
        .bind(context)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn remove_usage(&self, article_slug: &str) -> RepoResult<()> {
        sqlx::query("DELETE FROM media_usage WHERE article_slug = $1")
            .bind(article_slug)
            .execute(&self.db)
            .await?;

        Ok(())
    }
}
