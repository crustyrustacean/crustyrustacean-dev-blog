// src/lib/routes/articles.rs

use crate::{
    ApiError, AppState,
    auth::{ApiKeyUser, AuthenticatedUser, AuthorUser, OptionalUser},
    markdown::markdown_to_html,
    models::{
        ArticleQuery, ArticleResponse, CreateArticle, FeedQuery, MultipleArticlesResponse,
        SingleArticleResponse, UpdateArticle,
    },
};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
};
use chrono::{Datelike, Utc};
use serde_json::{Value, json};
use uuid::Uuid;
use validator::Validate;

/// Helper function to convert SQLite INTEGER to boolean draft status
///
/// Converts from SQLite's INTEGER representation to Rust bool:
/// - 0 = published (false)
/// - non-zero = draft (true)
#[inline]
pub(crate) fn draft_int_to_bool(draft: i64) -> bool {
    draft != 0
}

pub async fn create_article(
    State(state): State<AppState>,
    user: AuthorUser,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<SingleArticleResponse>), ApiError> {
    use crate::repositories::NewArticle;

    let article_data: CreateArticle = serde_json::from_value(
        payload
            .get("article")
            .ok_or_else(|| ApiError::BadRequest("Missing article field".to_string()))?
            .clone(),
    )
    .map_err(|_| ApiError::BadRequest("Invalid article data".to_string()))?;

    article_data
        .validate()
        .map_err(|e| ApiError::BadRequest(format!("Validation error: {}", e)))?;

    // Process shortcodes in article body before storing
    let processed_body = crate::shortcodes::process_shortcodes(&article_data.body, &state.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to process shortcodes: {}", e);
            ApiError::InternalServerError("Failed to process article links".to_string())
        })?;

    // Validate category if provided
    let category_id = if let Some(ref cat_slug) = article_data.category {
        let category = state
            .categories
            .find_by_slug(cat_slug)
            .await?
            .ok_or_else(|| {
                ApiError::BadRequest(format!("Category '{}' does not exist", cat_slug))
            })?;
        Some(category.id)
    } else {
        None
    };

    // Create the article using repository
    let new_article = NewArticle {
        title: article_data.title.clone(),
        description: article_data.description.clone(),
        body: processed_body,
        author_id: user.user_id,
        category_id,
        draft: article_data.draft,
    };

    let article = state.articles.create(&new_article).await?;

    // Set tags if provided
    if let Some(tags) = &article_data.tag_list {
        state.articles.set_tags(article.id, tags).await?;
    }

    // Get the full article with details for the response
    let details = state
        .articles
        .get_with_details(&article.slug, Some(user.user_id))
        .await?
        .ok_or_else(|| ApiError::InternalServerError("Failed to retrieve created article".to_string()))?;

    let rendered_body = markdown_to_html(&details.article.body);

    Ok((
        StatusCode::CREATED,
        Json(SingleArticleResponse {
            article: details.to_response(Some(rendered_body)),
        }),
    ))
}

