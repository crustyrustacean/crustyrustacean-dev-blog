// src/lib/routes/api_keys.rs

use crate::{
    ApiError, AppState,
    auth::{AuthenticatedUser, generate_api_key, hash_api_key},
    models::{ApiKeyInfo, ApiKeyResponse, ApiKeysResponse, CreateApiKey},
    repositories::NewApiKey,
};
use axum::{
    extract::{Path, State},
    response::Json,
};
use uuid::Uuid;
use validator::Validate;

pub async fn create_api_key(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateApiKey>,
) -> Result<Json<ApiKeyResponse>, ApiError> {
    payload
        .validate()
        .map_err(|e| ApiError::BadRequest(format!("Validation error: {}", e)))?;

    // Generate API key
    let api_key = generate_api_key();
    let key_hash = hash_api_key(&api_key)?;

    let new_key = NewApiKey {
        user_id: user.user_id,
        name: payload.name.clone(),
        key_hash,
    };

    let record = state.api_keys.create(&new_key).await?;

    let response = ApiKeyResponse {
        id: record.id,
        name: record.name,
        key: api_key, // Return the raw key (only shown once)
        created_at: record.created_at,
    };

    Ok(Json(response))
}

pub async fn list_api_keys(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<ApiKeysResponse>, ApiError> {
    let records = state.api_keys.list_for_user(user.user_id).await?;

    let api_keys: Vec<ApiKeyInfo> = records
        .into_iter()
        .map(|r| ApiKeyInfo {
            id: r.id,
            name: r.name,
            created_at: r.created_at,
            last_used_at: r.last_used_at,
            expires_at: r.expires_at,
        })
        .collect();

    Ok(Json(ApiKeysResponse { api_keys }))
}

pub async fn delete_api_key(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(key_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // First verify the key belongs to the user
    let key = state
        .api_keys
        .find_by_id(key_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("API key not found".to_string()))?;

    if key.user_id != user.user_id {
        return Err(ApiError::NotFound("API key not found".to_string()));
    }

    // Delete the API key
    state.api_keys.delete(key_id).await?;

    Ok(Json(serde_json::json!({ "message": "API key deleted" })))
}
