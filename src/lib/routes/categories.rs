// src/lib/routes/categories.rs

use crate::{
    AppError, AppState,
    auth::{AuthenticatedUser, AuthorUser},
    models::{
        CategoriesResponse, CategoryResponse, CreateCategory, SingleCategoryResponse,
        UpdateCategory,
    },
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
};
use chrono::Datelike;
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

/// Convert a category name to a slug
/// Example: "Web Development" -> "web-development"
fn slugify(name: &str) -> String {
    name.to_lowercase()
        .trim()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("-")
}

/// GET /api/categories - List all categories
pub async fn get_categories(
    State(state): State<AppState>,
) -> Result<Json<CategoriesResponse>, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Get all categories with article counts
    let mut category_rows = conn
        .query(
            r#"
            SELECT c.id, c.name, c.slug, c.description,
                   COUNT(DISTINCT a.id) as article_count
            FROM categories c
            LEFT JOIN articles a ON a.category_id = c.id
            GROUP BY c.id, c.name, c.slug, c.description
            ORDER BY c.name ASC
            "#,
            libsql::params![],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let mut categories = Vec::new();

    while let Some(row) = category_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        let name: String = row
            .get(1)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let slug: String = row
            .get(2)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let description: Option<String> = row.get(3).ok();
        let article_count: i64 = row
            .get(4)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        categories.push(CategoryResponse {
            name,
            slug,
            description,
            article_count,
        });
    }

    Ok(Json(CategoriesResponse { categories }))
}

/// POST /api/categories - Create a new category
pub async fn create_category(
    State(state): State<AppState>,
    _user: AuthorUser,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<SingleCategoryResponse>), AppError> {
    let create_data: CreateCategory = serde_json::from_value(
        payload
            .get("category")
            .ok_or_else(|| AppError::BadRequest("Missing category field".to_string()))?
            .clone(),
    )
    .map_err(|_| AppError::BadRequest("Invalid category data".to_string()))?;

    create_data
        .validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Generate slug from name
    let slug = slugify(&create_data.name);

    // Check if category with same name or slug already exists
    let mut existing_rows = conn
        .query(
            "SELECT id FROM categories WHERE name = ? OR slug = ?",
            libsql::params![create_data.name.clone(), slug.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    if existing_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .is_some()
    {
        return Err(AppError::Conflict(
            "A category with this name or slug already exists".to_string(),
        ));
    }

    // Create the category
    let category_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO categories (id, name, slug, description) VALUES (?, ?, ?, ?)",
        libsql::params![
            category_id,
            create_data.name.clone(),
            slug.clone(),
            create_data.description.clone().unwrap_or_default()
        ],
    )
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(SingleCategoryResponse {
            category: CategoryResponse {
                name: create_data.name,
                slug,
                description: create_data.description,
                article_count: 0,
            },
        }),
    ))
}

/// GET /api/categories/:slug - Get a specific category by slug
pub async fn get_category(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<SingleCategoryResponse>, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Get category with article count
    let mut category_rows = conn
        .query(
            r#"
            SELECT c.id, c.name, c.slug, c.description,
                   COUNT(DISTINCT a.id) as article_count
            FROM categories c
            LEFT JOIN articles a ON a.category_id = c.id
            WHERE c.slug = ?
            GROUP BY c.id, c.name, c.slug, c.description
            "#,
            libsql::params![slug],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let category_row = category_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

    let name: String = category_row
        .get(1)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let slug: String = category_row
        .get(2)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let description: Option<String> = category_row.get(3).ok();
    let article_count: i64 = category_row
        .get(4)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(Json(SingleCategoryResponse {
        category: CategoryResponse {
            name,
            slug,
            description,
            article_count,
        },
    }))
}

/// PUT /api/categories/:slug - Update a category
pub async fn update_category(
    State(state): State<AppState>,
    Path(old_slug): Path<String>,
    _user: AuthorUser,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<SingleCategoryResponse>, AppError> {
    let update_data: UpdateCategory = serde_json::from_value(
        payload
            .get("category")
            .ok_or_else(|| AppError::BadRequest("Missing category field".to_string()))?
            .clone(),
    )
    .map_err(|_| AppError::BadRequest("Invalid category data".to_string()))?;

    update_data
        .validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Check if old category exists
    let mut old_category_rows = conn
        .query(
            "SELECT id FROM categories WHERE slug = ?",
            libsql::params![old_slug.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let category_row = old_category_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

    let category_id: String = category_row
        .get(0)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Generate new slug from new name
    let new_slug = slugify(&update_data.name);

    // Check if new name/slug conflicts with existing categories (excluding current one)
    if old_slug != new_slug {
        let mut conflict_rows = conn
            .query(
                "SELECT id FROM categories WHERE (name = ? OR slug = ?) AND id != ?",
                libsql::params![
                    update_data.name.clone(),
                    new_slug.clone(),
                    category_id.clone()
                ],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        if conflict_rows
            .next()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .is_some()
        {
            return Err(AppError::Conflict(
                "A category with this name or slug already exists".to_string(),
            ));
        }
    }

    // Update the category
    conn.execute(
        "UPDATE categories SET name = ?, slug = ?, description = ?, updated_at = datetime('now') WHERE id = ?",
        libsql::params![
            update_data.name.clone(),
            new_slug.clone(),
            update_data.description.clone().unwrap_or_default(),
            category_id.clone()
        ],
    )
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Get article count for response
    let mut count_rows = conn
        .query(
            "SELECT COUNT(*) FROM articles WHERE category_id = ?",
            libsql::params![category_id],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let count_row = count_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or_else(|| AppError::InternalServerError("Failed to get article count".to_string()))?;

    let article_count: i64 = count_row
        .get(0)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(Json(SingleCategoryResponse {
        category: CategoryResponse {
            name: update_data.name,
            slug: new_slug,
            description: update_data.description,
            article_count,
        },
    }))
}

/// DELETE /api/categories/:slug - Delete a category
pub async fn delete_category(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    _user: AuthorUser,
) -> Result<StatusCode, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Check if category exists
    let mut category_rows = conn
        .query(
            "SELECT id FROM categories WHERE slug = ?",
            libsql::params![slug.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let category_row = category_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

    let category_id: String = category_row
        .get(0)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Set category_id to NULL for all articles with this category
    conn.execute(
        "UPDATE articles SET category_id = NULL WHERE category_id = ?",
        libsql::params![category_id.clone()],
    )
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Delete the category
    conn.execute(
        "DELETE FROM categories WHERE id = ?",
        libsql::params![category_id],
    )
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /admin/categories - Categories admin page
pub async fn get_categories_admin_page(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    // Get user info for template context
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let mut user_rows = conn
        .query(
            "SELECT username, email, bio, image FROM users WHERE id = ?",
            libsql::params![user.user_id.to_string()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let user_info = if let Some(row) = user_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        let username: String = row
            .get(0)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let email: String = row
            .get(1)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let bio: Option<String> = row.get(2).ok();
        let image: Option<String> = row.get(3).ok();

        Some(json!({
            "username": username,
            "email": email,
            "bio": bio,
            "image": image
        }))
    } else {
        None
    };

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
        .map_err(|e| AppError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}
