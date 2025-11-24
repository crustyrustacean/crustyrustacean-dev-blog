// src/lib/routes/api_keys.rs

use crate::{
    ApiError, AppState,
    auth::{AuthenticatedUser, generate_api_key, hash_api_key},
    models::{ApiKeyInfo, ApiKeyResponse, ApiKeysResponse, CreateApiKey},
};
use axum::{
    extract::{Path, State},
    response::Json,
};
use chrono::Utc;
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

    let conn = state
        .db
        .connect()
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    // Generate API key
    let api_key = generate_api_key();
    let key_hash = hash_api_key(&api_key)?;
    let key_id = Uuid::new_v4();
    let now = Utc::now();

    // Insert into database
    conn.execute(
        "INSERT INTO api_keys (id, user_id, name, key_hash, created_at) VALUES (?, ?, ?, ?, ?)",
        libsql::params![
            key_id.to_string(),
            user.user_id.to_string(),
            payload.name.clone(),
            key_hash,
            now.to_rfc3339(),
        ],
    )
    .await
    .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let response = ApiKeyResponse {
        id: key_id,
        name: payload.name,
        key: api_key,
        created_at: now,
    };

    Ok(Json(response))
}

pub async fn list_api_keys(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<ApiKeysResponse>, ApiError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let mut rows = conn
        .query(
            "SELECT id, name, created_at, last_used_at, expires_at FROM api_keys WHERE user_id = ? ORDER BY created_at DESC",
            libsql::params![user.user_id.to_string()],
        )
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let mut api_keys = Vec::new();

    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
    {
        let id: String = row
            .get(0)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        let name: String = row
            .get(1)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        let created_at: String = row
            .get(2)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        let last_used_at: Option<String> = row.get(3).ok();
        let expires_at: Option<String> = row.get(4).ok();

        let id_uuid = Uuid::parse_str(&id)
            .map_err(|_| ApiError::InternalServerError("Invalid key ID".to_string()))?;
        let created_at_dt = chrono::DateTime::parse_from_rfc3339(&created_at)
            .map_err(|_| ApiError::InternalServerError("Invalid created_at".to_string()))?
            .with_timezone(&Utc);
        let last_used_at_dt = last_used_at.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        });
        let expires_at_dt = expires_at.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        });

        api_keys.push(ApiKeyInfo {
            id: id_uuid,
            name,
            created_at: created_at_dt,
            last_used_at: last_used_at_dt,
            expires_at: expires_at_dt,
        });
    }

    Ok(Json(ApiKeysResponse { api_keys }))
}

pub async fn delete_api_key(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(key_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    // Delete the API key (only if it belongs to the user)
    let result = conn
        .execute(
            "DELETE FROM api_keys WHERE id = ? AND user_id = ?",
            libsql::params![key_id.to_string(), user.user_id.to_string()],
        )
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    if result == 0 {
        return Err(ApiError::NotFound("API key not found".to_string()));
    }

    Ok(Json(serde_json::json!({ "message": "API key deleted" })))
}
