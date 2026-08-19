// src/lib/repositories/comment_pg.rs

//! PostgreSQL implementation of the CommentRepository trait.

use async_trait::async_trait;
use uuid::Uuid;

use crate::models::UserProfile;

use super::comment::{CommentRecord, CommentRepository, CommentWithAuthor, NewComment};
use super::error::{RepoResult, RepositoryError};

/// PostgreSQL implementation of the CommentRepository.
#[derive(Debug, Clone)]
pub struct PgCommentRepository {
    db: sqlx::PgPool,
}

impl PgCommentRepository {
    /// Create a new PostgreSQL comment repository.
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl CommentRepository for PgCommentRepository {
    async fn create(&self, comment: &NewComment) -> RepoResult<CommentRecord> {
        let record = sqlx::query_as::<_, (Uuid, String, Uuid, Uuid, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            r"INSERT INTO comments (body, author_id, article_id)
              VALUES ($1, $2, $3)
              RETURNING id, body, author_id, article_id, created_at, updated_at",
        )
        .bind(&comment.body)
        .bind(comment.author_id)
        .bind(comment.article_id)
        .fetch_one(&self.db)
        .await
        .map(|(id, body, author_id, article_id, created_at, updated_at)| CommentRecord {
            id,
            body,
            author_id,
            article_id,
            created_at,
            updated_at,
        })?;

        Ok(record)
    }

    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<CommentRecord>> {
        let record = sqlx::query_as::<_, (Uuid, String, Uuid, Uuid, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, body, author_id, article_id, created_at, updated_at FROM comments WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .map(|(id, body, author_id, article_id, created_at, updated_at)| CommentRecord {
            id,
            body,
            author_id,
            article_id,
            created_at,
            updated_at,
        });

        Ok(record)
    }

    async fn get_with_author(
        &self,
        id: Uuid,
        current_user_id: Option<Uuid>,
    ) -> RepoResult<Option<CommentWithAuthor>> {
        let viewer = current_user_id.unwrap_or(Uuid::nil());
        let row = sqlx::query_as::<_, (Uuid, String, Uuid, Uuid, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>, String, Option<String>, Option<String>, Option<i32>)>(
            r"SELECT c.id, c.body, c.author_id, c.article_id, c.created_at, c.updated_at,
                     u.username, u.bio, u.image,
                     (SELECT 1 FROM user_follows
                      WHERE follower_id = $1 AND following_id = c.author_id) as is_following
              FROM comments c
              JOIN users u ON c.author_id = u.id
              WHERE c.id = $2",
        )
        .bind(viewer)
        .bind(id)
        .fetch_optional(&self.db)
        .await?;

        let Some((cid, body, author_id, article_id, created_at, updated_at, username, bio, image, following)) = row else {
            return Ok(None);
        };

        let following = current_user_id.is_some() && following.unwrap_or(0) == 1;

        Ok(Some(CommentWithAuthor {
            comment: CommentRecord {
                id: cid,
                body,
                author_id,
                article_id,
                created_at,
                updated_at,
            },
            author: UserProfile {
                username,
                bio,
                image,
                following,
            },
        }))
    }

    async fn list_for_article(
        &self,
        article_id: Uuid,
        current_user_id: Option<Uuid>,
    ) -> RepoResult<Vec<CommentWithAuthor>> {
        let viewer = current_user_id.unwrap_or(Uuid::nil());
        let rows = sqlx::query_as::<_, (Uuid, String, Uuid, Uuid, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>, String, Option<String>, Option<String>, Option<i32>)>(
            r"SELECT c.id, c.body, c.author_id, c.article_id, c.created_at, c.updated_at,
                     u.username, u.bio, u.image,
                     (SELECT 1 FROM user_follows
                      WHERE follower_id = $1 AND following_id = c.author_id) as is_following
              FROM comments c
              JOIN users u ON c.author_id = u.id
              WHERE c.article_id = $2
              ORDER BY c.created_at ASC",
        )
        .bind(viewer)
        .bind(article_id)
        .fetch_all(&self.db)
        .await?;

        let mut comments = Vec::with_capacity(rows.len());
        for (id, body, author_id, aid, created_at, updated_at, username, bio, image, following) in
            rows
        {
            let following = current_user_id.is_some() && following.unwrap_or(0) == 1;
            comments.push(CommentWithAuthor {
                comment: CommentRecord {
                    id,
                    body,
                    author_id,
                    article_id: aid,
                    created_at,
                    updated_at,
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
        let count: i32 = sqlx::query_scalar(
            "SELECT COUNT(*)::INT FROM comments WHERE article_id = $1",
        )
        .bind(article_id)
        .fetch_one(&self.db)
        .await?;

        Ok(count)
    }

    async fn delete(&self, id: Uuid) -> RepoResult<()> {
        let result = sqlx::query("DELETE FROM comments WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!("Comment with id {}", id)));
        }

        Ok(())
    }

    async fn delete_for_article(&self, article_id: Uuid) -> RepoResult<i32> {
        let result = sqlx::query("DELETE FROM comments WHERE article_id = $1")
            .bind(article_id)
            .execute(&self.db)
            .await?;

        Ok(result.rows_affected() as i32)
    }
}
