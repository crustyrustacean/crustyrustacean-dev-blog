// src/lib/repositories/user_libsql.rs

//! LibSQL implementation of the UserRepository trait.
//!
//! This module provides the concrete implementation for user data access
//! using the libsql database driver (compatible with Turso).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::database::DatabaseConnection;
use crate::models::{AdminUsersQuery, ProfilesQuery, Role, User, UserProfile};

use super::error::{RepoResult, RepositoryError};
use super::user::{NewUser, UpdateUser, UserRepository, UserSummary};

/// LibSQL implementation of the UserRepository.
#[derive(Debug, Clone)]
pub struct LibSqlUserRepository {
    db: DatabaseConnection,
}

impl LibSqlUserRepository {
    /// Create a new LibSQL user repository.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Helper to parse a datetime string from the database.
    fn parse_datetime(s: &str) -> RepoResult<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                // Try parsing without timezone (SQLite datetime format)
                chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| ndt.and_utc())
            })
            .map_err(|e| RepositoryError::InternalError(format!("Failed to parse datetime: {}", e)))
    }

    /// Helper to extract a User from a database row.
    ///
    /// Expected columns: id, username, email, password_hash, bio, image, disabled, role,
    ///                   email_verified, theme_preference, created_at, updated_at
    fn user_from_row(row: &libsql::Row) -> RepoResult<User> {
        let id_str: String = row.get(0)?;
        let id = Uuid::parse_str(&id_str)
            .map_err(|e| RepositoryError::InternalError(format!("Invalid UUID: {}", e)))?;

        let username: String = row.get(1)?;
        let email: String = row.get(2)?;
        let password_hash: String = row.get(3)?;
        let bio: Option<String> = row.get(4).ok();
        let image: Option<String> = row.get(5).ok();
        let disabled_int: i64 = row.get(6).unwrap_or(0);
        let role_str: String = row.get(7).unwrap_or_else(|_| "subscriber".to_string());
        let email_verified_int: i64 = row.get(8).unwrap_or(0);
        let theme_preference: Option<String> = row.get(9).ok();
        let created_at_str: String = row.get(10)?;
        let updated_at_str: String = row.get(11)?;

        let role = role_str
            .parse::<Role>()
            .map_err(|_| RepositoryError::InternalError("Invalid role in database".to_string()))?;

        let created_at = Self::parse_datetime(&created_at_str)?;
        let updated_at = Self::parse_datetime(&updated_at_str)?;

        Ok(User {
            id,
            username,
            email,
            password_hash,
            bio,
            image,
            disabled: disabled_int != 0,
            role,
            email_verified: email_verified_int != 0,
            theme_preference,
            created_at,
            updated_at,
        })
    }
}

#[async_trait]
impl UserRepository for LibSqlUserRepository {
    // ========================================================================
    // Create Operations
    // ========================================================================

    async fn create(&self, user: &NewUser) -> RepoResult<User> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let id = Uuid::new_v4();
        let now = Utc::now();

        conn.execute(
            r"INSERT INTO users (id, username, email, password_hash, role, email_verified, created_at, updated_at)
              VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            libsql::params![
                id.to_string(),
                user.username.clone(),
                user.email.clone(),
                user.password_hash.clone(),
                user.role.to_string(),
                0_i64, // email_verified = false
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )
        .await?;

        // Return the created user
        self.find_by_id(id).await?.ok_or_else(|| {
            RepositoryError::InternalError("Failed to retrieve created user".to_string())
        })
    }

