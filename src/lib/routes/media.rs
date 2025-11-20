// src/lib/routes/media.rs

use crate::{
    auth::AuthenticatedUser,
    errors::AppError,
    models::{Media, MediaQuery, MultipleMediaResponse, SingleMediaResponse, UpdateMedia},
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
use chrono::{Datelike, Utc};
use serde_json::json;
use uuid::Uuid;

/// Upload media file
/// POST /api/media
pub async fn upload_media(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    mut multipart: Multipart,
) -> Result<Json<SingleMediaResponse>, AppError> {
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
        .map_err(|e| AppError::BadRequest(format!("Failed to read multipart field: {}", e)))?
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
                    return Err(AppError::BadRequest(
                        "Only image files are allowed".to_string(),
                    ));
                }

                file_data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| {
                            AppError::BadRequest(format!("Failed to read file data: {}", e))
                        })?
                        .to_vec(),
                );
            }
            "title" => {
                title =
                    Some(field.text().await.map_err(|e| {
                        AppError::BadRequest(format!("Failed to read title: {}", e))
                    })?);
            }
            "alt_text" => {
                alt_text = Some(field.text().await.map_err(|e| {
                    AppError::BadRequest(format!("Failed to read alt_text: {}", e))
                })?);
            }
            "caption" => {
                caption =
                    Some(field.text().await.map_err(|e| {
                        AppError::BadRequest(format!("Failed to read caption: {}", e))
                    })?);
            }
            "description" => {
                description = Some(field.text().await.map_err(|e| {
                    AppError::BadRequest(format!("Failed to read description: {}", e))
                })?);
            }
            _ => {}
        }
    }

    // Validate required fields
    let filename = filename.ok_or_else(|| AppError::BadRequest("No file provided".to_string()))?;

    let file_data =
        file_data.ok_or_else(|| AppError::BadRequest("No file data provided".to_string()))?;

    // Check file size (50MB limit)
    const MAX_FILE_SIZE: usize = 50 * 1024 * 1024;
    if file_data.len() > MAX_FILE_SIZE {
        return Err(AppError::BadRequest(
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
        .map_err(|e| AppError::InternalServerError(format!("Storage upload failed: {}", e)))?;

    // Create media record in database
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        r#"
        INSERT INTO media_library (
            id, user_id, filename, storage_path, title, alt_text, 
            caption, description, mime_type, file_size, width, height,
            uploaded_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        libsql::params![
            id.clone(),
            user.user_id.to_string(),
            filename.clone(),
            storage_path.clone(),
            title.clone(),
            alt_text.clone(),
            caption.clone(),
            description.clone(),
            content_type
                .clone()
                .unwrap_or_else(|| "application/octet-stream".to_string()),
            file_metadata.size as i64,
            None::<i64>, // width
            None::<i64>, // height
            now.clone(),
            now.clone(),
        ],
    )
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let media = Media {
        id: id.clone(),
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
        uploaded_at: now.clone(),
        updated_at: now,
    };

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
) -> Result<Json<MultipleMediaResponse>, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    let (sql, params): (String, Vec<libsql::Value>) = if let Some(mime_type) = query.mime_type {
        (
            r#"
            SELECT id, user_id, filename, storage_path, title, alt_text,
                   caption, description, mime_type, file_size, width, height,
                   uploaded_at, updated_at
            FROM media_library
            WHERE user_id = ? AND mime_type = ?
            ORDER BY uploaded_at DESC
            LIMIT ? OFFSET ?
            "#
            .to_string(),
            vec![
                user.user_id.to_string().into(),
                mime_type.into(),
                limit.into(),
                offset.into(),
            ],
        )
    } else {
        (
            r#"
            SELECT id, user_id, filename, storage_path, title, alt_text,
                   caption, description, mime_type, file_size, width, height,
                   uploaded_at, updated_at
            FROM media_library
            WHERE user_id = ?
            ORDER BY uploaded_at DESC
            LIMIT ? OFFSET ?
            "#
            .to_string(),
            vec![user.user_id.to_string().into(), limit.into(), offset.into()],
        )
    };

    let mut rows = conn
        .query(&sql, libsql::params_from_iter(params))
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let mut media_list = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        let media = Media {
            id: row
                .get(0)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            user_id: row
                .get(1)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            filename: row
                .get(2)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            storage_path: row
                .get(3)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            title: row
                .get(4)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            alt_text: row
                .get(5)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            caption: row
                .get(6)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            description: row
                .get(7)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            mime_type: row
                .get(8)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            file_size: row
                .get(9)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            width: row
                .get(10)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            height: row
                .get(11)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            uploaded_at: row
                .get(12)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            updated_at: row
                .get(13)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
        };
        media_list.push(media.into());
    }

    // Get total count
    let mut count_rows = conn
        .query(
            "SELECT COUNT(*) as count FROM media_library WHERE user_id = ?",
            libsql::params![user.user_id.to_string()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let media_count = if let Some(row) = count_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        row.get::<i64>(0)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
    } else {
        0
    };

    Ok(Json(MultipleMediaResponse {
        media: media_list,
        media_count,
    }))
}

/// Get media metadata
/// GET /api/media/:id
pub async fn get_media_metadata(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<SingleMediaResponse>, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let mut rows = conn
        .query(
            r#"
        SELECT id, user_id, filename, storage_path, title, alt_text,
               caption, description, mime_type, file_size, width, height,
               uploaded_at, updated_at
        FROM media_library
        WHERE id = ?
        "#,
            libsql::params![id],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let media = if let Some(row) = rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        Media {
            id: row
                .get(0)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            user_id: row
                .get(1)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            filename: row
                .get(2)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            storage_path: row
                .get(3)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            title: row
                .get(4)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            alt_text: row
                .get(5)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            caption: row
                .get(6)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            description: row
                .get(7)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            mime_type: row
                .get(8)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            file_size: row
                .get(9)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            width: row
                .get(10)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            height: row
                .get(11)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            uploaded_at: row
                .get(12)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            updated_at: row
                .get(13)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
        }
    } else {
        return Err(AppError::NotFound("Media not found".to_string()));
    };

    // Verify ownership
    if media.user_id != user.user_id.to_string() {
        return Err(AppError::Forbidden(
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
) -> Result<Json<SingleMediaResponse>, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // First, check if media exists and user owns it
    let mut check_rows = conn
        .query(
            "SELECT user_id FROM media_library WHERE id = ?",
            libsql::params![id.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let owner_id: String = if let Some(row) = check_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        row.get(0)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
    } else {
        return Err(AppError::NotFound("Media not found".to_string()));
    };

    if owner_id != user.user_id.to_string() {
        return Err(AppError::Forbidden(
            "You don't have permission to update this media".to_string(),
        ));
    }

    // Update the media
    let now = Utc::now().to_rfc3339();
    conn.execute(
        r#"
        UPDATE media_library
        SET title = ?, alt_text = ?, caption = ?, description = ?, updated_at = ?
        WHERE id = ?
        "#,
        libsql::params![
            payload.title,
            payload.alt_text,
            payload.caption,
            payload.description,
            now,
            id.clone(),
        ],
    )
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Fetch updated media
    let mut rows = conn
        .query(
            r#"
        SELECT id, user_id, filename, storage_path, title, alt_text,
               caption, description, mime_type, file_size, width, height,
               uploaded_at, updated_at
        FROM media_library
        WHERE id = ?
        "#,
            libsql::params![id],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let media = if let Some(row) = rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        Media {
            id: row
                .get(0)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            user_id: row
                .get(1)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            filename: row
                .get(2)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            storage_path: row
                .get(3)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            title: row
                .get(4)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            alt_text: row
                .get(5)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            caption: row
                .get(6)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            description: row
                .get(7)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            mime_type: row
                .get(8)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            file_size: row
                .get(9)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            width: row
                .get(10)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            height: row
                .get(11)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            uploaded_at: row
                .get(12)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            updated_at: row
                .get(13)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
        }
    } else {
        return Err(AppError::NotFound(
            "Media not found after update".to_string(),
        ));
    };

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
) -> Result<StatusCode, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Fetch media to get storage path and verify ownership
    let mut rows = conn
        .query(
            "SELECT user_id, storage_path FROM media_library WHERE id = ?",
            libsql::params![id.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let (owner_id, storage_path): (String, String) = if let Some(row) = rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        (
            row.get(0)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            row.get(1)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
        )
    } else {
        return Err(AppError::NotFound("Media not found".to_string()));
    };

    if owner_id != user.user_id.to_string() {
        return Err(AppError::Forbidden(
            "You don't have permission to delete this media".to_string(),
        ));
    }

    // Delete from storage
    state
        .storage
        .delete(&storage_path)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Storage deletion failed: {}", e)))?;

    // Delete from database
    conn.execute(
        "DELETE FROM media_library WHERE id = ?",
        libsql::params![id],
    )
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

/// Download media file
/// GET /api/media/:id/download
pub async fn download_media(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let mut rows = conn
        .query(
            "SELECT storage_path, filename, mime_type FROM media_library WHERE id = ?",
            libsql::params![id],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let (storage_path, filename, mime_type): (String, String, String) = if let Some(row) = rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        (
            row.get(0)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            row.get(1)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            row.get(2)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
        )
    } else {
        return Err(AppError::NotFound("Media not found".to_string()));
    };

    // Download from storage
    let file_data =
        state.storage.download(&storage_path).await.map_err(|e| {
            AppError::InternalServerError(format!("Storage download failed: {}", e))
        })?;

    // Return file with appropriate headers
    let mut response = (StatusCode::OK, Bytes::from(file_data)).into_response();

    response.headers_mut().insert(
        header::CONTENT_TYPE,
        mime_type
            .parse()
            .unwrap_or_else(|_| "application/octet-stream".parse().unwrap()),
    );

    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        format!("inline; filename=\"{}\"", filename)
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
        .map_err(|e| AppError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}
