// src/lib/models/article.rs

use super::UserProfile;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub author_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ArticleResponse {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub tag_list: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub favorited: bool,
    pub favorites_count: i32,
    pub author: UserProfile,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateArticle {
    #[validate(length(min = 1))]
    pub title: String,
    #[validate(length(min = 1))]
    pub description: String,
    #[validate(length(min = 1))]
    pub body: String,
    pub tag_list: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateArticle {
    pub title: Option<String>,
    pub description: Option<String>,
    pub body: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SingleArticleResponse {
    pub article: ArticleResponse,
}

#[derive(Debug, Serialize)]
pub struct MultipleArticlesResponse {
    pub articles: Vec<ArticleResponse>,
    pub articles_count: i32,
}

#[derive(Debug, Deserialize)]
pub struct ArticleQuery {
    pub tag: Option<String>,
    pub author: Option<String>,
    pub favorited: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct FeedQuery {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}