    // ========================================================================
    // Read Operations
    // ========================================================================

    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<User>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                r"SELECT id, username, email, password_hash, bio, image, disabled, role,
                         email_verified, theme_preference, created_at, updated_at
                  FROM users WHERE id = ?",
                libsql::params![id.to_string()],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(Self::user_from_row(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn find_by_username(&self, username: &str) -> RepoResult<Option<User>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                r"SELECT id, username, email, password_hash, bio, image, disabled, role,
                         email_verified, theme_preference, created_at, updated_at
                  FROM users WHERE username = ?",
                libsql::params![username],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(Self::user_from_row(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn find_by_email(&self, email: &str) -> RepoResult<Option<User>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                r"SELECT id, username, email, password_hash, bio, image, disabled, role,
                         email_verified, theme_preference, created_at, updated_at
                  FROM users WHERE email = ?",
                libsql::params![email],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(Self::user_from_row(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn list(&self, query: &AdminUsersQuery) -> RepoResult<Vec<UserSummary>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let limit = query.limit.unwrap_or(20);
        let offset = query.offset.unwrap_or(0);

        // Build query based on filters
        let mut where_clauses = Vec::new();
        let mut params_vec: Vec<String> = Vec::new();

        // Search filter
        if let Some(search) = &query.search
            && !search.trim().is_empty()
        {
            where_clauses.push("(u.username LIKE ?1 OR u.email LIKE ?1)");
            params_vec.push(format!("%{}%", search.trim()));
        }

        // Status filter
        match query.status.as_deref() {
            Some("active") => where_clauses.push("u.disabled = 0"),
            Some("disabled") => where_clauses.push("u.disabled = 1"),
            _ => {} // "all" or None - no filter
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let query_str = format!(
            r"SELECT u.id, u.username, u.email, u.bio, u.image, u.disabled, u.role,
                     u.email_verified, u.created_at, COUNT(a.id) as article_count
              FROM users u
              LEFT JOIN articles a ON u.id = a.author_id
              {}
              GROUP BY u.id
              ORDER BY u.created_at DESC
              LIMIT ? OFFSET ?",
            where_clause
        );

        params_vec.push(limit.to_string());
        params_vec.push(offset.to_string());

        let params: Vec<libsql::Value> = params_vec
            .iter()
            .map(|s| libsql::Value::Text(s.clone()))
            .collect();

        let mut rows = conn
            .query(&query_str, libsql::params_from_iter(params))
            .await?;

        let mut users = Vec::new();
        while let Some(row) = rows.next().await? {
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid UUID: {}", e)))?;

            let username: String = row.get(1)?;
            let email: String = row.get(2)?;
            let bio: Option<String> = row.get(3).ok();
            let image: Option<String> = row.get(4).ok();
            let disabled_int: i64 = row.get(5).unwrap_or(0);
            let role_str: String = row.get(6).unwrap_or_else(|_| "subscriber".to_string());
            let email_verified_int: i64 = row.get(7).unwrap_or(0);
            let created_at_str: String = row.get(8)?;
            let article_count: i32 = row.get(9).unwrap_or(0);

            let role = role_str.parse::<Role>().unwrap_or(Role::Subscriber);
            let created_at = Self::parse_datetime(&created_at_str)?;

            users.push(UserSummary {
                id,
                username,
                email,
                bio,
                image,
                disabled: disabled_int != 0,
                role,
                email_verified: email_verified_int != 0,
                created_at,
                article_count,
            });
        }

        Ok(users)
    }

    async fn count(&self, query: &AdminUsersQuery) -> RepoResult<i32> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut where_clauses = Vec::new();
        let mut params_vec: Vec<String> = Vec::new();

        if let Some(search) = &query.search
            && !search.trim().is_empty()
        {
            where_clauses.push("(username LIKE ?1 OR email LIKE ?1)");
            params_vec.push(format!("%{}%", search.trim()));
        }

        match query.status.as_deref() {
            Some("active") => where_clauses.push("disabled = 0"),
            Some("disabled") => where_clauses.push("disabled = 1"),
            _ => {}
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let query_str = format!("SELECT COUNT(*) FROM users {}", where_clause);

        let params: Vec<libsql::Value> = params_vec
            .iter()
            .map(|s| libsql::Value::Text(s.clone()))
            .collect();

        let mut rows = conn
            .query(&query_str, libsql::params_from_iter(params))
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(row.get(0).unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    async fn list_profiles(
        &self,
        query: &ProfilesQuery,
        current_user_id: Option<Uuid>,
    ) -> RepoResult<Vec<UserProfile>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let limit = query.limit.unwrap_or(20).min(100);
        let offset = query.offset.unwrap_or(0);

        // Only show users with Author or Admin role (not Subscribers)
        let mut where_clauses = vec![
            "disabled = 0".to_string(),
            "(role = 'author' OR role = 'admin')".to_string(),
        ];
        let mut params: Vec<libsql::Value> = Vec::new();

        // Exclude current user from results
        if let Some(current_id) = current_user_id {
            where_clauses.push("id != ?".to_string());
            params.push(libsql::Value::Text(current_id.to_string()));
        }

        if let Some(search) = &query.search
            && !search.trim().is_empty()
        {
            where_clauses.push("(username LIKE ? OR bio LIKE ?)".to_string());
            let search_pattern = format!("%{}%", search.trim());
            params.push(libsql::Value::Text(search_pattern.clone()));
            params.push(libsql::Value::Text(search_pattern));
        }

        let where_clause = where_clauses.join(" AND ");

        let query_str = format!(
            r"SELECT id, username, bio, image FROM users
              WHERE {}
              ORDER BY username ASC
              LIMIT ? OFFSET ?",
            where_clause
        );

        params.push(libsql::Value::Integer(limit as i64));
        params.push(libsql::Value::Integer(offset as i64));

        let mut rows = conn
            .query(&query_str, libsql::params_from_iter(params))
            .await?;

        let mut profiles = Vec::new();
        while let Some(row) = rows.next().await? {
            let user_id_str: String = row.get(0)?;
            let user_id = Uuid::parse_str(&user_id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid UUID: {}", e)))?;

            let username: String = row.get(1)?;
            let bio: Option<String> = row.get(2).ok();
            let image: Option<String> = row.get(3).ok();

            // Check following status if current user is authenticated
            let following = if let Some(current_id) = current_user_id {
                self.is_following(current_id, user_id).await?
            } else {
                false
            };

            profiles.push(UserProfile {
                username,
                bio,
                image,
                following,
            });
        }

        Ok(profiles)
    }

    async fn count_profiles(&self, query: &ProfilesQuery) -> RepoResult<i32> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // Only count users with Author or Admin role (not Subscribers)
        let mut where_clauses = vec![
            "disabled = 0".to_string(),
            "(role = 'author' OR role = 'admin')".to_string(),
        ];
        let mut params: Vec<libsql::Value> = Vec::new();

        if let Some(search) = &query.search
            && !search.trim().is_empty()
        {
            where_clauses.push("(username LIKE ? OR bio LIKE ?)".to_string());
            let search_pattern = format!("%{}%", search.trim());
            params.push(libsql::Value::Text(search_pattern.clone()));
            params.push(libsql::Value::Text(search_pattern));
        }

        let where_clause = where_clauses.join(" AND ");
        let query_str = format!("SELECT COUNT(*) FROM users WHERE {}", where_clause);

        let mut rows = conn
            .query(&query_str, libsql::params_from_iter(params))
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(row.get(0).unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    async fn get_profile(
        &self,
        username: &str,
        current_user_id: Option<Uuid>,
    ) -> RepoResult<Option<UserProfile>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                "SELECT id, username, bio, image FROM users WHERE username = ? AND disabled = 0",
                libsql::params![username],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let user_id_str: String = row.get(0)?;
            let user_id = Uuid::parse_str(&user_id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid UUID: {}", e)))?;

            let username: String = row.get(1)?;
            let bio: Option<String> = row.get(2).ok();
            let image: Option<String> = row.get(3).ok();

            let following = if let Some(current_id) = current_user_id {
                self.is_following(current_id, user_id).await?
            } else {
                false
            };

            Ok(Some(UserProfile {
                username,
                bio,
                image,
                following,
            }))
        } else {
            Ok(None)
        }
    }

    // ========================================================================
    // Update Operations
    // ========================================================================

    async fn update(&self, id: Uuid, data: &UpdateUser) -> RepoResult<User> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // Check if user exists
        let existing = self.find_by_id(id).await?;
        if existing.is_none() {
            return Err(RepositoryError::NotFound(format!("User with id {}", id)));
        }

        let mut updates = Vec::new();
        let mut params: Vec<libsql::Value> = Vec::new();

        if let Some(username) = &data.username {
            updates.push("username = ?");
            params.push(libsql::Value::Text(username.clone()));
        }
        if let Some(email) = &data.email {
            updates.push("email = ?");
            params.push(libsql::Value::Text(email.clone()));
        }
        if let Some(bio) = &data.bio {
            updates.push("bio = ?");
            params.push(libsql::Value::Text(bio.clone()));
        }
        if let Some(image) = &data.image {
            updates.push("image = ?");
            params.push(libsql::Value::Text(image.clone()));
        }
        if let Some(disabled) = data.disabled {
            updates.push("disabled = ?");
            params.push(libsql::Value::Integer(if disabled { 1 } else { 0 }));
        }
        if let Some(role) = &data.role {
            updates.push("role = ?");
            params.push(libsql::Value::Text(role.to_string()));
        }
        if let Some(theme) = &data.theme_preference {
            updates.push("theme_preference = ?");
            params.push(libsql::Value::Text(theme.clone()));
        }

        if updates.is_empty() {
            // Nothing to update, return existing user
            return self.find_by_id(id).await?.ok_or_else(|| {
                RepositoryError::InternalError("User disappeared during update".to_string())
            });
        }

        updates.push("updated_at = ?");
        params.push(libsql::Value::Text(Utc::now().to_rfc3339()));

        params.push(libsql::Value::Text(id.to_string()));

        let update_query = format!("UPDATE users SET {} WHERE id = ?", updates.join(", "));

        conn.execute(&update_query, libsql::params_from_iter(params))
            .await?;

        self.find_by_id(id).await?.ok_or_else(|| {
            RepositoryError::InternalError("Failed to retrieve updated user".to_string())
        })
    }

    async fn update_password(&self, id: Uuid, password_hash: &str) -> RepoResult<()> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let now = Utc::now();

        let result = conn
            .execute(
                "UPDATE users SET password_hash = ?, password_changed_at = ?, updated_at = ? WHERE id = ?",
                libsql::params![
                    password_hash,
                    now.to_rfc3339(),
                    now.to_rfc3339(),
                    id.to_string(),
                ],
            )
            .await?;

        if result == 0 {
            return Err(RepositoryError::NotFound(format!("User with id {}", id)));
        }

        Ok(())
    }

    async fn verify_email(&self, id: Uuid) -> RepoResult<()> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let result = conn
            .execute(
                "UPDATE users SET email_verified = 1, updated_at = ? WHERE id = ?",
                libsql::params![Utc::now().to_rfc3339(), id.to_string()],
            )
            .await?;

        if result == 0 {
            return Err(RepositoryError::NotFound(format!("User with id {}", id)));
        }

        Ok(())
    }

    async fn update_theme_preference(&self, id: Uuid, theme: &str) -> RepoResult<()> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let result = conn
            .execute(
                "UPDATE users SET theme_preference = ?, updated_at = ? WHERE id = ?",
                libsql::params![theme, Utc::now().to_rfc3339(), id.to_string()],
            )
            .await?;

        if result == 0 {
            return Err(RepositoryError::NotFound(format!("User with id {}", id)));
        }

        Ok(())
    }

    // ========================================================================
    // Delete Operations
    // ========================================================================

    async fn delete(&self, id: Uuid) -> RepoResult<()> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let result = conn
            .execute(
                "DELETE FROM users WHERE id = ?",
                libsql::params![id.to_string()],
            )
            .await?;

        if result == 0 {
            return Err(RepositoryError::NotFound(format!("User with id {}", id)));
        }

        Ok(())
    }

    // ========================================================================
    // Relationship Operations
    // ========================================================================

    async fn follow(&self, follower_id: Uuid, following_id: Uuid) -> RepoResult<()> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        conn.execute(
            "INSERT OR IGNORE INTO user_follows (follower_id, following_id) VALUES (?, ?)",
            libsql::params![follower_id.to_string(), following_id.to_string()],
        )
        .await?;

        Ok(())
    }

    async fn unfollow(&self, follower_id: Uuid, following_id: Uuid) -> RepoResult<()> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        conn.execute(
            "DELETE FROM user_follows WHERE follower_id = ? AND following_id = ?",
            libsql::params![follower_id.to_string(), following_id.to_string()],
        )
        .await?;

        Ok(())
    }

    async fn is_following(&self, follower_id: Uuid, following_id: Uuid) -> RepoResult<bool> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                "SELECT 1 FROM user_follows WHERE follower_id = ? AND following_id = ?",
                libsql::params![follower_id.to_string(), following_id.to_string()],
            )
            .await?;

