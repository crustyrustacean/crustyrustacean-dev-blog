// src/lib/models/comment.rs

use super::UserProfile;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    pub body: String,
    pub author_id: Uuid,
    pub article_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CommentResponse {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub body: String,
    pub author: UserProfile,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateComment {
    #[validate(length(min = 1))]
    pub body: String,
}

#[derive(Debug, Serialize)]
pub struct SingleCommentResponse {
    pub comment: CommentResponse,
}

#[derive(Debug, Serialize)]
pub struct MultipleCommentsResponse {
    pub comments: Vec<CommentResponse>,
}
