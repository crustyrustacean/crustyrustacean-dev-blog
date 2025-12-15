// src/lib/routes/users.rs

use crate::{
    ApiError, AppState,
    auth::{AuthenticatedUser, generate_token, hash_password, verify_password},
    models::{
        AdminUsersQuery, ProfileResponse, ProfilesQuery, ProfilesResponse,
        RegistrationSuccessResponse, Role, ThemePreferenceResponse, ThemePreferenceUpdate,
        UserData, UserLogin, UserRegistration, UserResponse, UserUpdate,
    },
    repositories::{NewUser, UpdateUser},
    response::ApiResponse,
    theme::ThemeListItem,
};
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Json,
};
use chrono::{Duration, Utc};
use validator::Validate;

pub async fn register_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Result<ApiResponse<RegistrationSuccessResponse>, ApiError> {
    let user_data: UserRegistration = serde_json::from_value(
        payload
            .get("user")
            .ok_or_else(|| ApiError::BadRequest("Missing user field".to_string()))?
            .clone(),
    )
    .map_err(|_| ApiError::BadRequest("Invalid user data".to_string()))?;

    user_data
        .validate()
        .map_err(|e| ApiError::BadRequest(format!("Validation error: {}", e)))?;

    // Check if user already exists (by email or username)
    if state.users.find_by_email(&user_data.email).await?.is_some() {
        return Err(ApiError::Conflict(
            "User with this email already exists".to_string(),
        ));
    }

    if state
        .users
        .find_by_username(&user_data.username)
        .await?
        .is_some()
    {
        return Err(ApiError::Conflict(
            "User with this username already exists".to_string(),
        ));
    }

    // Check if this is the first user (for admin assignment)
    let user_count = state
        .users
        .count(&AdminUsersQuery {
            search: None,
            status: None,
            limit: None,
            offset: None,
        })
        .await?;

    // First user gets admin role, all others get subscriber role
    let role = if user_count == 0 {
        Role::Admin
    } else {
        Role::Subscriber
    };

    let password_hash = hash_password(&user_data.password)?;

    let new_user = NewUser {
        username: user_data.username.clone(),
        email: user_data.email.clone(),
        password_hash,
        role,
    };

    let user = state.users.create(&new_user).await?;

    // Generate email verification token
    let verification_token = uuid::Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(24);

    state
        .tokens
        .create_email_verification_token(user.id, &verification_token, expires_at)
        .await?;

    // Send verification email
    // Get host from headers
    let host = headers
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost:8000");

    // Determine protocol based on host
    let protocol = if host.contains("localhost") || host.contains("127.0.0.1") {
        "http"
    } else {
        "https"
    };
    let base_url = format!("{}://{}", protocol, host);

    // Send verification email - registration fails if email cannot be sent
    state
        .email
        .send_email_verification(
            &user_data.email,
            &user_data.username,
            &verification_token,
            &base_url,
        )
        .await?;

    let response = RegistrationSuccessResponse {
        message: "Registration successful! Please check your email to verify your account."
            .to_string(),
        email: user_data.email,
    };

    Ok(ApiResponse::success(response))
}

