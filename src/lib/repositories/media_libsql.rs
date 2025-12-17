// src/lib/repositories/media_libsql.rs

//! LibSQL implementation of the MediaRepository trait.

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::database::DatabaseConnection;
use crate::models::{Media, MediaQuery};

use super::error::{RepoResult, RepositoryError};
use super::media::{MediaRepository, NewMedia, UpdateMediaData};

/// LibSQL implementation of the MediaRepository.
#[derive(Debug, Clone)]
pub struct LibSqlMediaRepository {
    db: DatabaseConnection,
}

impl LibSqlMediaRepository {
    /// Create a new LibSQL media repository.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Convert a database row to a Media struct.
    fn row_to_media(&self, row: &libsql::Row) -> RepoResult<Media> {
        let id: String = row.get(0)?;
        let user_id: String = row.get(1)?;
        let filename: String = row.get(2)?;
        let storage_path: String = row.get(3)?;
        let title: Option<String> = row.get(4).ok();
        let alt_text: Option<String> = row.get(5).ok();
        let caption: Option<String> = row.get(6).ok();
        let description: Option<String> = row.get(7).ok();
        let mime_type: String = row.get(8)?;
        let file_size: i64 = row.get(9)?;
        let width: Option<i64> = row.get(10).ok();
        let height: Option<i64> = row.get(11).ok();
        let uploaded_at: String = row.get(12)?;
        let updated_at: String = row.get(13)?;

        Ok(Media {
            id,
            user_id,
            filename,
            storage_path,
            title,
            alt_text,
            caption,
            description,
            mime_type,
            file_size,
            width,
            height,
            uploaded_at,
            updated_at,
        })
    }
}

#[async_trait]
impl MediaRepository for LibSqlMediaRepository {
    async fn create(&self, media: &NewMedia) -> RepoResult<Media> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let title_param = match &media.title {
            Some(t) => libsql::Value::Text(t.clone()),
            None => libsql::Value::Null,
        };

        let alt_text_param = match &media.alt_text {
            Some(a) => libsql::Value::Text(a.clone()),
            None => libsql::Value::Null,
        };

        let width_param = match media.width {
            Some(w) => libsql::Value::Integer(w),
            None => libsql::Value::Null,
        };

        let height_param = match media.height {
            Some(h) => libsql::Value::Integer(h),
            None => libsql::Value::Null,
        };

        conn.execute(
            r"INSERT INTO media (id, user_id, filename, storage_path, title, alt_text, mime_type, file_size, width, height, uploaded_at, updated_at)
              VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            vec![
                libsql::Value::Text(id.clone()),
                libsql::Value::Text(media.user_id.clone()),
                libsql::Value::Text(media.filename.clone()),
                libsql::Value::Text(media.storage_path.clone()),
                title_param,
                alt_text_param,
                libsql::Value::Text(media.mime_type.clone()),
                libsql::Value::Integer(media.file_size),
                width_param,
                height_param,
                libsql::Value::Text(now.to_rfc3339()),
                libsql::Value::Text(now.to_rfc3339()),
            ],
        )
        .await?;

