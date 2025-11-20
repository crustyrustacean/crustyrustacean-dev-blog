// src/lib/routes/index.rs

// route to serve the home page template

// dependencies
use crate::auth::OptionalUser;
use crate::errors::AppError;
use crate::models::ArticleQuery;
use crate::routes::articles::list_articles;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
};
use axum_macros::debug_handler;
use chrono::Datelike;
use serde_json::{Value, json};

// handler which renders the index page template
#[debug_handler]
pub async fn get_index(
    State(state): State<AppState>,
    optional_user: OptionalUser,
) -> Result<impl IntoResponse, AppError> {
    let current_year = chrono::Utc::now().year();

    // Get user info and ID if authenticated
    let user_id_opt = optional_user.user.as_ref().map(|u| u.user_id);
    let user_info = if let Some(auth_user) = optional_user.user.as_ref() {
        // Fetch user details from database
        let conn = state
            .db
            .connect()
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let mut rows = conn
            .query(
                "SELECT username, email, bio, image FROM users WHERE id = ?",
                libsql::params![auth_user.user_id.to_string()],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        if let Some(row) = rows
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

            Some(json!({
                "username": username,
                "email": email,
                "bio": bio,
                "image": image
            }))
        } else {
            None
        }
    } else {
        None
    };

    // Fetch recent articles for the homepage
    let articles_query = ArticleQuery {
        tag: None,
        author: None,
        favorited: None,
        category: None,
        limit: Some(4), // Show only 4 recent articles on homepage
        offset: Some(0),
    };

    let articles = match list_articles(State(state.clone()), Query(articles_query)).await {
        Ok(articles_response) => articles_response.0.articles,
        Err(_) => vec![], // If there's an error fetching articles, show empty list
    };

    // Convert articles to JSON with formatted dates
    let articles_json: Vec<Value> = articles
        .iter()
        .map(|article| {
            json!({
                "slug": article.slug,
                "title": article.title,
                "description": article.description,
                "tag_list": article.tag_list,
                "created_at": article.created_at.format("%b %d, %Y").to_string(),
                "author": article.author
            })
        })
        .collect();

    // Get additional statistics for the homepage summary in a single optimized query
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let (total_articles_count, draft_count, tags_count) = if let Some(user_id) = user_id_opt {
        // Combine all counts in a single query for authenticated users
        let mut rows = conn
            .query(
                r#"
                SELECT
                    (SELECT COUNT(*) FROM articles WHERE draft = 0) as total_articles,
                    (SELECT COUNT(*) FROM articles WHERE draft = 1 AND author_id = ?) as draft_count,
                    (SELECT COUNT(DISTINCT name) FROM tags) as tags_count
                "#,
                libsql::params![user_id.to_string()],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
        {
            let total: i64 = row.get(0).unwrap_or(0);
            let drafts: i64 = row.get(1).unwrap_or(0);
            let tags: i64 = row.get(2).unwrap_or(0);
            (total, drafts, tags)
        } else {
            (0, 0, 0)
        }
    } else {
        // Combine counts for unauthenticated users (no draft count needed)
        let mut rows = conn
            .query(
                r#"
                SELECT
                    (SELECT COUNT(*) FROM articles WHERE draft = 0) as total_articles,
                    (SELECT COUNT(DISTINCT name) FROM tags) as tags_count
                "#,
                libsql::params![],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
        {
            let total: i64 = row.get(0).unwrap_or(0);
            let tags: i64 = row.get(1).unwrap_or(0);
            (total, 0, tags)
        } else {
            (0, 0, 0)
        }
    };

    // Get latest post date from the articles we already fetched
    let latest_post_date = if let Some(latest_article) = articles.first() {
        latest_article.created_at.format("%b %d, %Y").to_string()
    } else {
        "Never".to_string()
    };

    let context: Value = json!({
        "title": "CrustyRustacean Dev Blog",
        "page": "Home",
        "message": "Welcome to CrustyRustacean Dev Blog",
        "user": user_info,
        "articles": articles_json,
        "total_articles_count": total_articles_count,
        "draft_count": draft_count,
        "tags_count": tags_count,
        "latest_post_date": latest_post_date,
        "flash_message": null,
        "flash_type": null,
        "current_year": current_year
    });

    let html = state
        .templates
        .render("index.html", &tera::Context::from_serialize(&context)?)
        .map_err(|e| AppError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}

// handler which renders the about page template
#[debug_handler]
pub async fn get_about(
    State(state): State<AppState>,
    optional_user: OptionalUser,
) -> Result<impl IntoResponse, AppError> {
    let current_year = chrono::Utc::now().year();

    // Get user info if authenticated
    let user_info = if let Some(auth_user) = optional_user.user {
        let conn = state
            .db
            .connect()
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let mut rows = conn
            .query(
                "SELECT username, email, bio, image FROM users WHERE id = ?",
                libsql::params![auth_user.user_id.to_string()],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        if let Some(row) = rows
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

            Some(json!({
                "username": username,
                "email": email,
                "bio": bio,
                "image": image
            }))
        } else {
            None
        }
    } else {
        None
    };

    let context: Value = json!({
        "title": "About - CrustyRustacean Dev Blog",
        "page": "About",
        "user": user_info,
        "current_year": current_year
    });

    let html = state
        .templates
        .render("about.html", &tera::Context::from_serialize(&context)?)
        .map_err(|e| AppError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}

// handler which renders the privacy policy page template
#[debug_handler]
pub async fn get_privacy(
    State(state): State<AppState>,
    optional_user: OptionalUser,
) -> Result<impl IntoResponse, AppError> {
    let current_year = chrono::Utc::now().year();

    // Get user info if authenticated
    let user_info = if let Some(auth_user) = optional_user.user {
        let conn = state
            .db
            .connect()
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let mut rows = conn
            .query(
                "SELECT username, email, bio, image FROM users WHERE id = ?",
                libsql::params![auth_user.user_id.to_string()],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        if let Some(row) = rows
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

            Some(json!({
                "username": username,
                "email": email,
                "bio": bio,
                "image": image
            }))
        } else {
            None
        }
    } else {
        None
    };

    let context: Value = json!({
        "title": "Privacy Policy - CrustyRustacean Dev Blog",
        "page": "Privacy",
        "user": user_info,
        "current_year": current_year
    });

    let html = state
        .templates
        .render("privacy.html", &tera::Context::from_serialize(&context)?)
        .map_err(|e| AppError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}

// handler which renders the terms of service page template
#[debug_handler]
pub async fn get_terms(
    State(state): State<AppState>,
    optional_user: OptionalUser,
) -> Result<impl IntoResponse, AppError> {
    let current_year = chrono::Utc::now().year();

    // Get user info if authenticated
    let user_info = if let Some(auth_user) = optional_user.user {
        let conn = state
            .db
            .connect()
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let mut rows = conn
            .query(
                "SELECT username, email, bio, image FROM users WHERE id = ?",
                libsql::params![auth_user.user_id.to_string()],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        if let Some(row) = rows
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

            Some(json!({
                "username": username,
                "email": email,
                "bio": bio,
                "image": image
            }))
        } else {
            None
        }
    } else {
        None
    };

    let context: Value = json!({
        "title": "Terms of Service - CrustyRustacean Dev Blog",
        "page": "Terms",
        "user": user_info,
        "current_year": current_year
    });

    let html = state
        .templates
        .render("terms.html", &tera::Context::from_serialize(&context)?)
        .map_err(|e| AppError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}
