// src/lib/repositories/api_key.rs

//! API Key repository trait and related types.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::error::RepoResult;

/// Data for creating a new API key.
#[derive(Debug, Clone)]
pub struct NewApiKey {
    pub user_id: Uuid,
    pub name: String,
    pub key_hash: String,
}

/// An API key record from the database.
#[derive(Debug, Clone)]
pub struct ApiKeyRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub key_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Repository trait for API key data access operations.
#[async_trait]
pub trait ApiKeyRepository: Send + Sync {
    /// Create a new API key.
    async fn create(&self, key: &NewApiKey) -> RepoResult<ApiKeyRecord>;

    /// Find an API key by ID.
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<ApiKeyRecord>>;

    /// Find an API key by hash.
    async fn find_by_hash(&self, hash: &str) -> RepoResult<Option<ApiKeyRecord>>;

    /// List all API keys for a user.
    async fn list_for_user(&self, user_id: Uuid) -> RepoResult<Vec<ApiKeyRecord>>;

    /// Delete an API key by ID.
    async fn delete(&self, id: Uuid) -> RepoResult<()>;

    /// Update the last_used_at timestamp.
    async fn update_last_used(&self, id: Uuid) -> RepoResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_api_key() {
        let key = NewApiKey {
            user_id: Uuid::new_v4(),
            name: "My API Key".to_string(),
            key_hash: "hash123".to_string(),
        };

        assert_eq!(key.name, "My API Key");
    }
}
