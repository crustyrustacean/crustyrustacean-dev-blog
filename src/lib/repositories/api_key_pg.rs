// src/lib/repositories/api_key_pg.rs

//! PostgreSQL implementation of the ApiKeyRepository trait.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::api_key::{ApiKeyRecord, ApiKeyRepository, NewApiKey};
use super::error::{RepoResult, RepositoryError};

/// PostgreSQL implementation of the ApiKeyRepository.
#[derive(Debug, Clone)]
pub struct PgApiKeyRepository {
    db: sqlx::PgPool,
}

impl PgApiKeyRepository {
    /// Create a new PostgreSQL API key repository.
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ApiKeyRepository for PgApiKeyRepository {
    async fn create(&self, key: &NewApiKey) -> RepoResult<ApiKeyRecord> {
        let record = sqlx::query_as::<_, (Uuid, Uuid, String, String, DateTime<Utc>, Option<DateTime<Utc>>, Option<DateTime<Utc>>)>(
            r"INSERT INTO api_keys (user_id, name, key_hash)
              VALUES ($1, $2, $3)
              RETURNING id, user_id, name, key_hash, created_at, last_used_at, expires_at",
        )
        .bind(key.user_id)
        .bind(&key.name)
        .bind(&key.key_hash)
        .fetch_one(&self.db)
        .await
        .map(|(id, user_id, name, key_hash, created_at, last_used_at, expires_at)| ApiKeyRecord {
            id,
            user_id,
            name,
            key_hash,
            created_at,
            last_used_at,
            expires_at,
        })?;

        Ok(record)
    }

    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<ApiKeyRecord>> {
        let record = sqlx::query_as::<_, (Uuid, Uuid, String, String, DateTime<Utc>, Option<DateTime<Utc>>, Option<DateTime<Utc>>)>(
            "SELECT id, user_id, name, key_hash, created_at, last_used_at, expires_at FROM api_keys WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .map(|(id, user_id, name, key_hash, created_at, last_used_at, expires_at)| ApiKeyRecord {
            id,
            user_id,
            name,
            key_hash,
            created_at,
            last_used_at,
            expires_at,
        });

        Ok(record)
    }

    async fn find_by_hash(&self, hash: &str) -> RepoResult<Option<ApiKeyRecord>> {
        let record = sqlx::query_as::<_, (Uuid, Uuid, String, String, DateTime<Utc>, Option<DateTime<Utc>>, Option<DateTime<Utc>>)>(
            "SELECT id, user_id, name, key_hash, created_at, last_used_at, expires_at FROM api_keys WHERE key_hash = $1",
        )
        .bind(hash)
        .fetch_optional(&self.db)
        .await?
        .map(|(id, user_id, name, key_hash, created_at, last_used_at, expires_at)| ApiKeyRecord {
            id,
            user_id,
            name,
            key_hash,
            created_at,
            last_used_at,
            expires_at,
        });

        Ok(record)
    }

    async fn list_for_user(&self, user_id: Uuid) -> RepoResult<Vec<ApiKeyRecord>> {
        let rows = sqlx::query_as::<_, (Uuid, Uuid, String, String, DateTime<Utc>, Option<DateTime<Utc>>, Option<DateTime<Utc>>)>(
            "SELECT id, user_id, name, key_hash, created_at, last_used_at, expires_at \
             FROM api_keys WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, user_id, name, key_hash, created_at, last_used_at, expires_at)| {
                ApiKeyRecord {
                    id,
                    user_id,
                    name,
                    key_hash,
                    created_at,
                    last_used_at,
                    expires_at,
                }
            })
            .collect())
    }

    async fn delete(&self, id: Uuid) -> RepoResult<()> {
        let result = sqlx::query("DELETE FROM api_keys WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!(
                "API key with id '{}'",
                id
            )));
        }

        Ok(())
    }

    async fn update_last_used(&self, id: Uuid) -> RepoResult<()> {
        let result = sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!(
                "API key with id '{}'",
                id
            )));
        }

        Ok(())
    }
}