        self.find_by_id(&id)
            .await?
            .ok_or_else(|| RepositoryError::InternalError("Failed to create media".to_string()))
    }

    async fn find_by_id(&self, id: &str) -> RepoResult<Option<Media>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                r"SELECT id, user_id, filename, storage_path, title, alt_text, caption, description, mime_type, file_size, width, height, uploaded_at, updated_at
                  FROM media WHERE id = ?",
                libsql::params![id],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(self.row_to_media(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn list_for_user(&self, user_id: &str, query: &MediaQuery) -> RepoResult<Vec<Media>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut sql = String::from(
            r"SELECT id, user_id, filename, storage_path, title, alt_text, caption, description, mime_type, file_size, width, height, uploaded_at, updated_at
              FROM media WHERE user_id = ?",
        );
        let mut params: Vec<libsql::Value> = vec![libsql::Value::Text(user_id.to_string())];

        // Filter by mime_type prefix if specified
        if let Some(ref mime_type) = query.mime_type {
            sql.push_str(" AND mime_type LIKE ?");
            params.push(libsql::Value::Text(format!("{}%", mime_type)));
        }

        sql.push_str(" ORDER BY uploaded_at DESC");

        // Pagination
        let limit = query.limit.unwrap_or(20);
        let offset = query.offset.unwrap_or(0);
        sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        let mut rows = conn.query(&sql, libsql::params_from_iter(params)).await?;

        let mut media_list = Vec::new();
        while let Some(row) = rows.next().await? {
            media_list.push(self.row_to_media(&row)?);
        }

        Ok(media_list)
    }

    async fn count_for_user(&self, user_id: &str, query: &MediaQuery) -> RepoResult<i64> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut sql = String::from("SELECT COUNT(*) FROM media WHERE user_id = ?");
        let mut params: Vec<libsql::Value> = vec![libsql::Value::Text(user_id.to_string())];

        if let Some(ref mime_type) = query.mime_type {
            sql.push_str(" AND mime_type LIKE ?");
            params.push(libsql::Value::Text(format!("{}%", mime_type)));
        }

        let mut rows = conn.query(&sql, libsql::params_from_iter(params)).await?;

        if let Some(row) = rows.next().await? {
            let count: i64 = row.get(0)?;
            Ok(count)
        } else {
            Ok(0)
        }
    }

    async fn update(&self, id: &str, data: &UpdateMediaData) -> RepoResult<Media> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // Verify the media exists
        if self.find_by_id(id).await?.is_none() {
            return Err(RepositoryError::NotFound(format!("Media with id '{}'", id)));
        }

        let mut updates = Vec::new();
        let mut params: Vec<libsql::Value> = Vec::new();

        if let Some(title) = &data.title {
            updates.push("title = ?");
            params.push(libsql::Value::Text(title.clone()));
        }
        if let Some(alt_text) = &data.alt_text {
            updates.push("alt_text = ?");
            params.push(libsql::Value::Text(alt_text.clone()));
        }
        if let Some(caption) = &data.caption {
            updates.push("caption = ?");
            params.push(libsql::Value::Text(caption.clone()));
        }
        if let Some(description) = &data.description {
            updates.push("description = ?");
            params.push(libsql::Value::Text(description.clone()));
        }

        if updates.is_empty() {
            return self.find_by_id(id).await?.ok_or_else(|| {
                RepositoryError::InternalError("Media disappeared during update".to_string())
            });
        }

        updates.push("updated_at = ?");
        params.push(libsql::Value::Text(Utc::now().to_rfc3339()));

        params.push(libsql::Value::Text(id.to_string()));

        let update_query = format!("UPDATE media SET {} WHERE id = ?", updates.join(", "));

        conn.execute(&update_query, libsql::params_from_iter(params))
            .await?;

        self.find_by_id(id).await?.ok_or_else(|| {
            RepositoryError::InternalError("Failed to retrieve updated media".to_string())
        })
    }

    async fn delete(&self, id: &str) -> RepoResult<()> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // First remove any usage tracking
        conn.execute(
            "DELETE FROM media_usage WHERE media_id = ?",
            libsql::params![id],
        )
        .await?;

        let result = conn
            .execute("DELETE FROM media WHERE id = ?", libsql::params![id])
            .await?;

        if result == 0 {
            return Err(RepositoryError::NotFound(format!("Media with id '{}'", id)));
        }

        Ok(())
    }

    async fn track_usage(
        &self,
        media_id: &str,
        article_slug: &str,
        context: Option<&str>,
    ) -> RepoResult<()> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let context_param = match context {
            Some(c) => libsql::Value::Text(c.to_string()),
            None => libsql::Value::Null,
        };

        // Use INSERT OR REPLACE to handle duplicates
        conn.execute(
            r"INSERT OR REPLACE INTO media_usage (id, media_id, article_slug, context, created_at)
              VALUES (?, ?, ?, ?, ?)",
            vec![
                libsql::Value::Text(id),
                libsql::Value::Text(media_id.to_string()),
                libsql::Value::Text(article_slug.to_string()),
                context_param,
                libsql::Value::Text(now.to_rfc3339()),
            ],
        )
        .await?;

        Ok(())
    }

    async fn remove_usage(&self, article_slug: &str) -> RepoResult<()> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        conn.execute(
            "DELETE FROM media_usage WHERE article_slug = ?",
            libsql::params![article_slug],
        )
        .await?;

        Ok(())
    }
}
