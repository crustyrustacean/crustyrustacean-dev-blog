// src/lib/repositories/comment_libsql.rs

//! LibSQL implementation of the CommentRepository trait.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::database::DatabaseConnection;
use crate::models::UserProfile;

use super::error::{RepoResult, RepositoryError};
use super::comment::{CommentRecord, CommentRepository, CommentWithAuthor, NewComment};

/// LibSQL implementation of the CommentRepository.
#[derive(Debug, Clone)]
pub struct LibSqlCommentRepository {
    db: DatabaseConnection,
}

impl LibSqlCommentRepository {
    /// Create a new LibSQL comment repository.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Helper to parse a datetime string.
    fn parse_datetime(s: &str) -> RepoResult<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| ndt.and_utc())
            })
            .map_err(|e| RepositoryError::InternalError(format!("Failed to parse datetime: {}", e)))
    }
}

#[async_trait]
impl CommentRepository for LibSqlCommentRepository {
    async fn create(&self, comment: &NewComment) -> RepoResult<CommentRecord> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let id = Uuid::new_v4();
        let now = Utc::now();

        conn.execute(
            r"INSERT INTO comments (id, body, author_id, article_id, created_at, updated_at)
              VALUES (?, ?, ?, ?, ?, ?)",
            libsql::params![
                id.to_string(),
                comment.body.clone(),
                comment.author_id.to_string(),
                comment.article_id.to_string(),
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )
        .await?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| RepositoryError::InternalError("Failed to create comment".to_string()))
    }

    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<CommentRecord>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                "SELECT id, body, author_id, article_id, created_at, updated_at FROM comments WHERE id = ?",
                libsql::params![id.to_string()],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid UUID: {}", e)))?;
            let body: String = row.get(1)?;
            let author_id_str: String = row.get(2)?;
            let author_id = Uuid::parse_str(&author_id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid author UUID: {}", e)))?;
            let article_id_str: String = row.get(3)?;
            let article_id = Uuid::parse_str(&article_id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid article UUID: {}", e)))?;
            let created_at_str: String = row.get(4)?;
            let updated_at_str: String = row.get(5)?;

            Ok(Some(CommentRecord {
                id,
                body,
                author_id,
                article_id,
                created_at: Self::parse_datetime(&created_at_str)?,
                updated_at: Self::parse_datetime(&updated_at_str)?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_with_author(
        &self,
        id: Uuid,
        current_user_id: Option<Uuid>,
    ) -> RepoResult<Option<CommentWithAuthor>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                r"SELECT c.id, c.body, c.author_id, c.article_id, c.created_at, c.updated_at,
                         u.username, u.bio, u.image
                  FROM comments c
                  JOIN users u ON c.author_id = u.id
                  WHERE c.id = ?",
                libsql::params![id.to_string()],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid UUID: {}", e)))?;
            let body: String = row.get(1)?;
            let author_id_str: String = row.get(2)?;
            let author_id = Uuid::parse_str(&author_id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid author UUID: {}", e)))?;
            let article_id_str: String = row.get(3)?;
            let article_id = Uuid::parse_str(&article_id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid article UUID: {}", e)))?;
            let created_at_str: String = row.get(4)?;
            let updated_at_str: String = row.get(5)?;
            let username: String = row.get(6)?;
            let bio: Option<String> = row.get(7).ok();
            let image: Option<String> = row.get(8).ok();

            // Check following status
            let following = if let Some(user_id) = current_user_id {
                let mut follow_rows = conn
                    .query(
                        "SELECT 1 FROM user_follows WHERE follower_id = ? AND following_id = ?",
                        libsql::params![user_id.to_string(), author_id.to_string()],
                    )
                    .await?;
                follow_rows.next().await?.is_some()
            } else {
                false
            };

            Ok(Some(CommentWithAuthor {
                comment: CommentRecord {
                    id,
                    body,
                    author_id,
                    article_id,
                    created_at: Self::parse_datetime(&created_at_str)?,
                    updated_at: Self::parse_datetime(&updated_at_str)?,
                },
                author: UserProfile {
                    username,
                    bio,
                    image,
                    following,
                },
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_for_article(
        &self,
        article_id: Uuid,
        current_user_id: Option<Uuid>,
    ) -> RepoResult<Vec<CommentWithAuthor>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                r"SELECT c.id, c.body, c.author_id, c.article_id, c.created_at, c.updated_at,
                         u.username, u.bio, u.image
                  FROM comments c
                  JOIN users u ON c.author_id = u.id
                  WHERE c.article_id = ?
                  ORDER BY c.created_at ASC",
                libsql::params![article_id.to_string()],
            )
            .await?;

        let mut comments = Vec::new();
        while let Some(row) = rows.next().await? {
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid UUID: {}", e)))?;
            let body: String = row.get(1)?;
            let author_id_str: String = row.get(2)?;
            let author_id = Uuid::parse_str(&author_id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid author UUID: {}", e)))?;
            let article_id_str: String = row.get(3)?;
            let article_id = Uuid::parse_str(&article_id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid article UUID: {}", e)))?;
            let created_at_str: String = row.get(4)?;
            let updated_at_str: String = row.get(5)?;
            let username: String = row.get(6)?;
            let bio: Option<String> = row.get(7).ok();
            let image: Option<String> = row.get(8).ok();

            // Check following status
            let following = if let Some(user_id) = current_user_id {
                let mut follow_rows = conn
                    .query(
                        "SELECT 1 FROM user_follows WHERE follower_id = ? AND following_id = ?",
                        libsql::params![user_id.to_string(), author_id.to_string()],
                    )
                    .await?;
                follow_rows.next().await?.is_some()
            } else {
                false
            };

            comments.push(CommentWithAuthor {
                comment: CommentRecord {
                    id,
                    body,
                    author_id,
                    article_id,
                    created_at: Self::parse_datetime(&created_at_str)?,
                    updated_at: Self::parse_datetime(&updated_at_str)?,
                },
                author: UserProfile {
                    username,
                    bio,
                    image,
                    following,
                },
            });
        }

        Ok(comments)
    }

    async fn count_for_article(&self, article_id: Uuid) -> RepoResult<i32> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                "SELECT COUNT(*) FROM comments WHERE article_id = ?",
                libsql::params![article_id.to_string()],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(row.get(0).unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    async fn delete(&self, id: Uuid) -> RepoResult<()> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let result = conn
            .execute(
                "DELETE FROM comments WHERE id = ?",
                libsql::params![id.to_string()],
            )
            .await?;

        if result == 0 {
            return Err(RepositoryError::NotFound(format!("Comment with id {}", id)));
        }

        Ok(())
    }

    async fn delete_for_article(&self, article_id: Uuid) -> RepoResult<i32> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let result = conn
            .execute(
                "DELETE FROM comments WHERE article_id = ?",
                libsql::params![article_id.to_string()],
            )
            .await?;

        Ok(result as i32)
    }
}
