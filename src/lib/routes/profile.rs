// src/lib/routes/profile.rs

// route handlers for profile pages

// dependencies
use crate::{
    auth::{AuthenticatedUser, OptionalUser},
    errors::ApiError,
    models::ProfilesQuery,
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
};
use axum_macros::debug_handler;
use chrono::Datelike;
use serde::Serialize;
use serde_json::json;

// struct type to represent a basic user profile for the template
#[derive(Debug, Serialize)]
struct ProfileData {
    username: String,
    bio: Option<String>,
    image: Option<String>,
    articles_count: i64,
    followers_count: i64,
    following_count: i64,
    following: bool,
}

// struct type to represent an article summary for the template
#[derive(Debug, Serialize)]
struct ArticleSummary {
    title: String,
    slug: String,
    description: String,
    created_at: String,
    tags: Vec<String>,
    favorites_count: i64,
    comments_count: i64,
    reading_time: i64,
}

// handler which renders the profile page template
#[debug_handler]
pub async fn get_profile_page(
    State(state): State<AppState>,
    Path(username): Path<String>,
    optional_user: OptionalUser,
) -> Result<impl IntoResponse, ApiError> {
    let conn = state.db.connect()?;

    // Fetch the profile user from the database
    let mut user_rows = conn
        .query(
            "SELECT id, username, bio, image FROM users WHERE username = ?",
            libsql::params![username.clone()],
        )
        .await?;

    let profile_row = user_rows
        .next()
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("User '{}' not found", username)))?;

    let profile_id: String = profile_row.get(0)?;
    let profile_username: String = profile_row.get(1)?;
    let profile_bio: Option<String> = profile_row.get(2).ok();
    let profile_image: Option<String> = profile_row.get(3).ok();

    // Count published articles by this user
    let mut articles_count_rows = conn
        .query(
            "SELECT COUNT(*) FROM articles WHERE author_id = ? AND draft = 0",
            libsql::params![profile_id.clone()],
        )
        .await?;
    let articles_count: i64 = articles_count_rows
        .next()
        .await?
        .map(|row| row.get(0).unwrap_or(0))
        .unwrap_or(0);

    // Count followers (people following this user)
    let mut followers_count_rows = conn
        .query(
            "SELECT COUNT(*) FROM user_follows WHERE following_id = ?",
            libsql::params![profile_id.clone()],
        )
        .await?;
    let followers_count: i64 = followers_count_rows
        .next()
        .await?
        .map(|row| row.get(0).unwrap_or(0))
        .unwrap_or(0);

    // Count following (people this user follows)
    let mut following_count_rows = conn
        .query(
            "SELECT COUNT(*) FROM user_follows WHERE follower_id = ?",
            libsql::params![profile_id.clone()],
        )
        .await?;
    let following_count: i64 = following_count_rows
        .next()
        .await?
        .map(|row| row.get(0).unwrap_or(0))
        .unwrap_or(0);

    // Check if current user is following this profile
    let mut is_following = false;
    let mut current_user_info: Option<serde_json::Value> = None;

    if let Some(ref auth_user) = optional_user.user {
        // Get current user info for navbar
        let mut current_user_rows = conn
            .query(
                "SELECT username, email, bio, image FROM users WHERE id = ?",
                libsql::params![auth_user.user_id.to_string()],
            )
            .await?;

        if let Some(row) = current_user_rows.next().await? {
            let cu_username: String = row.get(0)?;
            let cu_email: String = row.get(1)?;
            let cu_bio: Option<String> = row.get(2).ok();
            let cu_image: Option<String> = row.get(3).ok();

            current_user_info = Some(json!({
                "username": cu_username,
                "email": cu_email,
                "bio": cu_bio,
                "image": cu_image
            }));
        }

        // Check if following
        let mut follow_rows = conn
            .query(
                "SELECT 1 FROM user_follows WHERE follower_id = ? AND following_id = ?",
                libsql::params![auth_user.user_id.to_string(), profile_id.clone()],
            )
            .await?;
        is_following = follow_rows.next().await?.is_some();
    }

    // Fetch published articles by this user
    let mut article_rows = conn
        .query(
            r#"
            SELECT
                a.title,
                a.slug,
                a.description,
                a.created_at,
                a.id,
                COALESCE(a.reading_time, 5) as reading_time
            FROM articles a
            WHERE a.author_id = ? AND a.draft = 0
            ORDER BY a.created_at DESC
            LIMIT 20
            "#,
            libsql::params![profile_id.clone()],
        )
        .await?;

    let mut articles: Vec<ArticleSummary> = Vec::new();
    while let Some(row) = article_rows.next().await? {
        let title: String = row.get(0)?;
        let slug: String = row.get(1)?;
        let description: String = row.get(2).unwrap_or_default();
        let created_at: String = row.get(3)?;
        let article_id: String = row.get(4)?;
        let reading_time: i64 = row.get(5).unwrap_or(5);

        // Get tags for this article
        let mut tag_rows = conn
            .query(
                r#"
                SELECT t.name
                FROM tags t
                JOIN article_tags at ON t.id = at.tag_id
                WHERE at.article_id = ?
                "#,
                libsql::params![article_id.clone()],
            )
            .await?;

        let mut tags: Vec<String> = Vec::new();
        while let Some(tag_row) = tag_rows.next().await? {
            let tag_name: String = tag_row.get(0)?;
            tags.push(tag_name);
        }

        // Count favorites for this article
        let mut fav_rows = conn
            .query(
                "SELECT COUNT(*) FROM user_favorites WHERE article_id = ?",
                libsql::params![article_id.clone()],
            )
            .await?;
        let favorites_count: i64 = fav_rows
            .next()
            .await?
            .map(|r| r.get(0).unwrap_or(0))
            .unwrap_or(0);

        // Count comments for this article
        let mut comment_rows = conn
            .query(
                "SELECT COUNT(*) FROM comments WHERE article_id = ?",
                libsql::params![article_id.clone()],
            )
            .await?;
        let comments_count: i64 = comment_rows
            .next()
            .await?
            .map(|r| r.get(0).unwrap_or(0))
            .unwrap_or(0);

        articles.push(ArticleSummary {
            title,
            slug,
            description,
            created_at,
            tags,
            favorites_count,
            comments_count,
            reading_time,
        });
    }

    // Build profile data
    let profile = ProfileData {
        username: profile_username.clone(),
        bio: profile_bio,
        image: profile_image,
        articles_count,
        followers_count,
        following_count,
        following: is_following,
    };

    // Build template context
    let context = json!({
        "title": format!("{}'s Profile", profile_username),
        "profile": profile,
        "articles": articles,
        "favorites": Vec::<ArticleSummary>::new(),
        "drafts": Vec::<ArticleSummary>::new(),
        "current_user": current_user_info,
        "user": current_user_info,
        "current_page": 1,
        "total_pages": 1,
        "current_year": chrono::Utc::now().year(),
    });

    let html = state
        .templates
        .render(
            "profile/profile.html",
            &tera::Context::from_serialize(&context)?,
        )
        .map_err(|e| ApiError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}

