// src/lib/repositories/user.rs

//! User repository trait and related types.
//!
//! This module defines the interface for user data access operations,
//! abstracting away the underlying database implementation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::{AdminUsersQuery, ProfilesQuery, Role, User, UserProfile};

use super::error::RepoResult;

/// Data required to create a new user.
#[derive(Debug, Clone)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: Role,
}

/// Data for updating an existing user.
#[derive(Debug, Clone, Default)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub disabled: Option<bool>,
    pub role: Option<Role>,
    pub theme_preference: Option<String>,
}

/// Summary of a user with article count for admin views.
#[derive(Debug, Clone)]
pub struct UserSummary {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub disabled: bool,
    pub role: Role,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub article_count: i32,
}

/// Repository trait for user data access operations.
///
/// This trait provides an abstraction over user data storage, enabling
/// the business logic to be decoupled from the database implementation.
/// Implementations might use SQLite, PostgreSQL, or an in-memory store
/// for testing.
#[async_trait]
pub trait UserRepository: Send + Sync {
    // ========================================================================
    // Create Operations
    // ========================================================================

    /// Create a new user.
    ///
    /// # Errors
    /// - `AlreadyExists` if username or email already exists
    /// - `DatabaseError` for other database errors
    async fn create(&self, user: &NewUser) -> RepoResult<User>;

    // ========================================================================
    // Read Operations
    // ========================================================================

    /// Find a user by their unique ID.
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<User>>;

    /// Find a user by their username.
    async fn find_by_username(&self, username: &str) -> RepoResult<Option<User>>;

    /// Find a user by their email address.
    async fn find_by_email(&self, email: &str) -> RepoResult<Option<User>>;

    /// List users with optional filtering and pagination.
    ///
    /// Used for admin user management.
    async fn list(&self, query: &AdminUsersQuery) -> RepoResult<Vec<UserSummary>>;

    /// Count users matching the given query.
    async fn count(&self, query: &AdminUsersQuery) -> RepoResult<i32>;

    /// List user profiles with optional filtering and pagination.
    ///
    /// Used for public profile listings (authors page).
    async fn list_profiles(
        &self,
        query: &ProfilesQuery,
        current_user_id: Option<Uuid>,
    ) -> RepoResult<Vec<UserProfile>>;

    /// Count profiles matching the given query.
    async fn count_profiles(&self, query: &ProfilesQuery) -> RepoResult<i32>;

    /// Get a user's profile with following status.
    async fn get_profile(
        &self,
        username: &str,
        current_user_id: Option<Uuid>,
    ) -> RepoResult<Option<UserProfile>>;

    // ========================================================================
    // Update Operations
    // ========================================================================

    /// Update a user's data.
    ///
    /// Only the fields that are `Some` will be updated.
    ///
    /// # Errors
    /// - `NotFound` if user doesn't exist
    /// - `AlreadyExists` if new username/email conflicts
    async fn update(&self, id: Uuid, data: &UpdateUser) -> RepoResult<User>;

    /// Update a user's password hash.
    ///
    /// Also updates the `password_changed_at` timestamp.
    async fn update_password(&self, id: Uuid, password_hash: &str) -> RepoResult<()>;

    /// Mark a user's email as verified.
    async fn verify_email(&self, id: Uuid) -> RepoResult<()>;

    /// Update a user's theme preference.
    async fn update_theme_preference(&self, id: Uuid, theme: &str) -> RepoResult<()>;

    // ========================================================================
    // Delete Operations
    // ========================================================================

    /// Delete a user by ID.
    ///
    /// This is a hard delete. Use `update` with `disabled: true` for soft delete.
    async fn delete(&self, id: Uuid) -> RepoResult<()>;

    // ========================================================================
    // Relationship Operations (Follow/Unfollow)
    // ========================================================================

    /// Follow a user.
    ///
    /// # Arguments
    /// * `follower_id` - The ID of the user who is following
    /// * `following_id` - The ID of the user being followed
    async fn follow(&self, follower_id: Uuid, following_id: Uuid) -> RepoResult<()>;

    /// Unfollow a user.
    async fn unfollow(&self, follower_id: Uuid, following_id: Uuid) -> RepoResult<()>;

    /// Check if a user is following another user.
    async fn is_following(&self, follower_id: Uuid, following_id: Uuid) -> RepoResult<bool>;

    /// Get the list of users that a user is following.
    async fn get_following(&self, user_id: Uuid) -> RepoResult<Vec<Uuid>>;

    /// Get the list of users following a user.
    async fn get_followers(&self, user_id: Uuid) -> RepoResult<Vec<Uuid>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_user_creation() {
        let new_user = NewUser {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash123".to_string(),
            role: Role::Subscriber,
        };

        assert_eq!(new_user.username, "testuser");
        assert_eq!(new_user.email, "test@example.com");
        assert_eq!(new_user.role, Role::Subscriber);
    }

    #[test]
    fn test_update_user_default() {
        let update = UpdateUser::default();

        assert!(update.username.is_none());
        assert!(update.email.is_none());
        assert!(update.bio.is_none());
        assert!(update.image.is_none());
        assert!(update.disabled.is_none());
        assert!(update.role.is_none());
        assert!(update.theme_preference.is_none());
    }
}
