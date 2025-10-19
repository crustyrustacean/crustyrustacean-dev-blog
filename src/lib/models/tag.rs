// src/lib/models/tag.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct TagsResponse {
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ArticleTag {
    pub article_id: Uuid,
    pub tag_id: Uuid,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTag {
    #[validate(length(min = 1, max = 50))]
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct SingleTagResponse {
    pub tag: String,
}

#[derive(Debug, Clone)]
pub struct UserFavorite {
    pub user_id: Uuid,
    pub article_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct UserFollow {
    pub follower_id: Uuid,
    pub following_id: Uuid,
}
