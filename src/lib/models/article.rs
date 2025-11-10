// src/lib/models/article.rs

// dependencies
use super::UserProfile;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// struct type to represent an article
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

// struct type to represent an article response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleResponse {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rendered_body: Option<String>,
    #[serde(rename = "tagList")]
    pub tag_list: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub favorited: bool,
    pub favorites_count: i32,
    pub author: UserProfile,
}

// struct type to represent a new article
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateArticle {
    #[validate(length(min = 1))]
    pub title: String,
    #[validate(length(min = 1))]
    pub description: String,
    #[validate(length(min = 1))]
    pub body: String,
    #[serde(rename = "tagList")]
    pub tag_list: Option<Vec<String>>,
    pub category: Option<String>,
}

// struct type to represent an updated article
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateArticle {
    pub title: Option<String>,
    pub description: Option<String>,
    pub body: Option<String>,
    #[serde(rename = "tagList")]
    pub tag_list: Option<Vec<String>>,
    pub category: Option<String>,
}

// struct type to represent a response with one article
#[derive(Debug, Serialize)]
pub struct SingleArticleResponse {
    pub article: ArticleResponse,
}

// struct type to represent a response with multiple articles
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultipleArticlesResponse {
    pub articles: Vec<ArticleResponse>,
    pub articles_count: i32,
}

// struct type to represent a query for an article
#[derive(Debug, Deserialize)]
pub struct ArticleQuery {
    pub tag: Option<String>,
    pub author: Option<String>,
    pub favorited: Option<String>,
    pub category: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

// struct type to represent a query for a feed
#[derive(Debug, Deserialize)]
pub struct FeedQuery {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}
