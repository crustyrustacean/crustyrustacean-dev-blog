// src/lib/routes/profile.rs

// route handlers for profile pages

// dependencies
use crate::errors::AppError;
use crate::state::AppState;
use axum::{extract::{State, Path}, response::IntoResponse};
use axum_macros::debug_handler;
use axum_template::RenderHtml;
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
) -> Result<impl IntoResponse, AppError> {
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
            description: "Learn how to build fast and safe web applications using Axum...".to_string(),
            created_at: "2024-01-15".to_string(),
            tags: vec!["Rust".to_string(), "WebDev".to_string()],
            favorites_count: 23,
            comments_count: 5,
            reading_time: 8,
        },
        ArticleSummary {
            title: "Building RESTful APIs with Rust".to_string(),
            slug: "restful-apis-with-rust".to_string(),
            description: "A comprehensive guide to creating robust and performant REST APIs...".to_string(),
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
        favorites: vec![], // Empty for now
        drafts: vec![], // Empty for now
        current_user: None, // Would come from authentication
        current_page: 1,
        total_pages: 1,
    };

    Ok(RenderHtml("profile/profile.html", state.engine, profile_content))
}