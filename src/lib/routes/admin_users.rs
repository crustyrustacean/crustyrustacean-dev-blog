// src/lib/routes/admin_users.rs

use crate::{
    auth::{AdminUser, AuthenticatedUser},
    errors::AppError,
    models::{
        AdminUserData, AdminUserResponse, AdminUserUpdate, AdminUsersQuery, AdminUsersResponse,
        Role,
    },
    response::ApiResponse,
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
};
use chrono::Datelike;
use validator::Validate;

/// Admin Users Page
/// GET /admin/users
pub async fn get_admin_users_page(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Fetch basic user count for display
    let mut count_rows = conn
        .query("SELECT COUNT(*) FROM users", ())
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let user_count: i32 = if let Some(row) = count_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        row.get(0).unwrap_or(0)
    } else {
        0
    };

    // Get user info for template
    let mut user_rows = conn
        .query(
            "SELECT username, email, bio, image FROM users WHERE id = ?",
            libsql::params![user.user_id.to_string()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let user_info = if let Some(row) = user_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        let username: String = row
            .get(0)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let email: String = row
            .get(1)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let bio: Option<String> = row.get(2).ok();
        let image: Option<String> = row.get(3).ok();

        serde_json::json!({
            "username": username,
            "email": email,
            "bio": bio,
            "image": image,
        })
    } else {
        serde_json::json!(null)
    };

    let html = state
        .templates
        .render(
            "admin/users.html",
            &tera::Context::from_serialize(serde_json::json!({
                "user": user_info,
                "user_count": user_count,
                "current_year": chrono::Utc::now().year(),
            }))
            .map_err(|e| AppError::InternalServerError(e.to_string()))?,
        )
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(Html(html))
}

/// List Users (Admin)
/// GET /api/admin/users?search=query&status=active&limit=20&offset=0
pub async fn list_users_admin(
    State(state): State<AppState>,
    _user: AdminUser, // Require admin authentication
    Query(query): Query<AdminUsersQuery>,
) -> Result<Json<ApiResponse<AdminUsersResponse>>, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

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

    // Fetch users with article count
    let query_str = format!(
        "SELECT u.id, u.username, u.email, u.bio, u.image, u.disabled, u.role, u.created_at,
                COUNT(a.id) as article_count
         FROM users u
         LEFT JOIN articles a ON u.id = a.author_id
         {}
         GROUP BY u.id
         ORDER BY u.created_at DESC
         LIMIT ? OFFSET ?",
        where_clause
    );

    let param_count = params_vec.len();
    params_vec.push(limit.to_string());
    params_vec.push(offset.to_string());

    let params: Vec<libsql::Value> = params_vec
        .iter()
        .map(|s| libsql::Value::Text(s.clone()))
        .collect();

    let mut rows = conn
        .query(&query_str, libsql::params_from_iter(params.clone()))
        .await
        .map_err(|e| {
            tracing::error!("Failed to query users: {:?}", e);
            AppError::InternalServerError(e.to_string())
        })?;

    let mut users = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        let disabled_int: i64 = row.get(5).unwrap_or(0);
        let role_str: String = row.get(6).unwrap_or_else(|_| "subscriber".to_string());
        let role = role_str.parse::<Role>().unwrap_or(Role::Subscriber);

        users.push(AdminUserData {
            id: row.get(0).unwrap(),
            username: row.get(1).unwrap(),
            email: row.get(2).unwrap(),
            bio: row.get(3).ok(),
            image: row.get(4).ok(),
            disabled: disabled_int != 0,
            role,
            created_at: row.get(7).unwrap(),
            article_count: row.get::<i32>(8).unwrap_or(0),
        });
    }

    // Get total count
    let count_query = format!("SELECT COUNT(DISTINCT u.id) FROM users u {}", where_clause);

    let count_params: Vec<libsql::Value> = params_vec
        .iter()
        .take(param_count)
        .map(|s| libsql::Value::Text(s.clone()))
        .collect();

    let mut count_rows = conn
        .query(&count_query, libsql::params_from_iter(count_params))
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let users_count: i32 = if let Some(row) = count_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        row.get(0).unwrap_or(0)
    } else {
        0
    };

    Ok(Json(ApiResponse::success(AdminUsersResponse {
        users,
        users_count,
    })))
}

