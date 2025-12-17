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

    let tags = state.tags.list_all().await?;

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

    let tag = state.tags.rename(&old_name, &update_data.name).await?;

    Ok(Json(SingleTagResponse { tag: tag.name }))
}

pub async fn delete_tag(
    State(state): State<AppState>,
    Path(name): Path<String>,
    _user: AuthorUser, // Requires author or admin role
) -> Result<StatusCode, ApiError> {
    state.tags.delete(&name).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_tags_admin_page(
    State(state): State<AppState>,
    user: AuthorUser,
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
