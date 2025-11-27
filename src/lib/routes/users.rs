// src/lib/routes/users.rs

use crate::{
    ApiError, AppState,
    auth::{AuthenticatedUser, generate_token, hash_password, verify_password},
    models::{
        ProfileResponse, ProfilesQuery, ProfilesResponse, RegistrationSuccessResponse, Role,
        UserData, UserLogin, UserProfile, UserRegistration, UserResponse, UserUpdate,
    },
    response::ApiResponse,
};
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Json,
};
use chrono::{Duration, Utc};
use uuid::Uuid;
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

    let conn = state.db.connect().map_err(ApiError::from_connection_error)?;

    // Check if user already exists
    let mut existing_user = conn
        .query(
            "SELECT id FROM users WHERE email = ? OR username = ?",
            libsql::params![user_data.email.clone(), user_data.username.clone()],
        )
        .await?;

    if existing_user.next().await?.is_some() {
        return Err(ApiError::Conflict(
            "User with this email or username already exists".to_string(),
        ));
    }

    // Check if this is the first user (for admin assignment)
    let mut count_row = conn.query("SELECT COUNT(*) FROM users", ()).await?;

    let row = count_row
        .next()
        .await?
        .ok_or_else(|| ApiError::InternalServerError("Failed to count users".to_string()))?;

    let user_count: i64 = row.get(0)?;

    // First user gets admin role, all others get subscriber role
    let role = if user_count == 0 {
        Role::Admin
    } else {
        Role::Subscriber
    };

    let user_id = Uuid::new_v4();
    let password_hash = hash_password(&user_data.password)?;
    let now = Utc::now();

    // New users start with email_verified = 0
    conn.execute(
        "INSERT INTO users (id, username, email, password_hash, role, email_verified, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 0, ?, ?)",
        libsql::params![
            user_id.to_string(),
            user_data.username.clone(),
            user_data.email.clone(),
            password_hash,
            role.to_string(),
            0_i64,
            now.to_rfc3339(),
            now.to_rfc3339(),
        ],
    )
    .await?;

    // Generate email verification token
    let token_id = Uuid::new_v4();
    let verification_token = Uuid::new_v4().to_string();
    let expires_at = now + Duration::hours(24);

    conn.execute(
        "INSERT INTO email_verification_tokens (id, user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?, ?)",
        libsql::params![
            token_id.to_string(),
            user_id.to_string(),
            verification_token.clone(),
            expires_at.to_rfc3339(),
            now.to_rfc3339(),
        ],
    )
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

    let conn = state.db.connect().map_err(ApiError::from_connection_error)?;

    let mut rows = conn
        .query(
            "SELECT id, username, email, password_hash, bio, image, role, email_verified FROM users WHERE email = ?",
            libsql::params![login_data.email],
        )
        .await?;

    let row = rows
        .next()
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Invalid credentials".to_string()))?;

    let user_id: String = row.get(0)?;
    let username: String = row.get(1)?;
    let email: String = row.get(2)?;
    let password_hash: String = row.get(3)?;
    let bio: Option<String> = row.get(4).ok();
    let image: Option<String> = row.get(5).ok();
    let role_str: String = row.get(6)?;
    let email_verified: i64 = row.get(7)?;

    let role = role_str
        .parse::<Role>()
        .map_err(|_| ApiError::InternalServerError("Invalid role in database".to_string()))?;

    let user_uuid = Uuid::parse_str(&user_id)
        .map_err(|_| ApiError::InternalServerError("Invalid user ID".to_string()))?;

    if !verify_password(&login_data.password, &password_hash)? {
        return Err(ApiError::Unauthorized("Invalid credentials".to_string()));
    }

    // Check if email is verified
    if email_verified == 0 {
        return Err(ApiError::Forbidden(
            "Please verify your email address before logging in. Check your inbox for the verification link.".to_string(),
        ));
    }

    let token = generate_token(user_uuid, &state.jwt_keys)?;

    let response = UserResponse {
        user: UserData {
            email,
            token,
            username,
            bio,
            image,
            role,
        },
    };

    Ok(Json(response))
}

