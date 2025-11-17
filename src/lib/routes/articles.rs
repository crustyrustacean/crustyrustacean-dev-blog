// src/lib/routes/articles.rs

use crate::{
    AppError, AppState,
    auth::{ApiKeyUser, AuthenticatedUser, OptionalUser},
    markdown::markdown_to_html,
    models::{
        ArticleQuery, ArticleResponse, CreateArticle, FeedQuery, MultipleArticlesResponse,
        SingleArticleResponse, UpdateArticle, UserProfile,
    },
};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
};
use chrono::{Datelike, Utc};
use serde_json::{Value, json};
use slug::slugify;
use uuid::Uuid;
use validator::Validate;

/// Helper function to convert a boolean draft status to SQLite INTEGER (0 or 1)
///
/// SQLite doesn't have a native boolean type, so we use INTEGER where:
/// - 0 = published (false)
/// - 1 = draft (true)
#[inline]
pub(crate) fn draft_bool_to_int(draft: bool) -> i64 {
    if draft { 1 } else { 0 }
}

/// Helper function to convert SQLite INTEGER to boolean draft status
///
/// Converts from SQLite's INTEGER representation to Rust bool:
/// - 0 = published (false)
/// - non-zero = draft (true)
#[inline]
pub(crate) fn draft_int_to_bool(draft: i64) -> bool {
    draft != 0
}

/// Helper function to validate draft article visibility
///
/// Checks if a user is authorized to view a draft article.
/// Only the article author can view their own drafts.
///
/// # Arguments
/// * `is_draft` - Whether the article is a draft
/// * `author_id` - The UUID of the article's author
/// * `optional_user` - The optional authenticated user attempting to view the article
///
/// # Returns
/// * `Ok(())` if the user is authorized to view the article
/// * `Err(AppError::NotFound)` if the article is a draft and the user is not authorized
fn check_draft_visibility(
    is_draft: bool,
    author_id: &str,
    optional_user: &OptionalUser,
) -> Result<(), AppError> {
    if is_draft {
        if let Some(auth_user) = &optional_user.user {
            let author_uuid = Uuid::parse_str(author_id)
                .map_err(|_| AppError::InternalServerError("Invalid author ID".to_string()))?;
            if auth_user.user_id != author_uuid {
                return Err(AppError::NotFound("Article not found".to_string()));
            }
        } else {
            return Err(AppError::NotFound("Article not found".to_string()));
        }
    }
    Ok(())
}

