// src/lib/routes/tags.rs

// src/lib/routes/tags.rs

use crate::state::CachedTags;
use crate::{
    ApiError, AppState,
    auth::AuthorUser,
    models::{SingleTagResponse, TagsResponse, UpdateTag},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
};
use chrono::Datelike;
use serde_json::json;
use std::time::{Duration, Instant};
use validator::Validate;

pub async fn get_tags(State(state): State<AppState>) -> Result<Json<TagsResponse>, ApiError> {
    {
        let cache = state.cached_tags.read().await;
        if let Some(cached) = cache.as_ref()
            && cached.cached_at.elapsed() < Duration::from_secs(300)
        {
            return Ok(Json(TagsResponse {
                tags: cached.tags.clone(),
            }));
        }
    }

    let conn = state
        .db
        .connect()
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let mut tag_rows = conn
        .query("SELECT name FROM tags ORDER BY name ASC", libsql::params![])
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let mut tags = Vec::new();

    while let Some(row) = tag_rows
        .next()
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
    {
        let tag_name: String = row
            .get(0)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        tags.push(tag_name);
    }

    *state.cached_tags.write().await = Some(CachedTags {
        tags: tags.clone(),
        cached_at: Instant::now(),
    });

    Ok(Json(TagsResponse { tags }))
}

pub async fn update_tag(
    State(state): State<AppState>,
    Path(old_name): Path<String>,
    _user: AuthorUser, // Requires author or admin role
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<SingleTagResponse>, ApiError> {
    let update_data: UpdateTag = serde_json::from_value(
        payload
            .get("tag")
            .ok_or_else(|| ApiError::BadRequest("Missing tag field".to_string()))?
            .clone(),
    )
    .map_err(|_| ApiError::BadRequest("Invalid tag data".to_string()))?;

    update_data
        .validate()
        .map_err(|e| ApiError::BadRequest(format!("Validation error: {}", e)))?;

    let conn = state
        .db
        .connect()
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    // Check if old tag exists
    let mut old_tag_rows = conn
        .query(
            "SELECT id FROM tags WHERE name = ?",
            libsql::params![old_name.clone()],
        )
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let tag_row = old_tag_rows
        .next()
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Tag not found".to_string()))?;

    let _tag_id: String = tag_row
        .get(0)
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    // Check if new name already exists (and it's not the same tag)
    if old_name != update_data.name {
        let mut new_tag_rows = conn
            .query(
                "SELECT id FROM tags WHERE name = ?",
                libsql::params![update_data.name.clone()],
            )
            .await
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

        if new_tag_rows
            .next()
            .await
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?
            .is_some()
        {
            return Err(ApiError::Conflict(
                "A tag with this name already exists".to_string(),
            ));
        }
    }

    // Update the tag
    conn.execute(
        "UPDATE tags SET name = ? WHERE name = ?",
        libsql::params![update_data.name.clone(), old_name],
    )
    .await
    .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(Json(SingleTagResponse {
        tag: update_data.name,
    }))
}

pub async fn delete_tag(
    State(state): State<AppState>,
    Path(name): Path<String>,
    _user: AuthorUser, // Requires author or admin role
) -> Result<StatusCode, ApiError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    // Check if tag exists
    let mut tag_rows = conn
        .query(
            "SELECT id FROM tags WHERE name = ?",
            libsql::params![name.clone()],
        )
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let tag_row = tag_rows
        .next()
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Tag not found".to_string()))?;

    let tag_id: String = tag_row
        .get(0)
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    // Delete article_tags relationships first (CASCADE should handle this, but being explicit)
    conn.execute(
        "DELETE FROM article_tags WHERE tag_id = ?",
        libsql::params![tag_id.clone()],
    )
    .await
    .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    // Delete the tag
    conn.execute("DELETE FROM tags WHERE id = ?", libsql::params![tag_id])
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_tags_admin_page(
    State(state): State<AppState>,
    user: AuthorUser,
) -> Result<impl IntoResponse, ApiError> {
    // Get user info for template context
    let conn = state
        .db
        .connect()
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let mut user_rows = conn
        .query(
            "SELECT username, email, bio, image FROM users WHERE id = ?",
            libsql::params![user.user_id.to_string()],
        )
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let user_info = if let Some(row) = user_rows
        .next()
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
    {
        let username: String = row
            .get(0)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        let email: String = row
            .get(1)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
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
        "title": "Manage Tags - Admin",
        "page": "TagsAdmin",
        "current_year": chrono::Utc::now().year(),
        "user": user_info,
    });

    let html = state
        .templates
        .render("admin/tags.html", &tera::Context::from_serialize(&context)?)
        .map_err(|e| ApiError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}