/// Redirect authenticated users to their own profile page
/// GET /profile
pub async fn get_my_profile_page(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let conn = state.db.connect()?;

    // Get the current user's username
    let mut rows = conn
        .query(
            "SELECT username FROM users WHERE id = ?",
            libsql::params![user.user_id.to_string()],
        )
        .await?;

    let row = rows
        .next()
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let username: String = row.get(0)?;

    // Redirect to the user's profile page
    Ok(axum::response::Redirect::to(&format!("/profiles/{}", username)))
}

pub async fn get_my_favorites_page(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    // Get user info for template context
    let conn = state
        .db
        .connect()
        .map_err(ApiError::from_connection_error)?;

    let mut user_rows = conn
        .query(
            "SELECT username, email, bio, image FROM users WHERE id = ?",
            libsql::params![user.user_id.to_string()],
        )
        .await?;

    let user_info = if let Some(row) = user_rows.next().await? {
        let username: String = row.get(0)?;
        let email: String = row.get(1)?;
        let bio: Option<String> = row.get(2).ok();
        let image: Option<String> = row.get(3).ok();

        Some(serde_json::json!({
            "username": username.clone(),
            "email": email,
            "bio": bio,
            "image": image
        }))
    } else {
        None
    };

    // Get the user's favorited articles using the existing list_articles function
    use crate::models::ArticleQuery;
    use axum::extract::Query;

    let username = user_info.as_ref().unwrap()["username"].as_str().unwrap();
    let favorites_query = ArticleQuery {
        tag: None,
        author: None,
        favorited: Some(username.to_string()),
        category: None,
        limit: Some(50), // Show up to 50 favorites
        offset: Some(0),
    };

    let articles =
        match crate::routes::list_articles(State(state.clone()), Query(favorites_query)).await {
            Ok(articles_response) => articles_response.0.articles,
            Err(_) => vec![], // If there's an error fetching articles, show empty list
        };

    let context: serde_json::Value = serde_json::json!({
        "title": "My Favorites - CrustyRustacean Dev Blog",
        "page": "Favorites",
        "current_year": chrono::Utc::now().year(),
        "user": user_info,
        "articles": articles
    });

    let html = state
        .templates
        .render(
            "profile/favorites.html",
            &tera::Context::from_serialize(&context)?,
        )
        .map_err(|e| ApiError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}

pub async fn get_authors_page(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<ProfilesQuery>,
) -> Result<impl IntoResponse, ApiError> {
    // Get user info for template context
    let conn = state
        .db
        .connect()
        .map_err(ApiError::from_connection_error)?;

    let mut user_rows = conn
        .query(
            "SELECT username, email, bio, image FROM users WHERE id = ?",
            libsql::params![user.user_id.to_string()],
        )
        .await?;

    let user_info = if let Some(row) = user_rows.next().await? {
        let username: String = row.get(0)?;
        let email: String = row.get(1)?;
        let bio: Option<String> = row.get(2).ok();
        let image: Option<String> = row.get(3).ok();

        Some(serde_json::json!({
            "username": username,
            "email": email,
            "bio": bio,
            "image": image
        }))
    } else {
        None
    };

    // Get profiles using the existing list_profiles function
    let profiles_response =
        crate::routes::list_profiles(State(state.clone()), user, Query(query.clone())).await?;
    let profiles = profiles_response.0.profiles;

    // Convert profiles to JSON for template
    let profiles_json: Vec<serde_json::Value> = profiles
        .iter()
        .map(|profile| {
            serde_json::json!({
                "username": profile.username,
                "bio": profile.bio,
                "image": profile.image,
                "following": profile.following
            })
        })
        .collect();

    let context: serde_json::Value = serde_json::json!({
        "title": "Find Authors - CrustyRustacean Dev Blog",
        "page": "Authors",
        "current_year": chrono::Utc::now().year(),
        "user": user_info,
        "profiles": profiles_json,
        "search_query": query.search
    });

    let html = state
        .templates
        .render(
            "profile/authors.html",
            &tera::Context::from_serialize(&context)?,
        )
        .map_err(|e| ApiError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}