        Ok(rows.next().await?.is_some())
    }

    async fn get_following(&self, user_id: Uuid) -> RepoResult<Vec<Uuid>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                "SELECT following_id FROM user_follows WHERE follower_id = ?",
                libsql::params![user_id.to_string()],
            )
            .await?;

        let mut following = Vec::new();
        while let Some(row) = rows.next().await? {
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid UUID: {}", e)))?;
            following.push(id);
        }

        Ok(following)
    }

    async fn get_followers(&self, user_id: Uuid) -> RepoResult<Vec<Uuid>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                "SELECT follower_id FROM user_follows WHERE following_id = ?",
                libsql::params![user_id.to_string()],
            )
            .await?;

        let mut followers = Vec::new();
        while let Some(row) = rows.next().await? {
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid UUID: {}", e)))?;
            followers.push(id);
        }

        Ok(followers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_parse_datetime_rfc3339() {
        let datetime = LibSqlUserRepository::parse_datetime("2024-01-15T10:30:00+00:00").unwrap();
        assert_eq!(datetime.year(), 2024);
        assert_eq!(datetime.month(), 1);
        assert_eq!(datetime.day(), 15);
    }

    #[test]
    fn test_parse_datetime_sqlite_format() {
        let datetime = LibSqlUserRepository::parse_datetime("2024-01-15 10:30:00").unwrap();
        assert_eq!(datetime.year(), 2024);
        assert_eq!(datetime.month(), 1);
        assert_eq!(datetime.day(), 15);
    }
}