pub async fn get_article(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    optional_user: OptionalUser,
) -> Result<Json<SingleArticleResponse>, ApiError> {
    let current_user_id = optional_user.user.as_ref().map(|u| u.user_id);

    let article_details = state
        .articles
        .get_with_details(&slug, current_user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Article not found".to_string()))?;

    // If article is a draft, only the author can see it
    if article_details.article.draft {
        match &optional_user.user {
            Some(auth_user) if auth_user.user_id == article_details.article.author_id => {}
            _ => return Err(ApiError::NotFound("Article not found".to_string())),
        }
    }

    let rendered_body = markdown_to_html(&article_details.article.body);
    let response = article_details.to_response(Some(rendered_body));

    Ok(Json(SingleArticleResponse { article: response }))
}

pub async fn get_editor_page(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    // Get user info for template context
    let conn = state.db.connect()?;

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

    let html = state
        .templates
        .render(
            "articles/editor.html",
            &tera::Context::from_serialize(&context)?,
        )
        .map_err(|e| ApiError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}

pub async fn get_edit_article_page(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    // Get the article to edit
    let article_response = get_article(
        State(state.clone()),
        Path(slug.clone()),
        OptionalUser {
            user: Some(user.clone()),
        },
    )
    .await?;
    let article = article_response.0.article;

    // Check if the current user is the author
    let conn = state.db.connect()?;

    let mut author_check = conn
        .query(
            "SELECT author_id FROM articles WHERE slug = ?",
            libsql::params![slug.clone()],
        )
        .await?;

    if let Some(row) = author_check.next().await? {
        let author_id_str: String = row.get(0)?;
        let author_id = Uuid::parse_str(&author_id_str)
            .map_err(|_| ApiError::InternalServerError("Invalid author ID".to_string()))?;

        if author_id != user.user_id {
            return Err(ApiError::Forbidden(
                "You can only edit your own articles".to_string(),
            ));
        }
    } else {
        return Err(ApiError::NotFound("Article not found".to_string()));
    }

    // Fetch user info for template context
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

    let html = state
        .templates
        .render(
            "articles/editor.html",
            &tera::Context::from_serialize(&context)?,
        )
        .map_err(|e| ApiError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}

pub async fn get_admin_dashboard(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(mut query): Query<ArticleQuery>,
) -> Result<impl IntoResponse, ApiError> {
    tracing::info!("Admin dashboard accessed by user: {}", user.user_id);

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
        .await?;

    let user_info = if let Some(row) = user_rows.next().await? {
        let username: String = row.get(0)?;
        let email: String = row.get(1)?;
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
            libsql::params![],
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
    let html = state
        .templates
        .render(
            "admin/dashboard.html",
            &tera::Context::from_serialize(&context)?,
        )
        .map_err(|e| ApiError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}

pub async fn update_article(
    State(state): State<AppState>,
    user: AuthorUser,
    Path(slug): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<SingleArticleResponse>, ApiError> {
    use crate::repositories::UpdateArticleData;

    let article_data: UpdateArticle = serde_json::from_value(
        payload
            .get("article")
            .ok_or_else(|| ApiError::BadRequest("Missing article field".to_string()))?
            .clone(),
    )
    .map_err(|_| ApiError::BadRequest("Invalid article data".to_string()))?;

    article_data
        .validate()
        .map_err(|e| ApiError::BadRequest(format!("Validation error: {}", e)))?;

    // Check if article exists and user is the author
    let article = state
        .articles
        .find_by_slug(&slug)
        .await?
        .ok_or_else(|| ApiError::NotFound("Article not found".to_string()))?;

    if article.author_id != user.user_id {
        return Err(ApiError::Forbidden(
            "You can only edit your own articles".to_string(),
        ));
    }

    // Process shortcodes if body is being updated
    let processed_body = if let Some(body) = &article_data.body {
        Some(
            crate::shortcodes::process_shortcodes(body, &state.db)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to process shortcodes: {}", e);
                    ApiError::InternalServerError("Failed to process article links".to_string())
                })?,
        )
    } else {
        None
    };

    // Resolve category_id from slug if provided
    let category_id = if let Some(ref cat_slug) = article_data.category {
        if cat_slug.is_empty() {
            // Empty string means clear the category
            Some(None)
        } else {
            // Validate category exists
            let category = state
                .categories
                .find_by_slug(cat_slug)
                .await?
                .ok_or_else(|| {
                    ApiError::BadRequest(format!("Category '{}' does not exist", cat_slug))
                })?;
            Some(Some(category.id))
        }
    } else {
        None // Don't update category
    };

    // Build update data
    let update_data = UpdateArticleData {
        title: article_data.title.clone(),
        description: article_data.description.clone(),
        body: processed_body,
        category_id,
        draft: article_data.draft,
        featured_image_id: None,
    };

    // Update the article
    state.articles.update(&slug, &update_data).await?;

    // Update tags if provided
    if let Some(tag_list) = &article_data.tag_list {
        state.articles.set_tags(article.id, tag_list).await?;
    }

    // Return updated article with details
    let details = state
        .articles
        .get_with_details(&slug, Some(user.user_id))
        .await?
        .ok_or_else(|| ApiError::NotFound("Article not found".to_string()))?;

    let rendered_body = markdown_to_html(&details.article.body);

    Ok(Json(SingleArticleResponse {
        article: details.to_response(Some(rendered_body)),
    }))
}

pub async fn delete_article(
    State(state): State<AppState>,
    user: AuthorUser,
    Path(slug): Path<String>,
) -> Result<StatusCode, ApiError> {
    // Check if article exists and user is the author
    let article = state
        .articles
        .find_by_slug(&slug)
        .await?
        .ok_or_else(|| ApiError::NotFound("Article not found".to_string()))?;

    if article.author_id != user.user_id {
        return Err(ApiError::Forbidden(
            "You can only delete your own articles".to_string(),
        ));
    }

    // Delete the article (repository handles cascading to tags and favorites)
    state.articles.delete(&slug).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_articles(
    State(state): State<AppState>,
    Query(query): Query<ArticleQuery>,
) -> Result<Json<MultipleArticlesResponse>, ApiError> {
    let articles_data = state.articles.list(&query, None, false).await?;

    let articles: Vec<ArticleResponse> = articles_data
        .into_iter()
        .map(|details| {
            let rendered_body = markdown_to_html(&details.article.body);
            details.to_response(Some(rendered_body))
        })
        .collect();

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
) -> Result<impl IntoResponse, ApiError> {
    // Get the article data using the existing API endpoint
    let article_response = get_article(
        State(state.clone()),
        Path(slug.clone()),
        OptionalUser {
            user: optional_user.user.clone(),
        },
    )
    .await?;
    let article = article_response.0.article;

    // Get user info if authenticated
    let user_info = if let Some(auth_user) = optional_user.user {
        let conn = state.db.connect()?;

        let mut rows = conn
            .query(
                "SELECT username, email, bio, image FROM users WHERE id = ?",
                libsql::params![auth_user.user_id.to_string()],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let username: String = row.get(0)?;
            let email: String = row.get(1)?;
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

    let html = state
        .templates
        .render(
            "articles/article.html",
            &tera::Context::from_serialize(&context)?,
        )
        .map_err(|e| ApiError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}

pub async fn get_articles_list_page(
    State(state): State<AppState>,
    optional_user: OptionalUser,
    Query(query): Query<ArticleQuery>,
) -> Result<impl IntoResponse, ApiError> {
    // Get user info if authenticated
    let user_info = if let Some(auth_user) = optional_user.user {
        let conn = state.db.connect()?;

        let mut rows = conn
            .query(
                "SELECT username, email, bio, image FROM users WHERE id = ?",
                libsql::params![auth_user.user_id.to_string()],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let username: String = row.get(0)?;
            let email: String = row.get(1)?;
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
    let conn = state.db.connect()?;

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

    let html = state
        .templates
        .render(
            "articles/list.html",
            &tera::Context::from_serialize(&context)?,
        )
        .map_err(|e| ApiError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}

pub async fn favorite_article(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(slug): Path<String>,
) -> Result<Json<SingleArticleResponse>, ApiError> {
    // Get the article to verify it exists and get its ID
    let article = state
        .articles
        .find_by_slug(&slug)
        .await?
        .ok_or_else(|| ApiError::NotFound("Article not found".to_string()))?;

    // Add to favorites (idempotent)
    state.articles.favorite(article.id, user.user_id).await?;

    // Return the article with updated favorite status
    let details = state
        .articles
        .get_with_details(&slug, Some(user.user_id))
        .await?
        .ok_or_else(|| ApiError::NotFound("Article not found".to_string()))?;

    let rendered_body = markdown_to_html(&details.article.body);
    Ok(Json(SingleArticleResponse {
        article: details.to_response(Some(rendered_body)),
    }))
}

pub async fn unfavorite_article(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(slug): Path<String>,
) -> Result<Json<SingleArticleResponse>, ApiError> {
    // Get the article to verify it exists and get its ID
    let article = state
        .articles
        .find_by_slug(&slug)
        .await?
        .ok_or_else(|| ApiError::NotFound("Article not found".to_string()))?;

    // Remove from favorites (idempotent)
    state.articles.unfavorite(article.id, user.user_id).await?;

    // Return the article with updated favorite status
    let details = state
        .articles
        .get_with_details(&slug, Some(user.user_id))
        .await?
        .ok_or_else(|| ApiError::NotFound("Article not found".to_string()))?;

    let rendered_body = markdown_to_html(&details.article.body);
    Ok(Json(SingleArticleResponse {
        article: details.to_response(Some(rendered_body)),
    }))
}

pub async fn get_articles_feed(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<FeedQuery>,
) -> Result<Json<MultipleArticlesResponse>, ApiError> {
    let articles_data = state.articles.get_feed(user.user_id, &query).await?;

    let articles: Vec<ArticleResponse> = articles_data
        .into_iter()
        .map(|details| {
            let rendered_body = markdown_to_html(&details.article.body);
            details.to_response(Some(rendered_body))
        })
        .collect();

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
) -> Result<impl IntoResponse, ApiError> {
    // Get user info for template context
    let conn = state.db.connect()?;

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
        .await?;

    let following_count = if let Some(row) = following_rows.next().await? {
        let count: i64 = row.get(0)?;
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

    let html = state
        .templates
        .render(
            "articles/feed.html",
            &tera::Context::from_serialize(&context)?,
        )
        .map_err(|e| ApiError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}

// Mobile upload endpoint - simplified API for iOS Shortcuts
// Accepts API key authentication via X-API-Key header
pub async fn mobile_upload_article(
    State(state): State<AppState>,
    api_user: ApiKeyUser,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use crate::repositories::NewArticle;

    // Extract article data from payload
    let title: String = payload
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::BadRequest("Missing title field".to_string()))?
        .to_string();

    let body: String = payload
        .get("body")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::BadRequest("Missing body field".to_string()))?
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

    // Create the article using repository
    let new_article = NewArticle {
        title: title.clone(),
        description,
        body,
        author_id: api_user.user_id,
        category_id: None,
        draft: false,
    };

    let article = state.articles.create(&new_article).await?;

    // Set tags if provided
    if !tags.is_empty() {
        state.articles.set_tags(article.id, &tags).await?;
    }

    // Return success response with article URL
    let article_url = format!("/articles/{}", article.slug);
    Ok(Json(json!({
        "success": true,
        "message": "Article created successfully",
        "slug": article.slug,
        "url": article_url,
        "id": article.id.to_string(),
    })))
}

/// Upload and parse a markdown file for article creation
/// POST /api/articles/upload-markdown
pub async fn upload_markdown_file(
    _state: State<AppState>,
    _user: AuthenticatedUser,
    mut multipart: Multipart,
) -> Result<Json<Value>, ApiError> {
    let mut filename: Option<String> = None;
    let mut file_content: Option<String> = None;

    // Process multipart form data
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Failed to read multipart field: {}", e)))?
    {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "file" {
            filename = field.file_name().map(|s| s.to_string());

            // Read file content as text
            let bytes = field
                .bytes()
                .await
                .map_err(|e| ApiError::BadRequest(format!("Failed to read file data: {}", e)))?;

            file_content =
                Some(String::from_utf8(bytes.to_vec()).map_err(|_| {
                    ApiError::BadRequest("File must be valid UTF-8 text".to_string())
                })?);
        }
    }

    // Validate we have a file
    let filename = filename.ok_or_else(|| ApiError::BadRequest("No file provided".to_string()))?;

    // Validate file extension
    if !filename.ends_with(".md") && !filename.ends_with(".markdown") {
        return Err(ApiError::BadRequest(
            "Only markdown files (.md or .markdown) are allowed".to_string(),
        ));
    }

    let content =
        file_content.ok_or_else(|| ApiError::BadRequest("No file content provided".to_string()))?;

    // Check file size (1MB limit for markdown files)
    const MAX_FILE_SIZE: usize = 1024 * 1024;
    if content.len() > MAX_FILE_SIZE {
        return Err(ApiError::BadRequest(
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
) -> Result<impl IntoResponse, ApiError> {
    // Get user info for template context
    let conn = state.db.connect()?;

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

    let html = state
        .templates
        .render(
            "admin/api-keys.html",
            &tera::Context::from_serialize(&context)?,
        )
        .map_err(|e| ApiError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}

/// List draft articles for the authenticated user
pub async fn list_user_drafts(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<MultipleArticlesResponse>, ApiError> {
    let articles_data = state.articles.list_user_drafts(user.user_id).await?;

    let articles: Vec<ArticleResponse> = articles_data
        .into_iter()
        .map(|details| {
            let rendered_body = markdown_to_html(&details.article.body);
            details.to_response(Some(rendered_body))
        })
        .collect();

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
) -> Result<impl IntoResponse, ApiError> {
    // Get user info for template context
    let conn = state.db.connect()?;

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

    let html = state
        .templates
        .render(
            "admin/drafts.html",
            &tera::Context::from_serialize(&context)?,
        )
        .map_err(|e| ApiError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(html))
}
