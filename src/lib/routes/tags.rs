// src/lib/routes/tags.rs

use crate::{AppError, AppState, models::TagsResponse};
use axum::{extract::State, response::Json};

pub async fn get_tags(State(state): State<AppState>) -> Result<Json<TagsResponse>, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Query all unique tags from the tags table, ordered alphabetically
    let mut tag_rows = conn
        .query("SELECT name FROM tags ORDER BY name ASC", libsql::params![])
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let mut tags = Vec::new();

    while let Some(row) = tag_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        let tag_name: String = row
            .get(0)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        tags.push(tag_name);
    }

    Ok(Json(TagsResponse { tags }))
}