pub async fn get_current_user(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<UserResponse>, ApiError> {
    let conn = state.db.connect().map_err(ApiError::from_connection_error)?;

    let mut rows = conn
        .query(
            "SELECT username, email, bio, image, role FROM users WHERE id = ?",
            libsql::params![user.user_id.to_string()],
        )
        .await
        ?;

    let row = rows
        .next()
        .await
        ?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let username: String = row
        .get(0)
        ?;
    let email: String = row
        .get(1)
        ?;
    let bio: Option<String> = row.get(2).ok();
    let image: Option<String> = row.get(3).ok();
    let role_str: String = row
        .get(4)
        ?;

    let role = role_str
        .parse::<Role>()
        .map_err(|_| ApiError::InternalServerError("Invalid role in database".to_string()))?;

    let token = generate_token(user.user_id, &state.jwt_keys)?;

    let response = UserResponse {
        user: UserData {
            email,
            token,
            username,
            bio,
            image,
            role,
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

    let conn = state.db.connect().map_err(ApiError::from_connection_error)?;

    let now = Utc::now();
    let mut updates = Vec::new();
    let mut values: Vec<libsql::Value> = Vec::new();

    if let Some(username) = &update_data.username {
        updates.push("username = ?");
        values.push(username.clone().into());
    }

    if let Some(email) = &update_data.email {
        updates.push("email = ?");
        values.push(email.clone().into());
    }

    if let Some(password) = &update_data.password {
        let password_hash = hash_password(password)?;
        updates.push("password_hash = ?");
        values.push(password_hash.into());
    }

    if let Some(bio) = &update_data.bio {
        updates.push("bio = ?");
        values.push(bio.clone().into());
    }

    if let Some(image) = &update_data.image {
        updates.push("image = ?");
        values.push(image.clone().into());
    }

    updates.push("updated_at = ?");
    values.push(now.to_rfc3339().into());
    values.push(user.user_id.to_string().into());

    let query = format!("UPDATE users SET {} WHERE id = ?", updates.join(", "));

    conn.execute(&query, libsql::params_from_iter(values))
        .await
        ?;

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
    let conn = state.db.connect().map_err(ApiError::from_connection_error)?;

    let mut rows = conn
        .query(
            "SELECT id, username, bio, image FROM users WHERE username = ?",
            libsql::params![username],
        )
        .await
        ?;

    let row = rows
        .next()
        .await
        ?
        .ok_or_else(|| ApiError::NotFound("Profile not found".to_string()))?;

    let profile_id: String = row
        .get(0)
        ?;
    let profile_username: String = row
        .get(1)
        ?;
    let bio: Option<String> = row.get(2).ok();
    let image: Option<String> = row.get(3).ok();

    let profile_uuid = Uuid::parse_str(&profile_id)
        .map_err(|_| ApiError::InternalServerError("Invalid profile ID".to_string()))?;

    let following = if let Some(current_user) = user {
        let mut follow_rows = conn
            .query(
                "SELECT 1 FROM user_follows WHERE follower_id = ? AND following_id = ?",
                libsql::params![current_user.user_id.to_string(), profile_uuid.to_string(),],
            )
            .await
            ?;

        follow_rows
            .next()
            .await
            ?
            .is_some()
    } else {
        false
    };

    let response = ProfileResponse {
        profile: UserProfile {
            username: profile_username,
            bio,
            image,
            following,
        },
    };

    Ok(Json(response))
}

pub async fn follow_user(
    State(state): State<AppState>,
    Path(username): Path<String>,
    user: AuthenticatedUser,
) -> Result<Json<ProfileResponse>, ApiError> {
    let conn = state.db.connect().map_err(ApiError::from_connection_error)?;

    // Get the user to follow
    let mut rows = conn
        .query(
            "SELECT id FROM users WHERE username = ?",
            libsql::params![username.clone()],
        )
        .await
        ?;

    let row = rows
        .next()
        .await
        ?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let following_id: String = row
        .get(0)
        ?;
    let following_uuid = Uuid::parse_str(&following_id)
        .map_err(|_| ApiError::InternalServerError("Invalid user ID".to_string()))?;

    // Insert follow relationship (ignore if already exists)
    conn.execute(
        "INSERT OR IGNORE INTO user_follows (follower_id, following_id) VALUES (?, ?)",
        libsql::params![user.user_id.to_string(), following_uuid.to_string(),],
    )
    .await?;

    get_profile_internal(state, username, Some(user)).await
}

pub async fn unfollow_user(
    State(state): State<AppState>,
    Path(username): Path<String>,
    user: AuthenticatedUser,
) -> Result<Json<ProfileResponse>, ApiError> {
    let conn = state.db.connect().map_err(ApiError::from_connection_error)?;

    // Get the user to unfollow
    let mut rows = conn
        .query(
            "SELECT id FROM users WHERE username = ?",
            libsql::params![username.clone()],
        )
        .await
        ?;

    let row = rows
        .next()
        .await
        ?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let following_id: String = row
        .get(0)
        ?;
    let following_uuid = Uuid::parse_str(&following_id)
        .map_err(|_| ApiError::InternalServerError("Invalid user ID".to_string()))?;

    // Remove follow relationship
    conn.execute(
        "DELETE FROM user_follows WHERE follower_id = ? AND following_id = ?",
        libsql::params![user.user_id.to_string(), following_uuid.to_string(),],
    )
    .await?;

    get_profile_internal(state, username, Some(user)).await
}

pub async fn list_profiles(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<ProfilesQuery>,
) -> Result<Json<ProfilesResponse>, ApiError> {
    let conn = state.db.connect().map_err(ApiError::from_connection_error)?;

    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    // Build the query with optional search
    let (sql, params) = if let Some(search) = &query.search {
        let search_pattern = format!("%{}%", search);
        (
            "SELECT id, username, bio, image FROM users WHERE username LIKE ? OR bio LIKE ? ORDER BY username LIMIT ? OFFSET ?".to_string(),
            vec![
                libsql::Value::from(search_pattern.clone()),
                libsql::Value::from(search_pattern),
                libsql::Value::from(limit),
                libsql::Value::from(offset),
            ]
        )
    } else {
        (
            "SELECT id, username, bio, image FROM users ORDER BY username LIMIT ? OFFSET ?"
                .to_string(),
            vec![libsql::Value::from(limit), libsql::Value::from(offset)],
        )
    };

    let mut rows = conn
        .query(&sql, params)
        .await
        ?;

    let mut profiles = Vec::new();

    while let Some(row) = rows
        .next()
        .await
        ?
    {
        let profile_id: String = row
            .get(0)
            ?;
        let username: String = row
            .get(1)
            ?;
        let bio: Option<String> = row.get(2).ok();
        let image: Option<String> = row.get(3).ok();

        // Skip the current user from the results
        let profile_uuid = Uuid::parse_str(&profile_id)
            .map_err(|_| ApiError::InternalServerError("Invalid profile ID".to_string()))?;

        if profile_uuid == user.user_id {
            continue;
        }

        // Check if current user is following this profile
        let mut follow_rows = conn
            .query(
                "SELECT 1 FROM user_follows WHERE follower_id = ? AND following_id = ?",
                libsql::params![user.user_id.to_string(), profile_id],
            )
            .await
            ?;

        let following = follow_rows
            .next()
            .await
            ?
            .is_some();

        let profile = UserProfile {
            username,
            bio,
            image,
            following,
        };

        profiles.push(profile);
    }

    let profiles_count = profiles.len() as i32;

    let response = ProfilesResponse {
        profiles,
        profiles_count,
    };

    Ok(Json(response))
}
