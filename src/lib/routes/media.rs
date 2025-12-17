// src/lib/routes/media.rs

use crate::{
    auth::AuthenticatedUser,
    errors::ApiError,
    models::{MediaQuery, MultipleMediaResponse, SingleMediaResponse, UpdateMedia},
    repositories::{NewMedia, UpdateMediaData},
    state::AppState,
    storage::generate_media_path,
};
use axum::{
    Json,
    body::Bytes,
    extract::{Multipart, Path, Query, State},
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
};
use chrono::Datelike;
use serde_json::json;
use uuid::Uuid;

/// Upload media file
/// POST /api/media
pub async fn upload_media(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    mut multipart: Multipart,
) -> Result<Json<SingleMediaResponse>, ApiError> {
    let mut filename: Option<String> = None;
    let mut file_data: Option<Vec<u8>> = None;
    let mut content_type: Option<String> = None;
    let mut title: Option<String> = None;
    let mut alt_text: Option<String> = None;
    let mut caption: Option<String> = None;
    let mut description: Option<String> = None;

    // Process multipart form data
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Failed to read multipart field: {}", e)))?
    {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "file" => {
                filename = field.file_name().map(|s| s.to_string());
                content_type = field.content_type().map(|s| s.to_string());

                // Validate content type (images only)
                if let Some(ct) = &content_type
                    && !ct.starts_with("image/")
                {
                    return Err(ApiError::BadRequest(
                        "Only image files are allowed".to_string(),
                    ));
                }

                file_data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| {
                            ApiError::BadRequest(format!("Failed to read file data: {}", e))
                        })?
                        .to_vec(),
                );
            }
            "title" => {
                title =
                    Some(field.text().await.map_err(|e| {
                        ApiError::BadRequest(format!("Failed to read title: {}", e))
                    })?);
            }
            "alt_text" => {
                alt_text = Some(field.text().await.map_err(|e| {
                    ApiError::BadRequest(format!("Failed to read alt_text: {}", e))
                })?);
            }
            "caption" => {
                caption =
                    Some(field.text().await.map_err(|e| {
                        ApiError::BadRequest(format!("Failed to read caption: {}", e))
                    })?);
            }
            "description" => {
                description = Some(field.text().await.map_err(|e| {
                    ApiError::BadRequest(format!("Failed to read description: {}", e))
                })?);
            }
            _ => {}
        }
    }

    // Validate required fields
    let filename = filename.ok_or_else(|| ApiError::BadRequest("No file provided".to_string()))?;

    let file_data =
        file_data.ok_or_else(|| ApiError::BadRequest("No file data provided".to_string()))?;

    // Check file size (50MB limit)
    const MAX_FILE_SIZE: usize = 50 * 1024 * 1024;
    if file_data.len() > MAX_FILE_SIZE {
        return Err(ApiError::BadRequest(
            "File size exceeds 50MB limit".to_string(),
        ));
    }

    // Generate unique filename
    let extension = std::path::Path::new(&filename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("jpg");

    let unique_filename = format!("{}.{}", Uuid::new_v4(), extension);
    let storage_path = generate_media_path(&user.user_id.to_string(), &unique_filename);

    // Upload to storage
    let file_metadata = state
        .storage
        .upload(&storage_path, file_data.clone(), content_type.clone())
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Storage upload failed: {}", e)))?;

    // Create media record in database
    let new_media = NewMedia {
        user_id: user.user_id.to_string(),
        filename,
        storage_path,
        title,
        alt_text,
        caption,
        description,
        mime_type: content_type.unwrap_or_else(|| "application/octet-stream".to_string()),
        file_size: file_metadata.size as i64,
        width: None,
        height: None,
    };

    let media = state.media.create(&new_media).await?;

    Ok(Json(SingleMediaResponse {
        media: media.into(),
    }))
}

/// List media files
/// GET /api/media?limit=20&offset=0&mime_type=image/jpeg
pub async fn list_media(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<MediaQuery>,
) -> Result<Json<MultipleMediaResponse>, ApiError> {
    let media_list = state
        .media
        .list_for_user(&user.user_id.to_string(), &query)
        .await?;

    let media_count = state
        .media
        .count_for_user(&user.user_id.to_string(), &query)
        .await?;

    Ok(Json(MultipleMediaResponse {
        media: media_list.into_iter().map(|m| m.into()).collect(),
        media_count,
    }))
}

/// Get media metadata
/// GET /api/media/:id
pub async fn get_media_metadata(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<SingleMediaResponse>, ApiError> {
    let media = state
        .media
        .find_by_id(&id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Media not found".to_string()))?;

    // Verify ownership
    if media.user_id != user.user_id.to_string() {
        return Err(ApiError::Forbidden(
            "You don't have permission to access this media".to_string(),
        ));
    }

    Ok(Json(SingleMediaResponse {
        media: media.into(),
    }))
}

/// Update media metadata
/// PUT /api/media/:id
pub async fn update_media_metadata(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateMedia>,
) -> Result<Json<SingleMediaResponse>, ApiError> {
    // First, check if media exists and user owns it
    let existing = state
        .media
        .find_by_id(&id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Media not found".to_string()))?;

    if existing.user_id != user.user_id.to_string() {
        return Err(ApiError::Forbidden(
            "You don't have permission to update this media".to_string(),
        ));
    }

    // Update the media
    let update_data = UpdateMediaData {
        title: payload.title,
        alt_text: payload.alt_text,
        caption: payload.caption,
        description: payload.description,
    };

    let media = state.media.update(&id, &update_data).await?;

    Ok(Json(SingleMediaResponse {
        media: media.into(),
    }))
}

/// Delete media
/// DELETE /api/media/:id
pub async fn delete_media(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    // Fetch media to get storage path and verify ownership
    let media = state
        .media
        .find_by_id(&id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Media not found".to_string()))?;

    if media.user_id != user.user_id.to_string() {
        return Err(ApiError::Forbidden(
            "You don't have permission to delete this media".to_string(),
        ));
    }

    // Delete from storage
    state
        .storage
        .delete(&media.storage_path)
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Storage deletion failed: {}", e)))?;

    // Delete from database
    state.media.delete(&id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Download media file
/// GET /api/media/:id/download
pub async fn download_media(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let media = state
        .media
        .find_by_id(&id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Media not found".to_string()))?;

    // Download from storage
    let file_data = state
        .storage
        .download(&media.storage_path)
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Storage download failed: {}", e)))?;

    // Return file with appropriate headers
    let mut response = (StatusCode::OK, Bytes::from(file_data)).into_response();

    response.headers_mut().insert(
        header::CONTENT_TYPE,
        media
            .mime_type
            .parse()
            .unwrap_or_else(|_| "application/octet-stream".parse().unwrap()),
    );

    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        format!("inline; filename=\"{}\"", media.filename)
            .parse()
            .unwrap(),
    );

    Ok(response)
}

/// Get media library admin page
/// GET /admin/media
pub async fn get_media_library_page(
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
        "title": "Media Library - Admin",
        "page": "MediaLibrary",
        "current_year": chrono::Utc::now().year(),
        "user": user_info,
    });

    let html = state
        .templates
        .render(
            "media/library.html",
            &tera::Context::from_serialize(&context)?,
        )
        .map_err(|e| ApiError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}
