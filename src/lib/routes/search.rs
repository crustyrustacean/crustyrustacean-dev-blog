// src/lib/routes/search.rs

use crate::auth::OptionalUser;
use crate::errors::AppError;
use crate::models::{ArticleResponse, UserProfile};
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub results: Vec<ArticleResponse>,
    pub results_count: i32,
}

/// Search articles by query in title, description, or body
pub async fn search_articles(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
    OptionalUser { user }: OptionalUser,
) -> Result<Json<SearchResponse>, AppError> {
    let current_user_id = user.map(|u| u.user_id);
    // Validate query parameter
    let search_query = query.q.ok_or_else(|| {
        AppError::BadRequest("Search query is required".to_string())
    })?;

    if search_query.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Search query cannot be empty".to_string(),
        ));
    }

    let conn = state.db.connect().map_err(|e| {
        tracing::error!("Database connection failed: {:?}", e);
        AppError::InternalServerError("Database connection failed".to_string())
    })?;

    // Search articles using LIKE query (case-insensitive)
    // Search in title, description, and body
    let search_pattern = format!("%{}%", search_query);

    let mut rows = conn
        .query(
            "SELECT DISTINCT a.id, a.slug, a.title, a.description, a.body,
                    a.author_id, a.created_at, a.updated_at, c.slug as category_slug, a.draft
             FROM articles a
             LEFT JOIN categories c ON a.category_id = c.id
             WHERE (a.title LIKE ?1 COLLATE NOCASE
                OR a.description LIKE ?1 COLLATE NOCASE
                OR a.body LIKE ?1 COLLATE NOCASE)
                AND a.draft = 0
             ORDER BY a.created_at DESC",
            libsql::params![search_pattern],
        )
        .await
        .map_err(|e| {
            tracing::error!("Search query failed: {:?}", e);
            AppError::InternalServerError("Search failed".to_string())
        })?;

    let mut results = Vec::new();
    let mut article_ids = Vec::new();
    let mut author_ids = HashSet::new();

    // Collect article data and IDs
    while let Some(row) = rows.next().await.map_err(|e| {
        tracing::error!("Failed to read row: {:?}", e);
        AppError::InternalServerError("Failed to read search results".to_string())
    })? {
        let article_id: String = row.get(0).map_err(|_| {
            AppError::InternalServerError("Failed to parse article ID".to_string())
        })?;
        let slug: String = row.get(1).map_err(|_| {
            AppError::InternalServerError("Failed to parse slug".to_string())
        })?;
        let title: String = row.get(2).map_err(|_| {
            AppError::InternalServerError("Failed to parse title".to_string())
        })?;
        let description: String = row.get(3).map_err(|_| {
            AppError::InternalServerError("Failed to parse description".to_string())
        })?;
        let body: String = row.get(4).map_err(|_| {
            AppError::InternalServerError("Failed to parse body".to_string())
        })?;
        let author_id: String = row.get(5).map_err(|_| {
            AppError::InternalServerError("Failed to parse author ID".to_string())
        })?;
        let created_at: String = row.get(6).map_err(|_| {
            AppError::InternalServerError("Failed to parse created_at".to_string())
        })?;
        let updated_at: String = row.get(7).map_err(|_| {
            AppError::InternalServerError("Failed to parse updated_at".to_string())
        })?;
        let category_slug: Option<String> = row.get(8).ok();
        let draft: i64 = row.get(9).map_err(|_| {
            AppError::InternalServerError("Failed to parse draft".to_string())
        })?;

        article_ids.push(article_id.clone());
        author_ids.insert(author_id.clone());

        results.push((
            article_id,
            slug,
            title,
            description,
            body,
            author_id,
            created_at,
            updated_at,
            category_slug,
            draft,
        ));
    }

    // Fetch tags for all articles
    let mut article_tags = std::collections::HashMap::new();
    for article_id in &article_ids {
        let mut tag_rows = conn
            .query(
                "SELECT t.name
                 FROM tags t
                 INNER JOIN article_tags at ON t.id = at.tag_id
                 WHERE at.article_id = ?",
                libsql::params![article_id.clone()],
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch tags: {:?}", e);
                AppError::InternalServerError("Failed to fetch tags".to_string())
            })?;

        let mut tags = Vec::new();
        while let Some(row) = tag_rows.next().await.map_err(|_| {
            AppError::InternalServerError("Failed to read tag row".to_string())
        })? {
            let tag_name: String = row.get(0).map_err(|_| {
                AppError::InternalServerError("Failed to parse tag name".to_string())
            })?;
            tags.push(tag_name);
        }
        article_tags.insert(article_id.clone(), tags);
    }

    // Fetch favorites count and check if current user favorited
    let mut favorites_info = std::collections::HashMap::new();
    for article_id in &article_ids {
        let mut fav_row = conn
            .query(
                "SELECT COUNT(*) FROM user_favorites WHERE article_id = ?",
                libsql::params![article_id.clone()],
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch favorites count: {:?}", e);
                AppError::InternalServerError("Failed to fetch favorites".to_string())
            })?;

        let favorites_count: i32 = if let Some(row) = fav_row.next().await.map_err(|_| {
            AppError::InternalServerError("Failed to read favorites row".to_string())
        })? {
            row.get::<i32>(0).unwrap_or(0)
        } else {
            0
        };

        let favorited = if let Some(user_id) = current_user_id {
            let mut check_row = conn
                .query(
                    "SELECT COUNT(*) FROM user_favorites WHERE user_id = ? AND article_id = ?",
                    libsql::params![user_id.to_string(), article_id.clone()],
                )
                .await
                .map_err(|e| {
                    tracing::error!("Failed to check favorite status: {:?}", e);
                    AppError::InternalServerError("Failed to check favorite status".to_string())
                })?;

            if let Some(row) = check_row.next().await.map_err(|_| {
                AppError::InternalServerError("Failed to read favorite check row".to_string())
            })? {
                row.get::<i32>(0).unwrap_or(0) > 0
            } else {
                false
            }
        } else {
            false
        };

        favorites_info.insert(article_id.clone(), (favorited, favorites_count));
    }

    // Fetch author profiles
    let mut author_profiles = std::collections::HashMap::new();
    for author_id in author_ids {
        let mut author_row = conn
            .query(
                "SELECT username, bio, image FROM users WHERE id = ?",
                libsql::params![author_id.clone()],
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch author: {:?}", e);
                AppError::InternalServerError("Failed to fetch author".to_string())
            })?;

        if let Some(row) = author_row.next().await.map_err(|_| {
            AppError::InternalServerError("Failed to read author row".to_string())
        })? {
            let username: String = row.get(0).map_err(|_| {
                AppError::InternalServerError("Failed to parse username".to_string())
            })?;
            let bio: Option<String> = row.get(1).ok();
            let image: Option<String> = row.get(2).ok();

            let following = if let Some(user_id) = current_user_id {
                let mut follow_row = conn
                    .query(
                        "SELECT COUNT(*) FROM user_follows WHERE follower_id = ? AND following_id = ?",
                        libsql::params![user_id.to_string(), author_id.clone()],
                    )
                    .await
                    .map_err(|e| {
                        tracing::error!("Failed to check follow status: {:?}", e);
                        AppError::InternalServerError("Failed to check follow status".to_string())
                    })?;

                if let Some(row) = follow_row.next().await.map_err(|_| {
                    AppError::InternalServerError("Failed to read follow check row".to_string())
                })? {
                    row.get::<i32>(0).unwrap_or(0) > 0
                } else {
                    false
                }
            } else {
                false
            };

            author_profiles.insert(
                author_id,
                UserProfile {
                    username,
                    bio,
                    image,
                    following,
                },
            );
        }
    }

    // Build response
    let article_responses: Vec<ArticleResponse> = results
        .into_iter()
        .filter_map(
            |(article_id, slug, title, description, body, author_id, created_at_str, updated_at_str, category_slug, draft)| {
                // Parse dates
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .ok()?
                    .with_timezone(&Utc);
                let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                    .ok()?
                    .with_timezone(&Utc);

                let tag_list = article_tags.get(&article_id).cloned().unwrap_or_default();
                let (favorited, favorites_count) =
                    favorites_info.get(&article_id).cloned().unwrap_or((false, 0));
                let author = author_profiles
                    .get(&author_id)
                    .cloned()
                    .unwrap_or_else(|| UserProfile {
                        username: "unknown".to_string(),
                        bio: None,
                        image: None,
                        following: false,
                    });

                Some(ArticleResponse {
                    slug,
                    title,
                    description,
                    body,
                    rendered_body: None,
                    tag_list,
                    category: category_slug,
                    draft: draft != 0,
                    created_at,
                    updated_at,
                    favorited,
                    favorites_count,
                    author,
                })
            },
        )
        .collect();

    let results_count = article_responses.len() as i32;

    Ok(Json(SearchResponse {
        results: article_responses,
        results_count,
    }))
}
