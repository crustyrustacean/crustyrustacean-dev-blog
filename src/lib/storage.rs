// src/lib/storage.rs

// dependencies
use async_trait::async_trait;
use futures_util::stream::StreamExt;
use std::io;
use thiserror::Error;

// enum type for storage operation errors
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("File not found: {0}")]
    NotFound(String),

    #[error("File already exists: {0}")]
    AlreadyExists(String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Storage backend error: {0}")]
    Backend(String),

    #[error("Invalid file path: {0}")]
    InvalidPath(String),
}

// result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

// struct type to represent metadata about a stored file
#[derive(Clone, Debug)]
pub struct FileMetadata {
    pub path: String,
    pub size: u64,
    pub content_type: Option<String>,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
}

// abstract storage interface
#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn upload(
        &self,
        path: &str,
        content: Vec<u8>,
        content_type: Option<String>,
    ) -> StorageResult<FileMetadata>;
    async fn download(&self, path: &str) -> StorageResult<Vec<u8>>;
    async fn delete(&self, path: &str) -> StorageResult<()>;
    async fn exists(&self, path: &str) -> StorageResult<bool>;
    async fn metadata(&self, path: &str) -> StorageResult<FileMetadata>;
    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>>;
}

// OpenDAL implementation of StorageBackend
pub struct OpenDalStorage {
    operator: opendal::Operator,
}

impl OpenDalStorage {
    pub fn new(operator: opendal::Operator) -> Self {
        Self { operator }
    }
}

#[async_trait]
impl StorageBackend for OpenDalStorage {
    async fn upload(
        &self,
        path: &str,
        content: Vec<u8>,
        content_type: Option<String>,
    ) -> StorageResult<FileMetadata> {
        // Set content type if provided
        if let Some(ct) = content_type.as_ref() {
            self.operator
                .write_with(path, content.clone())
                .content_type(ct)
                .await
                .map_err(|e| StorageError::Backend(e.to_string()))?;
        } else {
            self.operator
                .write(path, content.clone())
                .await
                .map_err(|e| StorageError::Backend(e.to_string()))?;
        }

        // Return metadata
        Ok(FileMetadata {
            path: path.to_string(),
            size: content.len() as u64,
            content_type,
            last_modified: Some(chrono::Utc::now()),
        })
    }

    async fn download(&self, path: &str) -> StorageResult<Vec<u8>> {
        let bytes = self.operator.read(path).await.map_err(|e| {
            if e.kind() == opendal::ErrorKind::NotFound {
                StorageError::NotFound(path.to_string())
            } else {
                StorageError::Backend(e.to_string())
            }
        })?;

        Ok(bytes.to_vec())
    }

    async fn delete(&self, path: &str) -> StorageResult<()> {
        self.operator.delete(path).await.map_err(|e| {
            if e.kind() == opendal::ErrorKind::NotFound {
                StorageError::NotFound(path.to_string())
            } else {
                StorageError::Backend(e.to_string())
            }
        })
    }

    async fn exists(&self, path: &str) -> StorageResult<bool> {
        match self.operator.stat(path).await {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == opendal::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(StorageError::Backend(e.to_string())),
        }
    }

    async fn metadata(&self, path: &str) -> StorageResult<FileMetadata> {
        let meta = self.operator.stat(path).await.map_err(|e| {
            if e.kind() == opendal::ErrorKind::NotFound {
                StorageError::NotFound(path.to_string())
            } else {
                StorageError::Backend(e.to_string())
            }
        })?;

        Ok(FileMetadata {
            path: path.to_string(),
            size: meta.content_length(),
            content_type: meta.content_type().map(|s| s.to_string()),
            last_modified: meta.last_modified().map(|t| {
                chrono::DateTime::from_timestamp(t.timestamp(), 0).unwrap_or_else(chrono::Utc::now)
            }),
        })
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        let mut paths = Vec::new();
        let mut lister = self
            .operator
            .lister(prefix)
            .await
            .map_err(|e| StorageError::Backend(e.to_string()))?;

        while let Some(entry) = lister
            .next()
            .await
            .transpose()
            .map_err(|e| StorageError::Backend(e.to_string()))?
        {
            paths.push(entry.path().to_string());
        }

        Ok(paths)
    }
}

/// Helper function to generate storage paths
pub fn generate_media_path(user_id: &str, filename: &str) -> String {
    let now = chrono::Utc::now();
    let year = now.format("%Y");
    let month = now.format("%m");

    format!("media/{}/{}/{}/{}", user_id, year, month, filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_media_path() {
        let path = generate_media_path("user-123", "image.jpg");
        assert!(path.starts_with("media/user-123/"));
        assert!(path.ends_with("/image.jpg"));
    }
}
