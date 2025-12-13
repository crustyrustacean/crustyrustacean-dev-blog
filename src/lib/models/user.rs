// src/lib/models/user.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use super::Role;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub disabled: bool,
    pub role: Role,
    pub email_verified: bool,
    pub theme_preference: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub following: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UserRegistration {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UserLogin {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UserUpdate {
    #[validate(length(min = 3, max = 50))]
    pub username: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 6))]
    pub password: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub user: UserData,
}

#[derive(Debug, Serialize)]
pub struct UserData {
    pub email: String,
    pub token: String,
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub role: Role,
}

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub profile: UserProfile,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilesResponse {
    pub profiles: Vec<UserProfile>,
    pub profiles_count: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProfilesQuery {
    pub search: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/// Response for successful registration (before email verification)
#[derive(Debug, Serialize)]
pub struct RegistrationSuccessResponse {
    pub message: String,
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct PasswordResetRequest {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct PasswordResetComplete {
    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 6))]
    pub current_password: String,
    #[validate(length(min = 6))]
    pub new_password: String,
}

// Admin user management models
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminUserData {
    pub id: String,
    pub username: String,
    pub email: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub disabled: bool,
    pub role: Role,
    pub email_verified: bool,
    pub created_at: String,
    pub article_count: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminUsersResponse {
    pub users: Vec<AdminUserData>,
    pub users_count: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminUserResponse {
    pub user: AdminUserData,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AdminUserUpdate {
    #[validate(length(min = 3, max = 50))]
    pub username: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub disabled: Option<bool>,
    pub role: Option<Role>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AdminUsersQuery {
    pub search: Option<String>,
    pub status: Option<String>, // "active", "disabled", or "all"
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

// Theme preference models
#[derive(Debug, Serialize)]
pub struct ThemePreferenceResponse {
    pub theme: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ThemePreferenceUpdate {
    #[validate(length(min = 1, max = 50))]
    pub theme: String,
}