pub async fn login_user(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<UserResponse>, ApiError> {
    let login_data: UserLogin = serde_json::from_value(
        payload
            .get("user")
            .ok_or_else(|| ApiError::BadRequest("Missing user field".to_string()))?
            .clone(),
    )
    .map_err(|_| ApiError::BadRequest("Invalid login data".to_string()))?;

    login_data
        .validate()
        .map_err(|e| ApiError::BadRequest(format!("Validation error: {}", e)))?;

    let user = state
        .users
        .find_by_email(&login_data.email)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Invalid credentials".to_string()))?;

    if !verify_password(&login_data.password, &user.password_hash)? {
        return Err(ApiError::Unauthorized("Invalid credentials".to_string()));
    }

    // Check if email is verified
    if !user.email_verified {
        return Err(ApiError::Forbidden(
            "Please verify your email address before logging in. Check your inbox for the verification link.".to_string(),
        ));
    }

    let token = generate_token(user.id, &state.jwt_keys)?;

    let response = UserResponse {
        user: UserData {
            email: user.email,
            token,
            username: user.username,
            bio: user.bio,
            image: user.image,
            role: user.role,
        },
    };

    Ok(Json(response))
}

pub async fn get_current_user(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<UserResponse>, ApiError> {
    let db_user = state
        .users
        .find_by_id(user.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let token = generate_token(user.user_id, &state.jwt_keys)?;

    let response = UserResponse {
        user: UserData {
            email: db_user.email,
            token,
            username: db_user.username,
            bio: db_user.bio,
            image: db_user.image,
            role: db_user.role,
        },
    };

    Ok(Json(response))
}

pub async fn update_current_user(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<UserResponse>, ApiError> {
    let update_data: UserUpdate = serde_json::from_value(
        payload
            .get("user")
            .ok_or_else(|| ApiError::BadRequest("Missing user field".to_string()))?
            .clone(),
    )
    .map_err(|_| ApiError::BadRequest("Invalid update data".to_string()))?;

    update_data
        .validate()
        .map_err(|e| ApiError::BadRequest(format!("Validation error: {}", e)))?;

    // Handle password update separately if provided
    if let Some(password) = &update_data.password {
        let password_hash = hash_password(password)?;
        state
            .users
            .update_password(user.user_id, &password_hash)
            .await?;
    }

    // Update other fields
    let repo_update = UpdateUser {
        username: update_data.username,
        email: update_data.email,
        bio: update_data.bio,
        image: update_data.image,
        ..Default::default()
    };

    state.users.update(user.user_id, &repo_update).await?;

    // Fetch updated user
    get_current_user(State(state), user).await
}

pub async fn get_profile(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<ProfileResponse>, ApiError> {
    get_profile_internal(state, username, None).await
}

pub async fn get_profile_authenticated(
    State(state): State<AppState>,
    Path(username): Path<String>,
    user: AuthenticatedUser,
) -> Result<Json<ProfileResponse>, ApiError> {
    get_profile_internal(state, username, Some(user)).await
}

async fn get_profile_internal(
    state: AppState,
    username: String,
    user: Option<AuthenticatedUser>,
) -> Result<Json<ProfileResponse>, ApiError> {
    let current_user_id = user.map(|u| u.user_id);

    let profile = state
        .users
        .get_profile(&username, current_user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Profile not found".to_string()))?;

    let response = ProfileResponse { profile };

    Ok(Json(response))
}

pub async fn follow_user(
    State(state): State<AppState>,
    Path(username): Path<String>,
    user: AuthenticatedUser,
) -> Result<Json<ProfileResponse>, ApiError> {
    // Get the user to follow
    let target_user = state
        .users
        .find_by_username(&username)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    // Follow the user
    state.users.follow(user.user_id, target_user.id).await?;

    get_profile_internal(state, username, Some(user)).await
}

pub async fn unfollow_user(
    State(state): State<AppState>,
    Path(username): Path<String>,
    user: AuthenticatedUser,
) -> Result<Json<ProfileResponse>, ApiError> {
    // Get the user to unfollow
    let target_user = state
        .users
        .find_by_username(&username)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    // Unfollow the user
    state.users.unfollow(user.user_id, target_user.id).await?;

    get_profile_internal(state, username, Some(user)).await
}

pub async fn list_profiles(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<ProfilesQuery>,
) -> Result<Json<ProfilesResponse>, ApiError> {
    let profiles = state
        .users
        .list_profiles(&query, Some(user.user_id))
        .await?;

    let profiles_count = profiles.len() as i32;

    let response = ProfilesResponse {
        profiles,
        profiles_count,
    };

    Ok(Json(response))
}

// ==========================================
// Theme Preference Routes
// ==========================================

/// Get user's theme preference
/// GET /api/user/theme
pub async fn get_theme_preference(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<ApiResponse<ThemePreferenceResponse>, ApiError> {
    let db_user = state
        .users
        .find_by_id(user.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let theme = db_user.theme_preference.unwrap_or_else(|| "auto".to_string());

    Ok(ApiResponse::success(ThemePreferenceResponse { theme }))
}

/// Update user's theme preference
/// PUT /api/user/theme
pub async fn update_theme_preference(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<ThemePreferenceUpdate>,
) -> Result<ApiResponse<ThemePreferenceResponse>, ApiError> {
    payload
        .validate()
        .map_err(|e| ApiError::BadRequest(format!("Validation error: {}", e)))?;

    // Validate that the theme exists
    let valid_themes = ["auto", "default", "dark", "high-contrast"];
    if !valid_themes.contains(&payload.theme.as_str()) && !state.themes.contains(&payload.theme) {
        return Err(ApiError::BadRequest(format!(
            "Invalid theme: {}. Valid themes are: {:?}",
            payload.theme, valid_themes
        )));
    }

    state
        .users
        .update_theme_preference(user.user_id, &payload.theme)
        .await?;

    Ok(ApiResponse::success(ThemePreferenceResponse {
        theme: payload.theme,
    }))
}

/// Get list of available themes
/// GET /api/themes
pub async fn list_themes(
    State(state): State<AppState>,
) -> Result<ApiResponse<Vec<ThemeListItem>>, ApiError> {
    let mut themes = state.available_themes();

    // Add 'auto' option
    themes.insert(
        0,
        ThemeListItem {
            id: "auto".to_string(),
            name: "System".to_string(),
            description: Some("Follow system preference".to_string()),
            color_scheme: "auto".to_string(),
            high_contrast: false,
            preview_colors: vec!["#667eea".to_string(), "#764ba2".to_string()],
        },
    );

    Ok(ApiResponse::success(themes))
}