/// Get Single User (Admin)
/// GET /api/admin/users/:id
pub async fn get_user_admin(
    State(state): State<AppState>,
    _user: AdminUser,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<AdminUserResponse>>, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let mut rows = conn
        .query(
            "SELECT u.id, u.username, u.email, u.bio, u.image, u.disabled, u.role, u.created_at,
                    COUNT(a.id) as article_count
             FROM users u
             LEFT JOIN articles a ON u.id = a.author_id
             WHERE u.id = ?
             GROUP BY u.id",
            libsql::params![id.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    if let Some(row) = rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        let disabled_int: i64 = row.get(5).unwrap_or(0);
        let role_str: String = row.get(6).unwrap_or_else(|_| "subscriber".to_string());
        let role = role_str.parse::<Role>().unwrap_or(Role::Subscriber);

        let user_data = AdminUserData {
            id: row.get(0).unwrap(),
            username: row.get(1).unwrap(),
            email: row.get(2).unwrap(),
            bio: row.get(3).ok(),
            image: row.get(4).ok(),
            disabled: disabled_int != 0,
            role,
            created_at: row.get(7).unwrap(),
            article_count: row.get::<i32>(8).unwrap_or(0),
        };

        Ok(Json(ApiResponse::success(AdminUserResponse {
            user: user_data,
        })))
    } else {
        Err(AppError::NotFound("User not found".to_string()))
    }
}

/// Update User (Admin)
/// PUT /api/admin/users/:id
pub async fn update_user_admin(
    State(state): State<AppState>,
    _user: AdminUser,
    Path(id): Path<String>,
    Json(payload): Json<AdminUserUpdate>,
) -> Result<Json<ApiResponse<AdminUserResponse>>, AppError> {
    // Validate input
    payload.validate().map_err(|e| {
        let error_messages: Vec<String> = e
            .field_errors()
            .iter()
            .flat_map(|(_, errors)| {
                errors
                    .iter()
                    .map(|error| error.message.clone().unwrap_or_default().to_string())
            })
            .collect();
        AppError::UnprocessableEntity(error_messages.join(", "))
    })?;

    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Check if user exists
    let mut check_rows = conn
        .query(
            "SELECT id FROM users WHERE id = ?",
            libsql::params![id.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    if check_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .is_none()
    {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    // Build update query dynamically
    let mut updates = Vec::new();
    let mut params: Vec<libsql::Value> = Vec::new();

    if let Some(username) = &payload.username {
        updates.push("username = ?");
        params.push(libsql::Value::Text(username.clone()));
    }
    if let Some(email) = &payload.email {
        updates.push("email = ?");
        params.push(libsql::Value::Text(email.clone()));
    }
    if let Some(bio) = &payload.bio {
        updates.push("bio = ?");
        params.push(libsql::Value::Text(bio.clone()));
    }
    if let Some(image) = &payload.image {
        updates.push("image = ?");
        params.push(libsql::Value::Text(image.clone()));
    }
    if let Some(disabled) = payload.disabled {
        updates.push("disabled = ?");
        params.push(libsql::Value::Integer(if disabled { 1 } else { 0 }));
    }
    if let Some(role) = &payload.role {
        updates.push("role = ?");
        params.push(libsql::Value::Text(role.to_string()));
    }

    if updates.is_empty() {
        return Err(AppError::BadRequest("No fields to update".to_string()));
    }

    updates.push("updated_at = datetime('now')");

    params.push(libsql::Value::Text(id.clone()));

    let update_query = format!("UPDATE users SET {} WHERE id = ?", updates.join(", "));

    conn.execute(&update_query, libsql::params_from_iter(params))
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                if e.to_string().contains("username") {
                    AppError::Conflict("Username already exists".to_string())
                } else if e.to_string().contains("email") {
                    AppError::Conflict("Email already exists".to_string())
                } else {
                    AppError::Conflict("Unique constraint violation".to_string())
                }
            } else {
                AppError::InternalServerError(e.to_string())
            }
        })?;

    // Fetch and return updated user
    get_user_admin(State(state), _user, Path(id)).await
}

/// Delete User (Admin)
/// DELETE /api/admin/users/:id
pub async fn delete_user_admin(
    State(state): State<AppState>,
    current_user: AdminUser,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    // Prevent admin from deleting themselves
    if current_user.user_id.to_string() == id {
        return Err(AppError::BadRequest(
            "Cannot delete your own account".to_string(),
        ));
    }

    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Check if user exists
    let mut check_rows = conn
        .query(
            "SELECT id FROM users WHERE id = ?",
            libsql::params![id.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    if check_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .is_none()
    {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    // Delete user (CASCADE will handle related records)
    conn.execute(
        "DELETE FROM users WHERE id = ?",
        libsql::params![id.clone()],
    )
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "message": "User deleted successfully"
    }))))
}
