// src/lib/models/category.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CategoriesResponse {
    pub categories: Vec<CategoryResponse>,
}

#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    pub name: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "articleCount")]
    pub article_count: i64,
}

#[derive(Debug, Serialize)]
pub struct SingleCategoryResponse {
    pub category: CategoryResponse,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCategory {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCategory {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