/// Helper function to handle tag association for articles
///
/// This function handles the common logic for associating tags with articles:
/// 1. Creates tags if they don't exist
/// 2. Links the article to the tags in the article_tags junction table
///
/// # Arguments
/// * `conn` - Database connection
/// * `article_id` - UUID of the article to associate tags with
/// * `tag_list` - List of tag names to associate
///
/// # Returns
/// * `Vec<String>` - List of tag names that were successfully associated
async fn associate_tags_with_article(
    conn: &libsql::Connection,
    article_id: &str,
    tag_list: &[String],
) -> Result<Vec<String>, AppError> {
    let mut associated_tags = Vec::new();

    for tag_name in tag_list {
        // Insert tag if it doesn't exist
        let tag_id = Uuid::new_v4();
        conn.execute(
            "INSERT OR IGNORE INTO tags (id, name) VALUES (?, ?)",
            libsql::params![tag_id.to_string(), tag_name.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        // Get the tag ID (either newly created or existing)
        let mut tag_rows = conn
            .query(
                "SELECT id FROM tags WHERE name = ?",
                libsql::params![tag_name.clone()],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        if let Some(tag_row) = tag_rows
            .next()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
        {
            let existing_tag_id: String = tag_row
                .get(0)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;

            // Link article to tag
            conn.execute(
                "INSERT INTO article_tags (article_id, tag_id) VALUES (?, ?)",
                libsql::params![article_id, existing_tag_id],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

            associated_tags.push(tag_name.clone());
        }
    }

    Ok(associated_tags)
}

pub async fn create_article(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<SingleArticleResponse>), AppError> {
    let article_data: CreateArticle = serde_json::from_value(
        payload
            .get("article")
            .ok_or_else(|| AppError::BadRequest("Missing article field".to_string()))?
            .clone(),
    )
    .map_err(|_| AppError::BadRequest("Invalid article data".to_string()))?;

    article_data
        .validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Generate base slug from title
    let base_slug = slugify(&article_data.title);

    // Check if slug already exists and generate unique one if needed
    let mut slug = base_slug.clone();
    let mut counter = 1;

    loop {
        let mut slug_check = conn
            .query(
                "SELECT id FROM articles WHERE slug = ?",
                libsql::params![slug.clone()],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        if slug_check
            .next()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .is_none()
        {
            break; // Slug is unique
        }

        // Generate a new slug with counter
        slug = format!("{}-{}", base_slug, counter);
        counter += 1;
    }

    let article_id = Uuid::new_v4();
    let now = Utc::now();

    // Process shortcodes in article body before storing
    let processed_body = crate::shortcodes::process_shortcodes(&article_data.body, &state.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to process shortcodes: {}", e);
            AppError::InternalServerError("Failed to process article links".to_string())
        })?;

    // Handle category if provided
    let (category_id, category_slug) = if let Some(ref cat_slug) = article_data.category {
        let mut cat_rows = conn
            .query(
                "SELECT id FROM categories WHERE slug = ?",
                libsql::params![cat_slug.clone()],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        if let Some(cat_row) = cat_rows
            .next()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
        {
            let cat_id: String = cat_row
                .get(0)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;
            (Some(cat_id), Some(cat_slug.clone()))
        } else {
            return Err(AppError::BadRequest(format!(
                "Category '{}' does not exist",
                cat_slug
            )));
        }
    } else {
        (None, None)
    };

    // Insert the article
    let category_id_param = match category_id {
        Some(id) => libsql::Value::Text(id),
        None => libsql::Value::Null,
    };

    conn.execute(
        "INSERT INTO articles (id, slug, title, description, body, author_id, category_id, draft, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        vec![
            libsql::Value::Text(article_id.to_string()),
            libsql::Value::Text(slug.clone()),
            libsql::Value::Text(article_data.title.clone()),
            libsql::Value::Text(article_data.description.clone()),
            libsql::Value::Text(processed_body.clone()),
            libsql::Value::Text(user.user_id.to_string()),
            category_id_param,
            libsql::Value::Integer(draft_bool_to_int(article_data.draft)),
            libsql::Value::Text(now.to_rfc3339()),
            libsql::Value::Text(now.to_rfc3339()),
        ],
    )
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Handle tags if provided
    let tag_names = if let Some(tags) = &article_data.tag_list {
        associate_tags_with_article(&conn, &article_id.to_string(), tags).await?
    } else {
        Vec::new()
    };

    // Get author profile
    let mut author_rows = conn
        .query(
            "SELECT username, bio, image FROM users WHERE id = ?",
            libsql::params![user.user_id.to_string()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let author_row = author_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or_else(|| AppError::InternalServerError("Author not found".to_string()))?;

    let username: String = author_row
        .get(0)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let bio: Option<String> = author_row.get(1).ok();
    let image: Option<String> = author_row.get(2).ok();

    let author = UserProfile {
        username,
        bio,
        image,
        following: false, // Not relevant for article creation
    };

    let rendered_body = markdown_to_html(&processed_body);

    let article_response = ArticleResponse {
        slug: slug.clone(),
        title: article_data.title,
        description: article_data.description,
        body: processed_body,
        rendered_body: Some(rendered_body),
        tag_list: tag_names,
        category: category_slug,
        draft: article_data.draft,
        created_at: now,
        updated_at: now,
        favorited: false, // New article is not favorited by creator
        favorites_count: 0,
        author,
    };

    let response = SingleArticleResponse {
        article: article_response,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn get_article(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    optional_user: OptionalUser,
) -> Result<Json<SingleArticleResponse>, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Get the article with author information
    let mut article_rows = conn
        .query(
            r#"
            SELECT
                a.slug, a.title, a.description, a.body, a.created_at, a.updated_at, a.author_id,
                u.username, u.bio, u.image,
                c.slug as category_slug, a.draft
            FROM articles a
            JOIN users u ON a.author_id = u.id
            LEFT JOIN categories c ON a.category_id = c.id
            WHERE a.slug = ?
            "#,
            libsql::params![slug.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let article_row = article_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Article not found".to_string()))?;

    let article_slug: String = article_row
        .get(0)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let title: String = article_row
        .get(1)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let description: String = article_row
        .get(2)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let body: String = article_row
        .get(3)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let created_at_str: String = article_row
        .get(4)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let updated_at_str: String = article_row
        .get(5)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let author_id_str: String = article_row
        .get(6)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let username: String = article_row
        .get(7)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let bio: Option<String> = article_row.get(8).ok();
    let image: Option<String> = article_row.get(9).ok();
    let category_slug: Option<String> = article_row.get(10).ok();
    let draft: i64 = article_row
        .get(11)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let is_draft = draft_int_to_bool(draft);

    // If article is a draft, only the author can see it
    check_draft_visibility(is_draft, &author_id_str, &optional_user)?;

    // Parse the timestamps
    let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|e| AppError::InternalServerError(format!("Invalid timestamp: {}", e)))?
        .with_timezone(&Utc);
    let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
        .map_err(|e| AppError::InternalServerError(format!("Invalid timestamp: {}", e)))?
        .with_timezone(&Utc);

    // Get article ID by slug for tags
    let mut article_id_rows = conn
        .query(
            "SELECT id FROM articles WHERE slug = ?",
            libsql::params![slug.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let article_id_row = article_id_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or_else(|| AppError::InternalServerError("Article ID not found".to_string()))?;

    let article_id_str: String = article_id_row
        .get(0)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Get tags for this article
    let mut tag_rows = conn
        .query(
            r#"
            SELECT t.name
            FROM tags t
            JOIN article_tags at ON t.id = at.tag_id
            WHERE at.article_id = ?
            "#,
            libsql::params![article_id_str.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let mut tag_names = Vec::new();
    while let Some(tag_row) = tag_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        let tag_name: String = tag_row
            .get(0)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        tag_names.push(tag_name);
    }

    // Get favorites count
    let mut favorites_rows = conn
        .query(
            "SELECT COUNT(*) FROM user_favorites WHERE article_id = ?",
            libsql::params![article_id_str],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let favorites_count = if let Some(fav_row) = favorites_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        let count: i64 = fav_row
            .get(0)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        count as i32
    } else {
        0
    };

    let author = UserProfile {
        username,
        bio,
        image,
        following: false, // TODO: Implement based on current user if provided
    };

    let rendered_body = markdown_to_html(&body);

    let article_response = ArticleResponse {
        slug: article_slug,
        title,
        description,
        body,
        rendered_body: Some(rendered_body),
        tag_list: tag_names,
        category: category_slug,
        draft: is_draft,
        created_at,
        updated_at,
        favorited: false, // TODO: Implement based on current user if provided
        favorites_count,
        author,
    };

    let response = SingleArticleResponse {
        article: article_response,
    };

    Ok(Json(response))
}

pub async fn get_editor_page(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    // Get user info for template context
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

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

        Some(json!({
            "username": username,
            "email": email,
            "bio": bio,
            "image": image
        }))
    } else {
        None
    };

    let context: Value = json!({
        "title": "Write New Article",
        "page": "Editor",
        "current_year": chrono::Utc::now().year(),
        "user": user_info
    });

    let html = state.templates
        .render("articles/editor.html", &tera::Context::from_serialize(&context)?)
        .map_err(|e| AppError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}

pub async fn get_edit_article_page(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // Get the article to edit
    let article_response = get_article(State(state.clone()), Path(slug.clone()), OptionalUser { user: Some(user.clone()) }).await?;
    let article = article_response.0.article;

    // Check if the current user is the author
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let mut author_check = conn
        .query(
            "SELECT author_id FROM articles WHERE slug = ?",
            libsql::params![slug.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    if let Some(row) = author_check
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        let author_id_str: String = row
            .get(0)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let author_id = Uuid::parse_str(&author_id_str)
            .map_err(|_| AppError::InternalServerError("Invalid author ID".to_string()))?;

        if author_id != user.user_id {
            return Err(AppError::Forbidden(
                "You can only edit your own articles".to_string(),
            ));
        }
    } else {
        return Err(AppError::NotFound("Article not found".to_string()));
    }

    // Fetch user info for template context
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

        Some(json!({
            "username": username,
            "email": email,
            "bio": bio,
            "image": image
        }))
    } else {
        None
    };

    let context: Value = json!({
        "title": format!("Edit: {}", article.title),
        "page": "Editor",
        "current_year": chrono::Utc::now().year(),
        "article": {
            "title": article.title,
            "description": article.description,
            "body": article.body,
            "tag_list": article.tag_list,
            "draft": article.draft,
            "updated_at": article.updated_at
        },
        "is_edit": true,
        "article_slug": slug,
        "user": user_info
    });

    let html = state.templates
        .render("articles/editor.html", &tera::Context::from_serialize(&context)?)
        .map_err(|e| AppError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}

pub async fn get_admin_dashboard(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(mut query): Query<ArticleQuery>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("Admin dashboard accessed by user: {}", user.user_id);

    // Get user info for template context
    let conn = state.db.connect().map_err(|e| {
        tracing::error!("Database connection failed: {}", e);
        AppError::InternalServerError(e.to_string())
    })?;

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

        Some(json!({
            "username": username,
            "email": email,
            "bio": bio,
            "image": image
        }))
    } else {
        None
    };

    // Override limit to 10 for admin dashboard pagination
    let limit = query.limit.unwrap_or(10).min(100);
    query.limit = Some(limit);
    let offset = query.offset.unwrap_or(0);

    // Get all articles for the dashboard
    tracing::info!("Fetching articles for dashboard");
    let articles_response = list_articles(State(state.clone()), Query(query.clone())).await?;
    let articles = articles_response.0.articles;
    tracing::info!("Found {} articles for dashboard", articles.len());

    // Get total count of published articles (excluding drafts) for pagination
    // This must match the filtering done by list_articles
    let total_articles = match conn
        .query(
            "SELECT COUNT(*) FROM articles WHERE draft = 0",
            libsql::params![]
        )
        .await
    {
        Ok(mut rows) => {
            if let Ok(Some(row)) = rows.next().await {
                let count: i64 = row.get(0).unwrap_or(0);
                count
            } else {
                0
            }
        }
        Err(_) => 0,
    };

    let total_pages = ((total_articles as f64) / (limit as f64)).ceil() as i64;
    let current_page = ((offset / limit) + 1) as i64;

    let context: Value = json!({
        "title": "Admin Dashboard",
        "page": "Admin",
        "current_year": chrono::Utc::now().year(),
        "user": user_info,
        "articles": articles,
        "pagination": {
            "current_page": current_page,
            "total_pages": total_pages,
            "total_articles": total_articles,
            "limit": limit,
            "has_prev": offset > 0,
            "has_next": current_page < total_pages,
            "prev_offset": if offset > 0 { offset - limit } else { 0 },
            "next_offset": offset + limit
        }
    });

    tracing::info!("Rendering admin dashboard template");
    let html = state.templates
        .render("admin/dashboard.html", &tera::Context::from_serialize(&context)?)
        .map_err(|e| AppError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}

pub async fn update_article(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(slug): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<SingleArticleResponse>, AppError> {
    let article_data: UpdateArticle = serde_json::from_value(
        payload
            .get("article")
            .ok_or_else(|| AppError::BadRequest("Missing article field".to_string()))?
            .clone(),
    )
    .map_err(|_| AppError::BadRequest("Invalid article data".to_string()))?;

    article_data
        .validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Check if article exists and user is the author
    let mut article_rows = conn
        .query(
            "SELECT id, author_id FROM articles WHERE slug = ?",
            libsql::params![slug.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let article_row = article_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Article not found".to_string()))?;

    let _article_id_str: String = article_row
        .get(0)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let author_id_str: String = article_row
        .get(1)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let author_id = Uuid::parse_str(&author_id_str)
        .map_err(|_| AppError::InternalServerError("Invalid author ID".to_string()))?;

    // Check if the current user is the author
    if author_id != user.user_id {
        return Err(AppError::Forbidden(
            "You can only edit your own articles".to_string(),
        ));
    }

    let now = Utc::now();
    let mut updates = Vec::new();
    let mut params = Vec::new();

    if let Some(title) = &article_data.title {
        updates.push("title = ?");
        params.push(title.clone());
    }
    if let Some(description) = &article_data.description {
        updates.push("description = ?");
        params.push(description.clone());
    }
    if let Some(body) = &article_data.body {
        // Process shortcodes in body before storing
        let processed_body = crate::shortcodes::process_shortcodes(body, &state.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to process shortcodes: {}", e);
                AppError::InternalServerError("Failed to process article links".to_string())
            })?;
        updates.push("body = ?");
        params.push(processed_body);
    }

    if let Some(draft) = article_data.draft {
        updates.push("draft = ?");
        params.push(draft_bool_to_int(draft).to_string());
    }

    // Handle category update if provided
    if let Some(ref cat_slug) = article_data.category {
        if !cat_slug.is_empty() {
            // Validate that category exists
            let mut cat_rows = conn
                .query(
                    "SELECT id FROM categories WHERE slug = ?",
                    libsql::params![cat_slug.clone()],
                )
                .await
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;

            if let Some(cat_row) = cat_rows
                .next()
                .await
                .map_err(|e| AppError::InternalServerError(e.to_string()))?
            {
                let cat_id: String = cat_row
                    .get(0)
                    .map_err(|e| AppError::InternalServerError(e.to_string()))?;
                updates.push("category_id = ?");
                params.push(cat_id);
            } else {
                return Err(AppError::BadRequest(format!(
                    "Category '{}' does not exist",
                    cat_slug
                )));
            }
        } else {
            // Empty string means remove category
            updates.push("category_id = NULL");
        }
    }

    updates.push("updated_at = ?");
    params.push(now.to_rfc3339());
    params.push(slug.clone());

    if !updates.is_empty() {
        let sql = format!("UPDATE articles SET {} WHERE slug = ?", updates.join(", "));
        let mut libsql_params = Vec::new();
        for param in &params {
            libsql_params.push(libsql::Value::from(param.clone()));
        }

        conn.execute(&sql, libsql_params)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    }

    // Handle tag updates if provided
    if let Some(tag_list) = &article_data.tag_list {
        // Get the article ID first
        let mut id_rows = conn
            .query(
                "SELECT id FROM articles WHERE slug = ?",
                libsql::params![slug.clone()],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let id_row = id_rows
            .next()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Article not found".to_string()))?;

        let article_id: String = id_row
            .get(0)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        // Delete existing tag associations
        conn.execute(
            "DELETE FROM article_tags WHERE article_id = ?",
            libsql::params![article_id.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        // Add new tags using helper function
        associate_tags_with_article(&conn, &article_id, tag_list).await?;
    }

    // Return the updated article
    get_article(State(state), Path(slug), OptionalUser { user: Some(user) }).await
}

pub async fn delete_article(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(slug): Path<String>,
) -> Result<StatusCode, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Check if article exists and user is the author
    let mut article_rows = conn
        .query(
            "SELECT id, author_id FROM articles WHERE slug = ?",
            libsql::params![slug.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let article_row = article_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Article not found".to_string()))?;

    let article_id_str: String = article_row
        .get(0)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let author_id_str: String = article_row
        .get(1)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let author_id = Uuid::parse_str(&author_id_str)
        .map_err(|_| AppError::InternalServerError("Invalid author ID".to_string()))?;

    // Check if the current user is the author
    if author_id != user.user_id {
        return Err(AppError::Forbidden(
            "You can only delete your own articles".to_string(),
        ));
    }

    // Delete article tags first (foreign key constraint)
    conn.execute(
        "DELETE FROM article_tags WHERE article_id = ?",
        libsql::params![article_id_str.clone()],
    )
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Delete user favorites
    conn.execute(
        "DELETE FROM user_favorites WHERE article_id = ?",
        libsql::params![article_id_str.clone()],
    )
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Delete the article
    conn.execute(
        "DELETE FROM articles WHERE id = ?",
        libsql::params![article_id_str],
    )
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_articles(
    State(state): State<AppState>,
    Query(query): Query<ArticleQuery>,
) -> Result<Json<MultipleArticlesResponse>, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    // Build the query
    let mut where_clauses = Vec::new();
    let mut params = Vec::new();

    if let Some(tag) = &query.tag {
        where_clauses.push("EXISTS (SELECT 1 FROM article_tags at JOIN tags t ON at.tag_id = t.id WHERE at.article_id = a.id AND t.name = ?)");
        params.push(libsql::Value::from(tag.clone()));
    }

    if let Some(author) = &query.author {
        where_clauses.push("u.username = ?");
        params.push(libsql::Value::from(author.clone()));
    }

    if let Some(favorited_user) = &query.favorited {
        where_clauses.push("EXISTS (SELECT 1 FROM user_favorites uf JOIN users fu ON uf.user_id = fu.id WHERE uf.article_id = a.id AND fu.username = ?)");
        params.push(libsql::Value::from(favorited_user.clone()));
    }

    if let Some(category) = &query.category {
        where_clauses.push("c.slug = ?");
        params.push(libsql::Value::from(category.clone()));
    }

    // Always filter out drafts in public listing
    where_clauses.push("a.draft = 0");

    let where_clause = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let sql = format!(
        r#"
        SELECT
            a.id, a.slug, a.title, a.description, a.body, a.created_at, a.updated_at,
            u.username, u.bio, u.image,
            c.slug as category_slug, a.draft,
            GROUP_CONCAT(DISTINCT t.name) as tag_list,
            COUNT(DISTINCT uf.user_id) as favorites_count
        FROM articles a
        JOIN users u ON a.author_id = u.id
        LEFT JOIN categories c ON a.category_id = c.id
        LEFT JOIN article_tags at ON a.id = at.article_id
        LEFT JOIN tags t ON at.tag_id = t.id
        LEFT JOIN user_favorites uf ON a.id = uf.article_id
        {}
        GROUP BY a.id, a.slug, a.title, a.description, a.body, a.created_at, a.updated_at,
                 u.username, u.bio, u.image, c.slug, a.draft
        ORDER BY a.created_at DESC
        LIMIT ? OFFSET ?
        "#,
        where_clause
    );

    params.push(libsql::Value::from(limit));
    params.push(libsql::Value::from(offset));

    let mut article_rows = conn
        .query(&sql, params)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let mut articles = Vec::new();

    while let Some(row) = article_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        let _article_id: String = row
            .get(0)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let slug: String = row
            .get(1)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let title: String = row
            .get(2)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let description: String = row
            .get(3)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let body: String = row
            .get(4)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let created_at_str: String = row
            .get(5)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let updated_at_str: String = row
            .get(6)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let username: String = row
            .get(7)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let bio: Option<String> = row.get(8).ok();
        let image: Option<String> = row.get(9).ok();
        let category_slug: Option<String> = row.get(10).ok();
        let draft: i64 = row
            .get(11)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let is_draft = draft_int_to_bool(draft);

        // Get tags from GROUP_CONCAT result (comma-separated string)
        let tag_list_str: Option<String> = row.get(12).ok();
        let tag_names: Vec<String> = tag_list_str
            .map(|s| s.split(',').map(|t| t.to_string()).collect())
            .unwrap_or_default();

        // Get favorites count from the query
        let favorites_count: i64 = row
            .get(13)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let favorites_count = favorites_count as i32;

        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| AppError::InternalServerError(format!("Invalid timestamp: {}", e)))?
            .with_timezone(&Utc);
        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| AppError::InternalServerError(format!("Invalid timestamp: {}", e)))?
            .with_timezone(&Utc);

        let author = UserProfile {
            username,
            bio,
            image,
            following: false,
        };

        let rendered_body = markdown_to_html(&body);

        let article_response = ArticleResponse {
            slug,
            title,
            description,
            body,
            rendered_body: Some(rendered_body),
            tag_list: tag_names,
            category: category_slug,
            draft: is_draft,
            created_at,
            updated_at,
            favorited: false,
            favorites_count,
            author,
        };

        articles.push(article_response);
    }

    let articles_count = articles.len() as i32;

    Ok(Json(MultipleArticlesResponse {
        articles,
        articles_count,
    }))
}

pub async fn get_article_page(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    optional_user: OptionalUser,
) -> Result<impl IntoResponse, AppError> {
    // Get the article data using the existing API endpoint
    let article_response = get_article(State(state.clone()), Path(slug.clone()), OptionalUser { user: optional_user.user.clone() }).await?;
    let article = article_response.0.article;

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
        "title": format!("{} - CrustyRustacean Dev Blog", article.title),
        "page": "Article",
        "current_year": chrono::Utc::now().year(),
        "user": user_info,
        "article": article
    });

    let html = state.templates
        .render("articles/article.html", &tera::Context::from_serialize(&context)?)
        .map_err(|e| AppError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}

pub async fn get_articles_list_page(
    State(state): State<AppState>,
    optional_user: OptionalUser,
    Query(query): Query<ArticleQuery>,
) -> Result<impl IntoResponse, AppError> {
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

    // Pagination settings
    let limit = query.limit.unwrap_or(10); // Default to 10 articles per page
    let offset = query.offset.unwrap_or(0);
    let current_page = ((offset / limit) + 1) as i64;

    // Fetch articles with pagination
    let articles_query = ArticleQuery {
        tag: query.tag.clone(),
        author: query.author.clone(),
        favorited: query.favorited.clone(),
        category: query.category.clone(),
        limit: Some(limit),
        offset: Some(offset),
    };

    let articles = match list_articles(State(state.clone()), Query(articles_query)).await {
        Ok(articles_response) => articles_response.0.articles,
        Err(_) => vec![], // If there's an error fetching articles, show empty list
    };

    // Get total count of published articles for pagination
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Build WHERE clause for total count (same filters as list_articles)
    let mut where_clauses = Vec::new();
    let mut count_params = Vec::new();

    if let Some(tag) = &query.tag {
        where_clauses.push("EXISTS (SELECT 1 FROM article_tags at JOIN tags t ON at.tag_id = t.id WHERE at.article_id = a.id AND t.name = ?)");
        count_params.push(libsql::Value::from(tag.clone()));
    }

    if let Some(author) = &query.author {
        where_clauses.push("u.username = ?");
        count_params.push(libsql::Value::from(author.clone()));
    }

    if let Some(favorited_user) = &query.favorited {
        where_clauses.push("EXISTS (SELECT 1 FROM user_favorites uf JOIN users fu ON uf.user_id = fu.id WHERE uf.article_id = a.id AND fu.username = ?)");
        count_params.push(libsql::Value::from(favorited_user.clone()));
    }

    if let Some(category) = &query.category {
        where_clauses.push("c.slug = ?");
        count_params.push(libsql::Value::from(category.clone()));
    }

    // Always filter out drafts
    where_clauses.push("a.draft = 0");

    let where_clause = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let count_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM articles a
        JOIN users u ON a.author_id = u.id
        LEFT JOIN categories c ON a.category_id = c.id
        {}
        "#,
        where_clause
    );

    let total_articles = match conn.query(&count_sql, count_params).await {
        Ok(mut rows) => {
            if let Ok(Some(row)) = rows.next().await {
                let count: i64 = row.get(0).unwrap_or(0);
                count
            } else {
                0
            }
        }
        Err(_) => 0,
    };

    let total_pages = ((total_articles as f64) / (limit as f64)).ceil() as i64;

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

    // Count unique tags
    let tags_count = match conn
        .query("SELECT COUNT(DISTINCT name) FROM tags", libsql::params![])
        .await
    {
        Ok(mut rows) => {
            if let Ok(Some(row)) = rows.next().await {
                let count: i64 = row.get(0).unwrap_or(0);
                count
            } else {
                0
            }
        }
        Err(_) => 0,
    };

    // Get latest post date from the articles we already fetched
    let latest_post_date = if let Some(latest_article) = articles.first() {
        latest_article.created_at.format("%b %d, %Y").to_string()
    } else {
        "Never".to_string()
    };

    let context: Value = json!({
        "title": "All Articles - CrustyRustacean Dev Blog",
        "page": "Articles",
        "current_year": chrono::Utc::now().year(),
        "user": user_info,
        "articles": articles_json,
        "tags_count": tags_count,
        "latest_post_date": latest_post_date,
        "pagination": {
            "current_page": current_page,
            "total_pages": total_pages,
            "total_articles": total_articles,
            "limit": limit,
            "has_prev": offset > 0,
            "has_next": current_page < total_pages,
            "prev_offset": if offset > 0 { offset - limit } else { 0 },
            "next_offset": offset + limit
        }
    });

    let html = state.templates
        .render("articles/list.html", &tera::Context::from_serialize(&context)?)
        .map_err(|e| AppError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}

pub async fn favorite_article(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(slug): Path<String>,
) -> Result<Json<SingleArticleResponse>, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Get the article ID by slug
    let mut article_rows = conn
        .query(
            "SELECT id FROM articles WHERE slug = ?",
            libsql::params![slug.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let article_row = article_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Article not found".to_string()))?;

    let article_id: String = article_row
        .get(0)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Insert favorite (ignore if already exists - idempotent)
    conn.execute(
        "INSERT OR IGNORE INTO user_favorites (user_id, article_id) VALUES (?, ?)",
        libsql::params![user.user_id.to_string(), article_id],
    )
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Return the article with updated favorite status
    get_article_with_user_context(State(state), Path(slug), Some(user)).await
}

pub async fn unfavorite_article(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(slug): Path<String>,
) -> Result<Json<SingleArticleResponse>, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Get the article ID by slug
    let mut article_rows = conn
        .query(
            "SELECT id FROM articles WHERE slug = ?",
            libsql::params![slug.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let article_row = article_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Article not found".to_string()))?;

    let article_id: String = article_row
        .get(0)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Remove favorite (ignore if doesn't exist - idempotent)
    conn.execute(
        "DELETE FROM user_favorites WHERE user_id = ? AND article_id = ?",
        libsql::params![user.user_id.to_string(), article_id],
    )
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Return the article with updated favorite status
    get_article_with_user_context(State(state), Path(slug), Some(user)).await
}

// Helper function to get article with user context for favorite status
async fn get_article_with_user_context(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    user: Option<AuthenticatedUser>,
) -> Result<Json<SingleArticleResponse>, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Get the article with author information
    let mut article_rows = conn
        .query(
            r#"
            SELECT
                a.id, a.slug, a.title, a.description, a.body, a.created_at, a.updated_at, a.author_id,
                u.username, u.bio, u.image,
                c.slug as category_slug, a.draft
            FROM articles a
            JOIN users u ON a.author_id = u.id
            LEFT JOIN categories c ON a.category_id = c.id
            WHERE a.slug = ?
            "#,
            libsql::params![slug.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let article_row = article_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Article not found".to_string()))?;

    let article_id: String = article_row
        .get(0)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let article_slug: String = article_row
        .get(1)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let title: String = article_row
        .get(2)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let description: String = article_row
        .get(3)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let body: String = article_row
        .get(4)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let created_at_str: String = article_row
        .get(5)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let updated_at_str: String = article_row
        .get(6)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let _author_id: String = article_row
        .get(7)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let username: String = article_row
        .get(8)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let bio: Option<String> = article_row.get(9).ok();
    let image: Option<String> = article_row.get(10).ok();
    let category_slug: Option<String> = article_row.get(11).ok();
    let draft: i64 = article_row
        .get(12)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let is_draft = draft_int_to_bool(draft);

    // Parse the timestamps
    let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|e| AppError::InternalServerError(format!("Invalid timestamp: {}", e)))?
        .with_timezone(&Utc);
    let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
        .map_err(|e| AppError::InternalServerError(format!("Invalid timestamp: {}", e)))?
        .with_timezone(&Utc);

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
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let mut tag_names = Vec::new();
    while let Some(tag_row) = tag_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        let tag_name: String = tag_row
            .get(0)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        tag_names.push(tag_name);
    }

    // Get favorites count
    let mut favorites_rows = conn
        .query(
            "SELECT COUNT(*) FROM user_favorites WHERE article_id = ?",
            libsql::params![article_id.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let favorites_count = if let Some(fav_row) = favorites_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        let count: i64 = fav_row
            .get(0)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        count as i32
    } else {
        0
    };

    // Check if user has favorited this article
    let favorited = if let Some(ref auth_user) = user {
        let mut user_fav_rows = conn
            .query(
                "SELECT 1 FROM user_favorites WHERE user_id = ? AND article_id = ?",
                libsql::params![auth_user.user_id.to_string(), article_id],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        user_fav_rows
            .next()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .is_some()
    } else {
        false
    };

    let author = UserProfile {
        username,
        bio,
        image,
        following: false, // TODO: Implement based on current user if provided
    };

    let rendered_body = markdown_to_html(&body);

    let article_response = ArticleResponse {
        slug: article_slug,
        title,
        description,
        body,
        rendered_body: Some(rendered_body),
        tag_list: tag_names,
        category: category_slug,
        draft: is_draft,
        created_at,
        updated_at,
        favorited,
        favorites_count,
        author,
    };

    let response = SingleArticleResponse {
        article: article_response,
    };

    Ok(Json(response))
}

pub async fn get_articles_feed(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<FeedQuery>,
) -> Result<Json<MultipleArticlesResponse>, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    // Get articles from users that the current user follows
    let sql = r#"
        SELECT
            a.id, a.slug, a.title, a.description, a.body, a.created_at, a.updated_at,
            u.username, u.bio, u.image,
            c.slug as category_slug, a.draft
        FROM articles a
        JOIN users u ON a.author_id = u.id
        LEFT JOIN categories c ON a.category_id = c.id
        JOIN user_follows uf ON a.author_id = uf.following_id
        WHERE uf.follower_id = ? AND a.draft = 0
        ORDER BY a.created_at DESC
        LIMIT ? OFFSET ?
        "#;

    let params = libsql::params![user.user_id.to_string(), limit, offset];

    let mut article_rows = conn
        .query(sql, params)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let mut articles = Vec::new();

    while let Some(row) = article_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        let article_id: String = row
            .get(0)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let slug: String = row
            .get(1)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let title: String = row
            .get(2)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let description: String = row
            .get(3)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let body: String = row
            .get(4)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let created_at_str: String = row
            .get(5)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let updated_at_str: String = row
            .get(6)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let username: String = row
            .get(7)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let bio: Option<String> = row.get(8).ok();
        let image: Option<String> = row.get(9).ok();
        let category_slug: Option<String> = row.get(10).ok();
        let draft: i64 = row
            .get(11)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let is_draft = draft_int_to_bool(draft);

        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| AppError::InternalServerError(format!("Invalid timestamp: {}", e)))?
            .with_timezone(&Utc);
        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| AppError::InternalServerError(format!("Invalid timestamp: {}", e)))?
            .with_timezone(&Utc);

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
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let mut tag_names = Vec::new();
        while let Some(tag_row) = tag_rows
            .next()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
        {
            let tag_name: String = tag_row
                .get(0)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;
            tag_names.push(tag_name);
        }

        // Get favorites count
        let mut favorites_rows = conn
            .query(
                "SELECT COUNT(*) FROM user_favorites WHERE article_id = ?",
                libsql::params![article_id.clone()],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let favorites_count = if let Some(fav_row) = favorites_rows
            .next()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
        {
            let count: i64 = fav_row
                .get(0)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;
            count as i32
        } else {
            0
        };

        // Check if current user has favorited this article
        let mut user_fav_rows = conn
            .query(
                "SELECT 1 FROM user_favorites WHERE user_id = ? AND article_id = ?",
                libsql::params![user.user_id.to_string(), article_id],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let favorited = user_fav_rows
            .next()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .is_some();

        let author = UserProfile {
            username,
            bio,
            image,
            following: true, // By definition, we're following authors in the feed
        };

        let rendered_body = markdown_to_html(&body);

        let article_response = ArticleResponse {
            slug,
            title,
            description,
            body,
            rendered_body: Some(rendered_body),
            tag_list: tag_names,
            category: category_slug,
            draft: is_draft,
            created_at,
            updated_at,
            favorited,
            favorites_count,
            author,
        };

        articles.push(article_response);
    }

    let articles_count = articles.len() as i32;

    Ok(Json(MultipleArticlesResponse {
        articles,
        articles_count,
    }))
}

pub async fn get_articles_feed_page(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<FeedQuery>,
) -> Result<impl IntoResponse, AppError> {
    // Get user info for template context
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

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

        Some(json!({
            "username": username,
            "email": email,
            "bio": bio,
            "image": image
        }))
    } else {
        None
    };

    // Get articles for the feed
    let articles_response =
        get_articles_feed(State(state.clone()), user.clone(), Query(query)).await?;
    let articles = articles_response.0.articles;

    // Convert articles to JSON with formatted dates for template
    let articles_json: Vec<Value> = articles
        .iter()
        .map(|article| {
            json!({
                "slug": article.slug,
                "title": article.title,
                "description": article.description,
                "tagList": article.tag_list,
                "createdAt": article.created_at,
                "favoritesCount": article.favorites_count,
                "favorited": article.favorited,
                "author": article.author
            })
        })
        .collect();

    // Get following count
    let mut following_rows = conn
        .query(
            "SELECT COUNT(*) FROM user_follows WHERE follower_id = ?",
            libsql::params![user.user_id.to_string()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let following_count = if let Some(row) = following_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        let count: i64 = row
            .get(0)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        count
    } else {
        0
    };

    // Get latest post date from the articles
    let latest_post_date = if let Some(latest_article) = articles.first() {
        latest_article.created_at.format("%b %d, %Y").to_string()
    } else {
        "Never".to_string()
    };

    let context: Value = json!({
        "title": "Your Feed - CrustyRustacean Dev Blog",
        "page": "Feed",
        "current_year": chrono::Utc::now().year(),
        "user": user_info,
        "articles": articles_json,
        "following_count": following_count,
        "latest_post_date": latest_post_date,
    });

    let html = state.templates
        .render("articles/feed.html", &tera::Context::from_serialize(&context)?)
        .map_err(|e| AppError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}

// Mobile upload endpoint - simplified API for iOS Shortcuts
// Accepts API key authentication via X-API-Key header
pub async fn mobile_upload_article(
    State(state): State<AppState>,
    api_user: ApiKeyUser,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Extract article data from payload
    let title: String = payload
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing title field".to_string()))?
        .to_string();

    let body: String = payload
        .get("body")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing body field".to_string()))?
        .to_string();

    // Description is optional, use first 200 chars of body if not provided
    let description: String = payload
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            let clean_body = body.chars().take(200).collect::<String>();
            clean_body.trim().to_string()
        });

    // Tags are optional
    let tags: Vec<String> = payload
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    // Create the article using the same logic as create_article
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Generate base slug from title
    let base_slug = slugify(&title);

    // Check if slug already exists and generate unique one if needed
    let mut slug = base_slug.clone();
    let mut counter = 1;

    loop {
        let mut slug_check = conn
            .query(
                "SELECT id FROM articles WHERE slug = ?",
                libsql::params![slug.clone()],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        if slug_check
            .next()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .is_none()
        {
            break; // Slug is unique
        }

        // Generate a new slug with counter
        slug = format!("{}-{}", base_slug, counter);
        counter += 1;
    }

    let article_id = Uuid::new_v4();
    let now = Utc::now();

    // Insert the article
    conn.execute(
        "INSERT INTO articles (id, slug, title, description, body, author_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        libsql::params![
            article_id.to_string(),
            slug.clone(),
            title.clone(),
            description.clone(),
            body.clone(),
            api_user.user_id.to_string(),
            now.to_rfc3339(),
            now.to_rfc3339(),
        ],
    )
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Handle tags if provided
    if !tags.is_empty() {
        for tag_name in &tags {
            // Insert tag if it doesn't exist
            let tag_id = Uuid::new_v4();
            conn.execute(
                "INSERT OR IGNORE INTO tags (id, name) VALUES (?, ?)",
                libsql::params![tag_id.to_string(), tag_name.clone()],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

            // Get the tag ID (either the one we just created or the existing one)
            let mut tag_rows = conn
                .query(
                    "SELECT id FROM tags WHERE name = ?",
                    libsql::params![tag_name.clone()],
                )
                .await
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;

            if let Some(tag_row) = tag_rows
                .next()
                .await
                .map_err(|e| AppError::InternalServerError(e.to_string()))?
            {
                let tag_id_str: String = tag_row
                    .get(0)
                    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

                // Associate tag with article
                conn.execute(
                    "INSERT INTO article_tags (article_id, tag_id) VALUES (?, ?)",
                    libsql::params![article_id.to_string(), tag_id_str],
                )
                .await
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;
            }
        }
    }

    // Return success response with article URL
    let article_url = format!("/articles/{}", slug);
    Ok(Json(json!({
        "success": true,
        "message": "Article created successfully",
        "slug": slug,
        "url": article_url,
        "id": article_id.to_string(),
    })))
}

/// Upload and parse a markdown file for article creation
/// POST /api/articles/upload-markdown
pub async fn upload_markdown_file(
    _state: State<AppState>,
    _user: AuthenticatedUser,
    mut multipart: Multipart,
) -> Result<Json<Value>, AppError> {
    let mut filename: Option<String> = None;
    let mut file_content: Option<String> = None;

    // Process multipart form data
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to read multipart field: {}", e)))?
    {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "file" {
            filename = field.file_name().map(|s| s.to_string());

            // Read file content as text
            let bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(format!("Failed to read file data: {}", e)))?;

            file_content = Some(
                String::from_utf8(bytes.to_vec())
                    .map_err(|_| AppError::BadRequest("File must be valid UTF-8 text".to_string()))?,
            );
        }
    }

    // Validate we have a file
    let filename = filename.ok_or_else(|| AppError::BadRequest("No file provided".to_string()))?;

    // Validate file extension
    if !filename.ends_with(".md") && !filename.ends_with(".markdown") {
        return Err(AppError::BadRequest(
            "Only markdown files (.md or .markdown) are allowed".to_string(),
        ));
    }

    let content = file_content
        .ok_or_else(|| AppError::BadRequest("No file content provided".to_string()))?;

    // Check file size (1MB limit for markdown files)
    const MAX_FILE_SIZE: usize = 1024 * 1024;
    if content.len() > MAX_FILE_SIZE {
        return Err(AppError::BadRequest(
            "File size exceeds 1MB limit".to_string(),
        ));
    }

    // Parse the markdown content to extract title and description
    let (title, description) = parse_markdown_metadata(&content);

    // Return the parsed data
    Ok(Json(json!({
        "success": true,
        "title": title,
        "description": description,
        "body": content,
        "filename": filename,
    })))
}

/// Helper function to parse markdown content and extract title and description
///
/// Extracts:
/// - Title: First H1 heading (# Title) if present
/// - Description: First paragraph after the title
///
/// Returns (title, description)
fn parse_markdown_metadata(content: &str) -> (String, String) {
    let lines: Vec<&str> = content.lines().collect();
    let mut title = String::new();
    let mut description = String::new();
    let mut found_title = false;
    let mut skip_empty = true;

    for line in lines {
        let trimmed = line.trim();

        // Skip empty lines at the beginning
        if skip_empty && trimmed.is_empty() {
            continue;
        }
        skip_empty = false;

        // Extract title from first H1 heading
        if !found_title && trimmed.starts_with("# ") {
            title = trimmed.strip_prefix("# ").unwrap_or("").trim().to_string();
            found_title = true;
            continue;
        }

        // Extract description from first non-empty paragraph
        if found_title && description.is_empty() {
            // Skip empty lines after title
            if trimmed.is_empty() {
                continue;
            }
            // Skip other headings
            if trimmed.starts_with('#') {
                continue;
            }
            // This is the first paragraph - use it as description
            description = trimmed.to_string();
            break;
        }
    }

    // If no title found, use filename or default
    if title.is_empty() {
        title = "Untitled Article".to_string();
    }

    // If no description found, use first 200 chars of content
    if description.is_empty() {
        description = content
            .chars()
            .filter(|c| !c.is_whitespace() || *c == ' ')
            .take(200)
            .collect::<String>()
            .trim()
            .to_string();
    }

    (title, description)
}

// API Keys admin page
pub async fn get_api_keys_admin_page(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    // Get user info for template context
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

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

        Some(json!({
            "username": username,
            "email": email,
            "bio": bio,
            "image": image
        }))
    } else {
        None
    };

    let context: Value = json!({
        "title": "API Keys - CrustyRustacean Dev Blog",
        "page": "API Keys",
        "current_year": Utc::now().year(),
        "user": user_info,
    });

    let html = state.templates
        .render("admin/api-keys.html", &tera::Context::from_serialize(&context)?)
        .map_err(|e| AppError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}

/// List draft articles for the authenticated user
pub async fn list_user_drafts(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<MultipleArticlesResponse>, AppError> {
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Query only draft articles for the current user
    let sql = r#"
        SELECT
            a.id, a.slug, a.title, a.description, a.body, a.created_at, a.updated_at,
            u.username, u.bio, u.image,
            c.slug as category_slug, a.draft
        FROM articles a
        JOIN users u ON a.author_id = u.id
        LEFT JOIN categories c ON a.category_id = c.id
        WHERE a.author_id = ? AND a.draft = 1
        ORDER BY a.updated_at DESC
        "#;

    let params = libsql::params![user.user_id.to_string()];

    let mut article_rows = conn
        .query(sql, params)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let mut articles = Vec::new();

    while let Some(row) = article_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        let article_id: String = row
            .get(0)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let slug: String = row
            .get(1)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let title: String = row
            .get(2)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let description: String = row
            .get(3)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let body: String = row
            .get(4)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let created_at_str: String = row
            .get(5)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let updated_at_str: String = row
            .get(6)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let username: String = row
            .get(7)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let bio: Option<String> = row.get(8).ok();
        let image: Option<String> = row.get(9).ok();
        let category_slug: Option<String> = row.get(10).ok();
        let draft: i64 = row
            .get(11)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let is_draft = draft_int_to_bool(draft);

        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| AppError::InternalServerError(format!("Invalid timestamp: {}", e)))?
            .with_timezone(&Utc);
        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| AppError::InternalServerError(format!("Invalid timestamp: {}", e)))?
            .with_timezone(&Utc);

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
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let mut tag_names = Vec::new();
        while let Some(tag_row) = tag_rows
            .next()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
        {
            let tag_name: String = tag_row
                .get(0)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;
            tag_names.push(tag_name);
        }

        // Get favorites count (drafts won't have many, but for consistency)
        let mut favorites_rows = conn
            .query(
                "SELECT COUNT(*) FROM user_favorites WHERE article_id = ?",
                libsql::params![article_id],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let favorites_count = if let Some(fav_row) = favorites_rows
            .next()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
        {
            let count: i64 = fav_row
                .get(0)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;
            count as i32
        } else {
            0
        };

        let author = UserProfile {
            username,
            bio,
            image,
            following: false,
        };

        let rendered_body = markdown_to_html(&body);

        let article_response = ArticleResponse {
            slug,
            title,
            description,
            body,
            rendered_body: Some(rendered_body),
            tag_list: tag_names,
            category: category_slug,
            draft: is_draft,
            created_at,
            updated_at,
            favorited: false,
            favorites_count,
            author,
        };

        articles.push(article_response);
    }

    let articles_count = articles.len() as i32;

    Ok(Json(MultipleArticlesResponse {
        articles,
        articles_count,
    }))
}

/// Admin page for managing draft articles
pub async fn get_drafts_admin_page(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    // Get user info for template context
    let conn = state
        .db
        .connect()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

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

        Some(json!({
            "username": username,
            "email": email,
            "bio": bio,
            "image": image
        }))
    } else {
        None
    };

    // Get draft articles for this user
    let drafts_response = list_user_drafts(State(state.clone()), user.clone()).await?;
    let drafts = drafts_response.0.articles;

    let context: Value = json!({
        "title": "Draft Articles - CrustyRustacean Dev Blog",
        "page": "Drafts",
        "current_year": Utc::now().year(),
        "user": user_info,
        "drafts": drafts,
    });

    let html = state.templates
        .render("admin/drafts.html", &tera::Context::from_serialize(&context)?)
        .map_err(|e| AppError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}
