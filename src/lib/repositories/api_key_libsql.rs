// src/lib/repositories/api_key_libsql.rs

//! LibSQL implementation of the ApiKeyRepository trait.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::database::DatabaseConnection;

use super::api_key::{ApiKeyRecord, ApiKeyRepository, NewApiKey};
use super::error::{RepoResult, RepositoryError};

/// LibSQL implementation of the ApiKeyRepository.
#[derive(Debug, Clone)]
pub struct LibSqlApiKeyRepository {
    db: DatabaseConnection,
}

impl LibSqlApiKeyRepository {
    /// Create a new LibSQL API key repository.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Parse a datetime string from the database.
    fn parse_datetime(s: &str) -> Result<DateTime<Utc>, RepositoryError> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| RepositoryError::InternalError(format!("Invalid datetime: {}", e)))
    }
}

#[async_trait]
impl ApiKeyRepository for LibSqlApiKeyRepository {
    async fn create(&self, key: &NewApiKey) -> RepoResult<ApiKeyRecord> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let id = Uuid::new_v4();
        let now = Utc::now();

        conn.execute(
            r"INSERT INTO api_keys (id, user_id, name, key_hash, created_at)
              VALUES (?, ?, ?, ?, ?)",
            vec![
                libsql::Value::Text(id.to_string()),
                libsql::Value::Text(key.user_id.to_string()),
                libsql::Value::Text(key.name.clone()),
                libsql::Value::Text(key.key_hash.clone()),
                libsql::Value::Text(now.to_rfc3339()),
            ],
        )
        .await?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| RepositoryError::InternalError("Failed to create API key".to_string()))
    }

    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<ApiKeyRecord>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                r"SELECT id, user_id, name, key_hash, created_at, last_used_at, expires_at
                  FROM api_keys WHERE id = ?",
                libsql::params![id.to_string()],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(self.row_to_api_key(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn find_by_hash(&self, hash: &str) -> RepoResult<Option<ApiKeyRecord>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                r"SELECT id, user_id, name, key_hash, created_at, last_used_at, expires_at
                  FROM api_keys WHERE key_hash = ?",
                libsql::params![hash],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(self.row_to_api_key(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn list_for_user(&self, user_id: Uuid) -> RepoResult<Vec<ApiKeyRecord>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                r"SELECT id, user_id, name, key_hash, created_at, last_used_at, expires_at
                  FROM api_keys WHERE user_id = ? ORDER BY created_at DESC",
                libsql::params![user_id.to_string()],
            )
            .await?;

        let mut keys = Vec::new();
        while let Some(row) = rows.next().await? {
            keys.push(self.row_to_api_key(&row)?);
        }

        Ok(keys)
    }

    async fn delete(&self, id: Uuid) -> RepoResult<()> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let result = conn
            .execute(
                "DELETE FROM api_keys WHERE id = ?",
                libsql::params![id.to_string()],
            )
            .await?;

        if result == 0 {
            return Err(RepositoryError::NotFound(format!(
                "API key with id '{}'",
                id
            )));
        }

        Ok(())
    }

    async fn update_last_used(&self, id: Uuid) -> RepoResult<()> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let now = Utc::now();

        let result = conn
            .execute(
                "UPDATE api_keys SET last_used_at = ? WHERE id = ?",
                vec![
                    libsql::Value::Text(now.to_rfc3339()),
                    libsql::Value::Text(id.to_string()),
                ],
            )
            .await?;

        if result == 0 {
            return Err(RepositoryError::NotFound(format!(
                "API key with id '{}'",
                id
            )));
        }

        Ok(())
    }
}

impl LibSqlApiKeyRepository {
    /// Convert a database row to an ApiKeyRecord.
    fn row_to_api_key(&self, row: &libsql::Row) -> RepoResult<ApiKeyRecord> {
        let id_str: String = row.get(0)?;
        let id = Uuid::parse_str(&id_str)
            .map_err(|e| RepositoryError::InternalError(format!("Invalid UUID: {}", e)))?;

        let user_id_str: String = row.get(1)?;
        let user_id = Uuid::parse_str(&user_id_str)
            .map_err(|e| RepositoryError::InternalError(format!("Invalid UUID: {}", e)))?;

        let name: String = row.get(2)?;
        let key_hash: String = row.get(3)?;

        let created_at_str: String = row.get(4)?;
        let created_at = Self::parse_datetime(&created_at_str)?;

        let last_used_at: Option<DateTime<Utc>> = row
            .get::<String>(5)
            .ok()
            .and_then(|s| Self::parse_datetime(&s).ok());

        let expires_at: Option<DateTime<Utc>> = row
            .get::<String>(6)
            .ok()
            .and_then(|s| Self::parse_datetime(&s).ok());

        Ok(ApiKeyRecord {
            id,
            user_id,
            name,
            key_hash,
            created_at,
            last_used_at,
            expires_at,
        })
    }
}
