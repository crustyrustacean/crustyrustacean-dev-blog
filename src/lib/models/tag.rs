// src/lib/models/tag.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

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