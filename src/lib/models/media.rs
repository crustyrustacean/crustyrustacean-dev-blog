// src/lib/models/media.rs

use serde::{Deserialize, Serialize};

/// Media database model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Media {
    pub id: String,
    pub user_id: String,
    pub filename: String,
    pub storage_path: String,
    pub title: Option<String>,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
    pub description: Option<String>,
    pub mime_type: String,
    pub file_size: i64,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub uploaded_at: String,
    pub updated_at: String,
}

/// Media metadata for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaResponse {
    pub id: String,
    pub user_id: String,
    pub filename: String,
    pub title: Option<String>,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
    pub description: Option<String>,
    pub mime_type: String,
    pub file_size: i64,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub uploaded_at: String,
    pub updated_at: String,
    pub download_url: String,
}

/// Request to update media metadata
#[derive(Debug, Deserialize)]
pub struct UpdateMedia {
    pub title: Option<String>,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
    pub description: Option<String>,
}

/// Query parameters for listing media
#[derive(Debug, Deserialize)]
pub struct MediaQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub mime_type: Option<String>,
}

/// Single media response
#[derive(Debug, Serialize)]
pub struct SingleMediaResponse {
    pub media: MediaResponse,
}

/// Multiple media response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultipleMediaResponse {
    pub media: Vec<MediaResponse>,
    pub media_count: i64,
}

impl From<Media> for MediaResponse {
    fn from(media: Media) -> Self {
        Self {
            download_url: format!("/api/media/{}/download", media.id),
            id: media.id,
            user_id: media.user_id,
            filename: media.filename,
            title: media.title,
            alt_text: media.alt_text,
            caption: media.caption,
            description: media.description,
            mime_type: media.mime_type,
            file_size: media.file_size,
            width: media.width,
            height: media.height,
            uploaded_at: media.uploaded_at,
            updated_at: media.updated_at,
        }
    }
}
