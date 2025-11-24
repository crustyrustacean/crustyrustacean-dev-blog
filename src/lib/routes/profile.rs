// src/lib/routes/profile.rs

// route handlers for profile pages

// dependencies
use crate::{auth::AuthenticatedUser, errors::ApiError, models::ProfilesQuery, state::AppState};
use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
};
use axum_macros::debug_handler;
use chrono::Datelike;
use serde::Serialize;

// struct type to represent a basic user profile
#[derive(Debug, Serialize)]
struct UserProfile {
    username: String,
    bio: Option<String>,
    image: Option<String>,
    articles_count: i32,
    followers_count: i32,
    following_count: i32,
    following: bool,
    website: Option<String>,
    twitter: Option<String>,
    github: Option<String>,
}

// struct type to represent an article summary
#[derive(Debug, Serialize)]
struct ArticleSummary {
    title: String,
    slug: String,
    description: String,
    created_at: String,
    tags: Vec<String>,
    favorites_count: i32,
    comments_count: i32,
    reading_time: i32,
}

// struct type to represent profile page content
#[derive(Debug, Serialize)]
struct ProfilePageContent {
    title: String,
    profile: UserProfile,
    articles: Vec<ArticleSummary>,
    favorites: Vec<ArticleSummary>,
    drafts: Vec<ArticleSummary>,
    current_user: Option<String>,
    current_page: i32,
    total_pages: i32,
}

// handler which renders the profile page template
#[debug_handler]
pub async fn get_profile_page(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    // Mock profile data - in a real app, this would come from the database
    let profile = UserProfile {
        username: username.clone(),
        bio: Some("Passionate Rust developer sharing insights on systems programming and web development.".to_string()),
        image: None,
        articles_count: 12,
        followers_count: 156,
        following_count: 89,
        following: false,
        website: Some("https://crustyrustacean.dev".to_string()),
        twitter: Some("crustyrustacean".to_string()),
        github: Some("crustyrustacean".to_string()),
    };

    // Mock articles data
    let articles = vec![
        ArticleSummary {
            title: "Getting Started with Axum Web Framework".to_string(),
            slug: "getting-started-with-axum".to_string(),
            description: "Learn how to build fast and safe web applications using Axum..."
                .to_string(),
            created_at: "2024-01-15".to_string(),
            tags: vec!["Rust".to_string(), "WebDev".to_string()],
            favorites_count: 23,
            comments_count: 5,
            reading_time: 8,
        },
        ArticleSummary {
            title: "Building RESTful APIs with Rust".to_string(),
            slug: "restful-apis-with-rust".to_string(),
            description: "A comprehensive guide to creating robust and performant REST APIs..."
                .to_string(),
            created_at: "2024-01-08".to_string(),
            tags: vec!["Rust".to_string(), "API".to_string()],
            favorites_count: 45,
            comments_count: 12,
            reading_time: 12,
        },
    ];

    let profile_content = ProfilePageContent {
        title: format!("{}'s Profile", username),
        profile,
        articles,
        favorites: vec![],  // Empty for now
        drafts: vec![],     // Empty for now
        current_user: None, // Would come from authentication
        current_page: 1,
        total_pages: 1,
    };

    let html = state
        .templates
        .render(
            "profile/profile.html",
            &tera::Context::from_serialize(&profile_content)?,
        )
        .map_err(|e| ApiError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}

pub async fn get_my_favorites_page(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    // Get user info for template context
    let conn = state.db.connect().map_err(|e| {
        tracing::error!("Database connection failed: {}", e);
        ApiError::InternalServerError(e.to_string())
    })?;

    let mut user_rows = conn
        .query(
            "SELECT username, email, bio, image FROM users WHERE id = ?",
            libsql::params![user.user_id.to_string()],
        )
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let user_info = if let Some(row) = user_rows
        .next()
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
    {
        let username: String = row
            .get(0)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        let email: String = row
            .get(1)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
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
    let conn = state.db.connect().map_err(|e| {
        tracing::error!("Database connection failed: {}", e);
        ApiError::InternalServerError(e.to_string())
    })?;

    let mut user_rows = conn
        .query(
            "SELECT username, email, bio, image FROM users WHERE id = ?",
            libsql::params![user.user_id.to_string()],
        )
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let user_info = if let Some(row) = user_rows
        .next()
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
    {
        let username: String = row
            .get(0)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        let email: String = row
            .get(1)
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
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
