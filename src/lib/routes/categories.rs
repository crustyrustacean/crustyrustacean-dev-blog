// src/lib/routes/categories.rs

use crate::{
    ApiError, AppState,
    auth::{AuthenticatedUser, AuthorUser},
    models::{
        CategoriesResponse, CategoryResponse, CreateCategory, SingleCategoryResponse,
        UpdateCategory,
    },
    repositories::{NewCategory, UpdateCategoryData},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
};
use chrono::Datelike;
use serde_json::json;
use validator::Validate;

/// GET /api/categories - List all categories
pub async fn get_categories(
    State(state): State<AppState>,
) -> Result<Json<CategoriesResponse>, ApiError> {
    let categories_data = state.categories.list_with_counts().await?;

    let categories: Vec<CategoryResponse> = categories_data
        .into_iter()
        .map(|cwc| CategoryResponse {
            name: cwc.category.name,
            slug: cwc.category.slug,
            description: cwc.category.description,
            article_count: cwc.article_count,
        })
        .collect();

    Ok(Json(CategoriesResponse { categories }))
}

/// POST /api/categories - Create a new category
pub async fn create_category(
    State(state): State<AppState>,
    _user: AuthorUser,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<SingleCategoryResponse>), ApiError> {
    let create_data: CreateCategory = serde_json::from_value(
        payload
            .get("category")
            .ok_or_else(|| ApiError::BadRequest("Missing category field".to_string()))?
            .clone(),
    )
    .map_err(|_| ApiError::BadRequest("Invalid category data".to_string()))?;

    create_data
        .validate()
        .map_err(|e| ApiError::BadRequest(format!("Validation error: {}", e)))?;

    // Check if category with same name already exists
    if state.categories.find_by_name(&create_data.name).await?.is_some() {
        return Err(ApiError::Conflict(
            "A category with this name or slug already exists".to_string(),
        ));
    }

    let new_category = NewCategory {
        name: create_data.name.clone(),
        description: create_data.description.clone(),
    };

    let category = state.categories.create(&new_category).await?;

    Ok((
        StatusCode::CREATED,
        Json(SingleCategoryResponse {
            category: CategoryResponse {
                name: category.name,
                slug: category.slug,
                description: category.description,
                article_count: 0,
            },
        }),
    ))
}

/// GET /api/categories/:slug - Get a specific category by slug
pub async fn get_category(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<SingleCategoryResponse>, ApiError> {
    let cwc = state
        .categories
        .find_by_slug_with_count(&slug)
        .await?
        .ok_or_else(|| ApiError::NotFound("Category not found".to_string()))?;

    Ok(Json(SingleCategoryResponse {
        category: CategoryResponse {
            name: cwc.category.name,
            slug: cwc.category.slug,
            description: cwc.category.description,
            article_count: cwc.article_count,
        },
    }))
}

/// PUT /api/categories/:slug - Update a category
pub async fn update_category(
    State(state): State<AppState>,
    Path(old_slug): Path<String>,
    _user: AuthorUser,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<SingleCategoryResponse>, ApiError> {
    let update_data: UpdateCategory = serde_json::from_value(
        payload
            .get("category")
            .ok_or_else(|| ApiError::BadRequest("Missing category field".to_string()))?
            .clone(),
    )
    .map_err(|_| ApiError::BadRequest("Invalid category data".to_string()))?;

    update_data
        .validate()
        .map_err(|e| ApiError::BadRequest(format!("Validation error: {}", e)))?;

    // Check if the category exists
    let existing = state
        .categories
        .find_by_slug(&old_slug)
        .await?
        .ok_or_else(|| ApiError::NotFound("Category not found".to_string()))?;

    // Check for name conflicts (excluding current category)
    if let Some(conflict) = state.categories.find_by_name(&update_data.name).await? {
        if conflict.id != existing.id {
            return Err(ApiError::Conflict(
                "A category with this name or slug already exists".to_string(),
            ));
        }
    }

    let data = UpdateCategoryData {
        name: Some(update_data.name.clone()),
        description: update_data.description.clone(),
    };

    let updated = state.categories.update(&old_slug, &data).await?;

    // Get article count for the updated category
    let cwc = state
        .categories
        .find_by_slug_with_count(&updated.slug)
        .await?
        .ok_or_else(|| ApiError::InternalServerError("Failed to get article count".to_string()))?;

    Ok(Json(SingleCategoryResponse {
        category: CategoryResponse {
            name: cwc.category.name,
            slug: cwc.category.slug,
            description: cwc.category.description,
            article_count: cwc.article_count,
        },
    }))
}

/// DELETE /api/categories/:slug - Delete a category
pub async fn delete_category(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    _user: AuthorUser,
) -> Result<StatusCode, ApiError> {
    state.categories.delete(&slug).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /admin/categories - Categories admin page
pub async fn get_categories_admin_page(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    // Get user info for template context
    let user_info = state.users.find_by_id(user.user_id).await?.map(|u| {
        json!({
            "username": u.username,
            "email": u.email,
            "bio": u.bio,
            "image": u.image
        })
    });

    let context = json!({
        "title": "Manage Categories - Admin",
        "page": "CategoriesAdmin",
        "current_year": chrono::Utc::now().year(),
        "user": user_info,
    });

    let html = state
        .templates
        .render(
            "admin/categories.html",
            &tera::Context::from_serialize(&context)?,
        )
        .map_err(|e| ApiError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}
