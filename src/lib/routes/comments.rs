// src/lib/routes/comments.rs

use crate::{
    ApiError, AppState,
    auth::AuthenticatedUser,
    models::{
        CommentResponse, CreateComment, MultipleCommentsResponse, SingleCommentResponse,
        UserProfile,
    },
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use uuid::Uuid;
use validator::Validate;

pub async fn add_comment(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    user: AuthenticatedUser,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<SingleCommentResponse>, ApiError> {
    let comment_data: CreateComment = serde_json::from_value(
        payload
            .get("comment")
            .ok_or_else(|| ApiError::BadRequest("Missing comment field".to_string()))?
            .clone(),
    )
    .map_err(|_| ApiError::BadRequest("Invalid comment data".to_string()))?;

    comment_data
        .validate()
        .map_err(|e| ApiError::BadRequest(format!("Validation error: {}", e)))?;

    let conn = state
        .db
        .connect()
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    // Check if article exists
    let mut article_rows = conn
        .query(
            "SELECT id FROM articles WHERE slug = ?",
            libsql::params![slug.clone()],
        )
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let article_row = article_rows
        .next()
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Article not found".to_string()))?;

    let article_id: String = article_row
        .get(0)
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    // Create comment
    let comment_id = Uuid::new_v4();
    let now = Utc::now();

    conn.execute(
        "INSERT INTO comments (id, body, author_id, article_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        libsql::params![
            comment_id.to_string(),
            comment_data.body.clone(),
            user.user_id.to_string(),
            article_id,
            now.to_rfc3339(),
            now.to_rfc3339(),
        ],
    )
    .await
    .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    // Get author profile
    let mut author_rows = conn
        .query(
            "SELECT username, bio, image FROM users WHERE id = ?",
            libsql::params![user.user_id.to_string()],
        )
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let author_row = author_rows
        .next()
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .ok_or_else(|| ApiError::InternalServerError("Author not found".to_string()))?;

    let username: String = author_row
        .get(0)
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    let bio: Option<String> = author_row.get(1).ok();
    let image: Option<String> = author_row.get(2).ok();

    let author = UserProfile {
        username,
        bio,
        image,
        following: false,
    };

    let comment_response = CommentResponse {
        id: comment_id,
        created_at: now,
        updated_at: now,
        body: comment_data.body,
        author,
    };

    let response = SingleCommentResponse {
        comment: comment_response,
    };

    Ok(Json(response))
}

pub async fn get_comments(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<MultipleCommentsResponse>, ApiError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    // Check if article exists
    let mut article_rows = conn
        .query(
            "SELECT id FROM articles WHERE slug = ?",
            libsql::params![slug.clone()],
        )
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let article_row = article_rows
        .next()
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Article not found".to_string()))?;

    let article_id: String = article_row
        .get(0)
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    // Get all comments for the article
    let mut comment_rows = conn
        .query(
            "SELECT c.id, c.body, c.created_at, c.updated_at, c.author_id, u.username, u.bio, u.image
             FROM comments c
             JOIN users u ON c.author_id = u.id
             WHERE c.article_id = ?
             ORDER BY c.created_at DESC",
            libsql::params![article_id],
        )
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let mut comments = Vec::new();

    while let Some(row) = comment_rows
        .next()
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
    {
        let comment_id_str: String = row
            .get(0)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        let comment_id = Uuid::parse_str(&comment_id_str)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

        let body: String = row
            .get(1)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

        let created_at_str: String = row
            .get(2)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?
            .with_timezone(&Utc);

        let updated_at_str: String = row
            .get(3)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?
            .with_timezone(&Utc);

        let username: String = row
            .get(5)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        let bio: Option<String> = row.get(6).ok();
        let image: Option<String> = row.get(7).ok();

        let author = UserProfile {
            username,
            bio,
            image,
            following: false,
        };

        let comment_response = CommentResponse {
            id: comment_id,
            created_at,
            updated_at,
            body,
            author,
        };

        comments.push(comment_response);
    }

    let response = MultipleCommentsResponse { comments };

    Ok(Json(response))
}

pub async fn delete_comment(
    State(state): State<AppState>,
    Path((slug, comment_id)): Path<(String, String)>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    // Check if article exists
    let mut article_rows = conn
        .query(
            "SELECT id FROM articles WHERE slug = ?",
            libsql::params![slug.clone()],
        )
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let _article_row = article_rows
        .next()
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Article not found".to_string()))?;

    // Check if comment exists and verify ownership
    let mut comment_rows = conn
        .query(
            "SELECT author_id FROM comments WHERE id = ?",
            libsql::params![comment_id.clone()],
        )
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let comment_row = comment_rows
        .next()
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Comment not found".to_string()))?;

    let author_id: String = comment_row
        .get(0)
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    // Verify the user is the comment author
    if author_id != user.user_id.to_string() {
        return Err(ApiError::Forbidden(
            "You can only delete your own comments".to_string(),
        ));
    }

    // Delete the comment
    conn.execute(
        "DELETE FROM comments WHERE id = ?",
        libsql::params![comment_id],
    )
    .await
    .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
