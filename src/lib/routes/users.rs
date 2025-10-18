// src/lib/routes/users.rs

use crate::{
    AppError, AppState,
    auth::{AuthenticatedUser, generate_token, hash_password, verify_password},
    models::{
        ProfileResponse, ProfilesQuery, ProfilesResponse, UserData, UserLogin, UserProfile, 
        UserRegistration, UserResponse, UserUpdate,
    },
};
use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use chrono::Utc;
use uuid::Uuid;
use validator::Validate;

pub async fn register_user(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<UserResponse>, AppError> {
    let user_data: UserRegistration = serde_json::from_value(
        payload
            .get("user")
            .ok_or_else(|| AppError::BadRequest("Missing user field".to_string()))?
            .clone(),
    )
    .map_err(|_| AppError::BadRequest("Invalid user data".to_string()))?;

    user_data
        .validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Check if user already exists
    let mut existing_user = conn
        .query(
            "SELECT id FROM users WHERE email = ? OR username = ?",
            libsql::params![user_data.email.clone(), user_data.username.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    if existing_user
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .is_some()
    {
        return Err(AppError::Conflict(
            "User with this email or username already exists".to_string(),
        ));
    }

    let user_id = Uuid::new_v4();
    let password_hash = hash_password(&user_data.password)?;
    let now = Utc::now();

    conn.execute(
        "INSERT INTO users (id, username, email, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        libsql::params![
            user_id.to_string(),
            user_data.username.clone(),
            user_data.email.clone(),
            password_hash,
            now.to_rfc3339(),
            now.to_rfc3339(),
        ],
    )
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let token = generate_token(user_id, &state.jwt_keys)?;

    let response = UserResponse {
        user: UserData {
            email: user_data.email,
            token,
            username: user_data.username,
            bio: None,
            image: None,
        },
    };

    Ok(Json(response))
}

pub async fn login_user(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<UserResponse>, AppError> {
    let login_data: UserLogin = serde_json::from_value(
        payload
            .get("user")
            .ok_or_else(|| AppError::BadRequest("Missing user field".to_string()))?
            .clone(),
    )
    .map_err(|_| AppError::BadRequest("Invalid login data".to_string()))?;

    login_data
        .validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let mut rows = conn
        .query(
            "SELECT id, username, email, password_hash, bio, image FROM users WHERE email = ?",
            libsql::params![login_data.email],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let row = rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    let user_id: String = row
        .get(0)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let username: String = row
        .get(1)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let email: String = row
        .get(2)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let password_hash: String = row
        .get(3)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let bio: Option<String> = row.get(4).ok();
    let image: Option<String> = row.get(5).ok();

    let user_uuid = Uuid::parse_str(&user_id)
        .map_err(|_| AppError::InternalServerError("Invalid user ID".to_string()))?;

    if !verify_password(&login_data.password, &password_hash)? {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    let token = generate_token(user_uuid, &state.jwt_keys)?;

    let response = UserResponse {
        user: UserData {
            email,
            token,
            username,
            bio,
            image,
        },
    };

    Ok(Json(response))
}

pub async fn get_current_user(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<UserResponse>, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let mut rows = conn
        .query(
            "SELECT username, email, bio, image FROM users WHERE id = ?",
            libsql::params![user.user_id.to_string()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let row = rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let username: String = row
        .get(0)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let email: String = row
        .get(1)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let bio: Option<String> = row.get(2).ok();
    let image: Option<String> = row.get(3).ok();

    let token = generate_token(user.user_id, &state.jwt_keys)?;

    let response = UserResponse {
        user: UserData {
            email,
            token,
            username,
            bio,
            image,
        },
    };

    Ok(Json(response))
}

pub async fn update_current_user(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<UserResponse>, AppError> {
    let update_data: UserUpdate = serde_json::from_value(
        payload
            .get("user")
            .ok_or_else(|| AppError::BadRequest("Missing user field".to_string()))?
            .clone(),
    )
    .map_err(|_| AppError::BadRequest("Invalid update data".to_string()))?;

    update_data
        .validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

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
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Fetch updated user
    get_current_user(State(state), user).await
}

pub async fn get_profile(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<ProfileResponse>, AppError> {
    get_profile_internal(state, username, None).await
}

pub async fn get_profile_authenticated(
    State(state): State<AppState>,
    Path(username): Path<String>,
    user: AuthenticatedUser,
) -> Result<Json<ProfileResponse>, AppError> {
    get_profile_internal(state, username, Some(user)).await
}

async fn get_profile_internal(
    state: AppState,
    username: String,
    user: Option<AuthenticatedUser>,
) -> Result<Json<ProfileResponse>, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let mut rows = conn
        .query(
            "SELECT id, username, bio, image FROM users WHERE username = ?",
            libsql::params![username],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let row = rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Profile not found".to_string()))?;

    let profile_id: String = row
        .get(0)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let profile_username: String = row
        .get(1)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let bio: Option<String> = row.get(2).ok();
    let image: Option<String> = row.get(3).ok();

    let profile_uuid = Uuid::parse_str(&profile_id)
        .map_err(|_| AppError::InternalServerError("Invalid profile ID".to_string()))?;

    let following = if let Some(current_user) = user {
        let mut follow_rows = conn
            .query(
                "SELECT 1 FROM user_follows WHERE follower_id = ? AND following_id = ?",
                libsql::params![current_user.user_id.to_string(), profile_uuid.to_string(),],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        follow_rows
            .next()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
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
) -> Result<Json<ProfileResponse>, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Get the user to follow
    let mut rows = conn
        .query(
            "SELECT id FROM users WHERE username = ?",
            libsql::params![username.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let row = rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let following_id: String = row
        .get(0)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let following_uuid = Uuid::parse_str(&following_id)
        .map_err(|_| AppError::InternalServerError("Invalid user ID".to_string()))?;

    // Insert follow relationship (ignore if already exists)
    conn.execute(
        "INSERT OR IGNORE INTO user_follows (follower_id, following_id) VALUES (?, ?)",
        libsql::params![user.user_id.to_string(), following_uuid.to_string(),],
    )
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    get_profile_internal(state, username, Some(user)).await
}

pub async fn unfollow_user(
    State(state): State<AppState>,
    Path(username): Path<String>,
    user: AuthenticatedUser,
) -> Result<Json<ProfileResponse>, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Get the user to unfollow
    let mut rows = conn
        .query(
            "SELECT id FROM users WHERE username = ?",
            libsql::params![username.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let row = rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let following_id: String = row
        .get(0)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let following_uuid = Uuid::parse_str(&following_id)
        .map_err(|_| AppError::InternalServerError("Invalid user ID".to_string()))?;

    // Remove follow relationship
    conn.execute(
        "DELETE FROM user_follows WHERE follower_id = ? AND following_id = ?",
        libsql::params![user.user_id.to_string(), following_uuid.to_string(),],
    )
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    get_profile_internal(state, username, Some(user)).await
}

pub async fn list_profiles(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<ProfilesQuery>,
) -> Result<Json<ProfilesResponse>, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

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
            "SELECT id, username, bio, image FROM users ORDER BY username LIMIT ? OFFSET ?".to_string(),
            vec![
                libsql::Value::from(limit),
                libsql::Value::from(offset),
            ]
        )
    };

    let mut rows = conn
        .query(&sql, params)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let mut profiles = Vec::new();

    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        let profile_id: String = row
            .get(0)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let username: String = row
            .get(1)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let bio: Option<String> = row.get(2).ok();
        let image: Option<String> = row.get(3).ok();

        // Skip the current user from the results
        let profile_uuid = Uuid::parse_str(&profile_id)
            .map_err(|_| AppError::InternalServerError("Invalid profile ID".to_string()))?;
        
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
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let following = follow_rows
            .next()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
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
