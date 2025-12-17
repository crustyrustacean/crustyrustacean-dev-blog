// src/lib/routes/comments.rs

use crate::{
    ApiError, AppState,
    auth::AuthenticatedUser,
    models::{
        CommentResponse, CreateComment, MultipleCommentsResponse, SingleCommentResponse,
        UserProfile,
    },
    repositories::NewComment,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
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

    // Check if article exists
    let article = state
        .articles
        .find_by_slug(&slug)
        .await?
        .ok_or_else(|| ApiError::NotFound("Article not found".to_string()))?;

    // Create comment
    let new_comment = NewComment {
        body: comment_data.body.clone(),
        author_id: user.user_id,
        article_id: article.id,
    };

    let comment = state.comments.create(&new_comment).await?;

    // Get author profile
    let author_user = state
        .users
        .find_by_id(user.user_id)
        .await?
        .ok_or_else(|| ApiError::InternalServerError("Author not found".to_string()))?;

    let author = UserProfile {
        username: author_user.username,
        bio: author_user.bio,
        image: author_user.image,
        following: false,
    };

    let comment_response = CommentResponse {
        id: comment.id,
        created_at: comment.created_at,
        updated_at: comment.updated_at,
        body: comment.body,
        author,
    };

    Ok(Json(SingleCommentResponse {
        comment: comment_response,
    }))
}

pub async fn get_comments(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<MultipleCommentsResponse>, ApiError> {
    // Check if article exists
    let article = state
        .articles
        .find_by_slug(&slug)
        .await?
        .ok_or_else(|| ApiError::NotFound("Article not found".to_string()))?;

    // Get all comments for the article with author info
    let comments_with_authors = state.comments.list_for_article(article.id, None).await?;

    let comments: Vec<CommentResponse> = comments_with_authors
        .into_iter()
        .map(|cwa| CommentResponse {
            id: cwa.comment.id,
            created_at: cwa.comment.created_at,
            updated_at: cwa.comment.updated_at,
            body: cwa.comment.body,
            author: cwa.author,
        })
        .collect();

    Ok(Json(MultipleCommentsResponse { comments }))
}

pub async fn delete_comment(
    State(state): State<AppState>,
    Path((slug, comment_id)): Path<(String, String)>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    // Check if article exists
    let _article = state
        .articles
        .find_by_slug(&slug)
        .await?
        .ok_or_else(|| ApiError::NotFound("Article not found".to_string()))?;

    // Parse comment ID - treat invalid UUID as "not found"
    let comment_uuid = Uuid::parse_str(&comment_id)
        .map_err(|_| ApiError::NotFound("Comment not found".to_string()))?;

    // Check if comment exists and verify ownership
    let comment = state
        .comments
        .find_by_id(comment_uuid)
        .await?
        .ok_or_else(|| ApiError::NotFound("Comment not found".to_string()))?;

    // Verify the user is the comment author
    if comment.author_id != user.user_id {
        return Err(ApiError::Forbidden(
            "You can only delete your own comments".to_string(),
        ));
    }

    // Delete the comment
    state.comments.delete(comment_uuid).await?;

    Ok(StatusCode::NO_CONTENT)
}
