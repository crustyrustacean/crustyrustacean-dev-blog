// src/lib/repositories/article_libsql.rs

//! LibSQL implementation of the ArticleRepository trait.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use slug::slugify;
use uuid::Uuid;

use crate::database::DatabaseConnection;
use crate::models::{ArticleQuery, FeedQuery, UserProfile};

use super::error::{RepoResult, RepositoryError};
use super::article::{
    ArticleRecord, ArticleRepository, ArticleWithDetails, NewArticle, UpdateArticleData,
};

/// LibSQL implementation of the ArticleRepository.
#[derive(Debug, Clone)]
pub struct LibSqlArticleRepository {
    db: DatabaseConnection,
}

impl LibSqlArticleRepository {
    /// Create a new LibSQL article repository.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Helper to parse a datetime string.
    fn parse_datetime(s: &str) -> RepoResult<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| ndt.and_utc())
            })
            .map_err(|e| RepositoryError::InternalError(format!("Failed to parse datetime: {}", e)))
    }

    /// Helper to convert draft int to bool.
    fn draft_int_to_bool(draft: i64) -> bool {
        draft != 0
    }

    /// Helper to convert draft bool to int.
    fn draft_bool_to_int(draft: bool) -> i64 {
        if draft { 1 } else { 0 }
    }

    /// Generate a unique slug for an article.
    async fn generate_unique_slug(&self, title: &str) -> RepoResult<String> {
        let base_slug = slugify(title);
        let mut slug = base_slug.clone();
        let mut counter = 1;

        while self.slug_exists(&slug).await? {
            slug = format!("{}-{}", base_slug, counter);
            counter += 1;
        }

        Ok(slug)
    }

    /// Extract ArticleRecord from a database row.
    fn article_from_row(row: &libsql::Row, with_category_slug: bool) -> RepoResult<ArticleRecord> {
        let id_str: String = row.get(0)?;
        let id = Uuid::parse_str(&id_str)
            .map_err(|e| RepositoryError::InternalError(format!("Invalid UUID: {}", e)))?;

        let slug: String = row.get(1)?;
        let title: String = row.get(2)?;
        let description: String = row.get(3)?;
        let body: String = row.get(4)?;

        let author_id_str: String = row.get(5)?;
        let author_id = Uuid::parse_str(&author_id_str)
            .map_err(|e| RepositoryError::InternalError(format!("Invalid author UUID: {}", e)))?;

        let category_id: Option<Uuid> = row
            .get::<String>(6)
            .ok()
            .and_then(|s| Uuid::parse_str(&s).ok());

        let draft: i64 = row.get(7).unwrap_or(0);

        let created_at_str: String = row.get(8)?;
        let updated_at_str: String = row.get(9)?;

        let featured_image_id: Option<String> = row.get(10).ok();

        let category_slug: Option<String> = if with_category_slug {
            row.get(11).ok()
        } else {
            None
        };

        Ok(ArticleRecord {
            id,
            slug,
            title,
            description,
            body,
            author_id,
            category_id,
            category_slug,
            draft: Self::draft_int_to_bool(draft),
            featured_image_id,
            created_at: Self::parse_datetime(&created_at_str)?,
            updated_at: Self::parse_datetime(&updated_at_str)?,
        })
    }
}

#[async_trait]
impl ArticleRepository for LibSqlArticleRepository {
    async fn create(&self, article: &NewArticle) -> RepoResult<ArticleRecord> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let id = Uuid::new_v4();
        let slug = self.generate_unique_slug(&article.title).await?;
        let now = Utc::now();

        let category_id_param = match &article.category_id {
            Some(id) => libsql::Value::Text(id.to_string()),
            None => libsql::Value::Null,
        };

        conn.execute(
            r"INSERT INTO articles (id, slug, title, description, body, author_id, category_id, draft, created_at, updated_at)
              VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            vec![
                libsql::Value::Text(id.to_string()),
                libsql::Value::Text(slug.clone()),
                libsql::Value::Text(article.title.clone()),
                libsql::Value::Text(article.description.clone()),
                libsql::Value::Text(article.body.clone()),
                libsql::Value::Text(article.author_id.to_string()),
                category_id_param,
                libsql::Value::Integer(Self::draft_bool_to_int(article.draft)),
                libsql::Value::Text(now.to_rfc3339()),
                libsql::Value::Text(now.to_rfc3339()),
            ],
        )
        .await?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| RepositoryError::InternalError("Failed to retrieve created article".to_string()))
    }

    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<ArticleRecord>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                r"SELECT a.id, a.slug, a.title, a.description, a.body, a.author_id,
                         a.category_id, a.draft, a.created_at, a.updated_at, a.featured_image_id,
                         c.slug as category_slug
                  FROM articles a
                  LEFT JOIN categories c ON a.category_id = c.id
                  WHERE a.id = ?",
                libsql::params![id.to_string()],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(Self::article_from_row(&row, true)?))
        } else {
            Ok(None)
        }
    }

    async fn find_by_slug(&self, slug: &str) -> RepoResult<Option<ArticleRecord>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                r"SELECT a.id, a.slug, a.title, a.description, a.body, a.author_id,
                         a.category_id, a.draft, a.created_at, a.updated_at, a.featured_image_id,
                         c.slug as category_slug
                  FROM articles a
                  LEFT JOIN categories c ON a.category_id = c.id
                  WHERE a.slug = ?",
                libsql::params![slug],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(Self::article_from_row(&row, true)?))
        } else {
            Ok(None)
        }
    }

    async fn get_with_details(
        &self,
        slug: &str,
        current_user_id: Option<Uuid>,
    ) -> RepoResult<Option<ArticleWithDetails>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                r"SELECT a.id, a.slug, a.title, a.description, a.body, a.author_id,
                         a.category_id, a.draft, a.created_at, a.updated_at, a.featured_image_id,
                         c.slug as category_slug,
                         u.username, u.bio, u.image
                  FROM articles a
                  LEFT JOIN categories c ON a.category_id = c.id
                  JOIN users u ON a.author_id = u.id
                  WHERE a.slug = ?",
                libsql::params![slug],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let article = Self::article_from_row(&row, true)?;

            let username: String = row.get(12)?;
            let bio: Option<String> = row.get(13).ok();
            let image: Option<String> = row.get(14).ok();

            // Check if current user is following the author
            let following = if let Some(user_id) = current_user_id {
                let mut follow_rows = conn
                    .query(
                        "SELECT 1 FROM user_follows WHERE follower_id = ? AND following_id = ?",
                        libsql::params![user_id.to_string(), article.author_id.to_string()],
                    )
                    .await?;
                follow_rows.next().await?.is_some()
            } else {
                false
            };

            let author = UserProfile {
                username,
                bio,
                image,
                following,
            };

            let tag_list = self.get_tags(article.id).await?;
            let favorited = if let Some(user_id) = current_user_id {
                self.is_favorited(article.id, user_id).await?
            } else {
                false
            };
            let favorites_count = self.favorites_count(article.id).await?;

            Ok(Some(ArticleWithDetails {
                article,
                author,
                tag_list,
                favorited,
                favorites_count,
            }))
        } else {
            Ok(None)
        }
    }

    async fn list(
        &self,
        query: &ArticleQuery,
        current_user_id: Option<Uuid>,
        include_drafts: bool,
    ) -> RepoResult<Vec<ArticleWithDetails>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let limit = query.limit.unwrap_or(20);
        let offset = query.offset.unwrap_or(0);

        let mut where_clauses = Vec::new();
        let mut params: Vec<libsql::Value> = Vec::new();

        if !include_drafts {
            where_clauses.push("a.draft = 0".to_string());
        }

        if let Some(tag) = &query.tag {
            where_clauses.push(
                "EXISTS (SELECT 1 FROM article_tags at JOIN tags t ON at.tag_id = t.id WHERE at.article_id = a.id AND t.name = ?)".to_string()
            );
            params.push(libsql::Value::Text(tag.clone()));
        }

        if let Some(author) = &query.author {
            where_clauses.push("u.username = ?".to_string());
            params.push(libsql::Value::Text(author.clone()));
        }

        if let Some(favorited) = &query.favorited {
            where_clauses.push(
                "EXISTS (SELECT 1 FROM user_favorites uf JOIN users fu ON uf.user_id = fu.id WHERE uf.article_id = a.id AND fu.username = ?)".to_string()
            );
            params.push(libsql::Value::Text(favorited.clone()));
        }

        if let Some(category) = &query.category {
            where_clauses.push("c.slug = ?".to_string());
            params.push(libsql::Value::Text(category.clone()));
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let query_str = format!(
            r"SELECT a.id, a.slug, a.title, a.description, a.body, a.author_id,
                     a.category_id, a.draft, a.created_at, a.updated_at, a.featured_image_id,
                     c.slug as category_slug,
                     u.username, u.bio, u.image
              FROM articles a
              LEFT JOIN categories c ON a.category_id = c.id
              JOIN users u ON a.author_id = u.id
              {}
              ORDER BY a.created_at DESC
              LIMIT ? OFFSET ?",
            where_clause
        );

        params.push(libsql::Value::Integer(limit as i64));
        params.push(libsql::Value::Integer(offset as i64));

        let mut rows = conn.query(&query_str, libsql::params_from_iter(params)).await?;

        let mut articles = Vec::new();
        while let Some(row) = rows.next().await? {
            let article = Self::article_from_row(&row, true)?;

            let username: String = row.get(12)?;
            let bio: Option<String> = row.get(13).ok();
            let image: Option<String> = row.get(14).ok();

            let following = if let Some(user_id) = current_user_id {
                let mut follow_rows = conn
                    .query(
                        "SELECT 1 FROM user_follows WHERE follower_id = ? AND following_id = ?",
                        libsql::params![user_id.to_string(), article.author_id.to_string()],
                    )
                    .await?;
                follow_rows.next().await?.is_some()
            } else {
                false
            };

            let author = UserProfile {
                username,
                bio,
                image,
                following,
            };

            let tag_list = self.get_tags(article.id).await?;
            let favorited = if let Some(user_id) = current_user_id {
                self.is_favorited(article.id, user_id).await?
            } else {
                false
            };
            let favorites_count = self.favorites_count(article.id).await?;

            articles.push(ArticleWithDetails {
                article,
                author,
                tag_list,
                favorited,
                favorites_count,
            });
        }

        Ok(articles)
    }

    async fn count(&self, query: &ArticleQuery, include_drafts: bool) -> RepoResult<i32> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut where_clauses = Vec::new();
        let mut params: Vec<libsql::Value> = Vec::new();

        if !include_drafts {
            where_clauses.push("a.draft = 0".to_string());
        }

        if let Some(tag) = &query.tag {
            where_clauses.push(
                "EXISTS (SELECT 1 FROM article_tags at JOIN tags t ON at.tag_id = t.id WHERE at.article_id = a.id AND t.name = ?)".to_string()
            );
            params.push(libsql::Value::Text(tag.clone()));
        }

        if let Some(author) = &query.author {
            where_clauses.push("u.username = ?".to_string());
            params.push(libsql::Value::Text(author.clone()));
        }

        if let Some(favorited) = &query.favorited {
            where_clauses.push(
                "EXISTS (SELECT 1 FROM user_favorites uf JOIN users fu ON uf.user_id = fu.id WHERE uf.article_id = a.id AND fu.username = ?)".to_string()
            );
            params.push(libsql::Value::Text(favorited.clone()));
        }

        if let Some(category) = &query.category {
            where_clauses.push("c.slug = ?".to_string());
            params.push(libsql::Value::Text(category.clone()));
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let query_str = format!(
            r"SELECT COUNT(*)
              FROM articles a
              LEFT JOIN categories c ON a.category_id = c.id
              JOIN users u ON a.author_id = u.id
              {}",
            where_clause
        );

        let mut rows = conn.query(&query_str, libsql::params_from_iter(params)).await?;

        if let Some(row) = rows.next().await? {
            Ok(row.get(0).unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    async fn get_feed(
        &self,
        user_id: Uuid,
        query: &FeedQuery,
    ) -> RepoResult<Vec<ArticleWithDetails>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let limit = query.limit.unwrap_or(20);
        let offset = query.offset.unwrap_or(0);

        let query_str = r"SELECT a.id, a.slug, a.title, a.description, a.body, a.author_id,
                                 a.category_id, a.draft, a.created_at, a.updated_at, a.featured_image_id,
                                 c.slug as category_slug,
                                 u.username, u.bio, u.image
                          FROM articles a
                          LEFT JOIN categories c ON a.category_id = c.id
                          JOIN users u ON a.author_id = u.id
                          WHERE a.draft = 0
                            AND a.author_id IN (SELECT following_id FROM user_follows WHERE follower_id = ?)
                          ORDER BY a.created_at DESC
                          LIMIT ? OFFSET ?";

        let mut rows = conn
            .query(
                query_str,
                libsql::params![user_id.to_string(), limit, offset],
            )
            .await?;

        let mut articles = Vec::new();
        while let Some(row) = rows.next().await? {
            let article = Self::article_from_row(&row, true)?;

            let username: String = row.get(12)?;
            let bio: Option<String> = row.get(13).ok();
            let image: Option<String> = row.get(14).ok();

            let author = UserProfile {
                username,
                bio,
                image,
                following: true, // They're in the feed because we follow them
            };

            let tag_list = self.get_tags(article.id).await?;
            let favorited = self.is_favorited(article.id, user_id).await?;
            let favorites_count = self.favorites_count(article.id).await?;

            articles.push(ArticleWithDetails {
                article,
                author,
                tag_list,
                favorited,
                favorites_count,
            });
        }

        Ok(articles)
    }

    async fn count_feed(&self, user_id: Uuid) -> RepoResult<i32> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                r"SELECT COUNT(*)
                  FROM articles a
                  WHERE a.draft = 0
                    AND a.author_id IN (SELECT following_id FROM user_follows WHERE follower_id = ?)",
                libsql::params![user_id.to_string()],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(row.get(0).unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    async fn list_user_drafts(&self, user_id: Uuid) -> RepoResult<Vec<ArticleWithDetails>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let query_str = r"SELECT a.id, a.slug, a.title, a.description, a.body, a.author_id,
                                 a.category_id, a.draft, a.created_at, a.updated_at, a.featured_image_id,
                                 c.slug as category_slug,
                                 u.username, u.bio, u.image
                          FROM articles a
                          LEFT JOIN categories c ON a.category_id = c.id
                          JOIN users u ON a.author_id = u.id
                          WHERE a.draft = 1 AND a.author_id = ?
                          ORDER BY a.updated_at DESC";

        let mut rows = conn
            .query(query_str, libsql::params![user_id.to_string()])
            .await?;

        let mut articles = Vec::new();
        while let Some(row) = rows.next().await? {
            let article = Self::article_from_row(&row, true)?;

            let username: String = row.get(12)?;
            let bio: Option<String> = row.get(13).ok();
            let image: Option<String> = row.get(14).ok();

            let author = UserProfile {
                username,
                bio,
                image,
                following: false,
            };

            let tag_list = self.get_tags(article.id).await?;
            let favorited = self.is_favorited(article.id, user_id).await?;
            let favorites_count = self.favorites_count(article.id).await?;

            articles.push(ArticleWithDetails {
                article,
                author,
                tag_list,
                favorited,
                favorites_count,
            });
        }

        Ok(articles)
    }

    async fn slug_exists(&self, slug: &str) -> RepoResult<bool> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                "SELECT 1 FROM articles WHERE slug = ?",
                libsql::params![slug],
            )
            .await?;

        Ok(rows.next().await?.is_some())
    }

    async fn update(&self, slug: &str, data: &UpdateArticleData) -> RepoResult<ArticleRecord> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let existing = self.find_by_slug(slug).await?;
        if existing.is_none() {
            return Err(RepositoryError::NotFound(format!("Article with slug {}", slug)));
        }

        let mut updates = Vec::new();
        let mut params: Vec<libsql::Value> = Vec::new();

        if let Some(title) = &data.title {
            updates.push("title = ?");
            params.push(libsql::Value::Text(title.clone()));
        }
        if let Some(description) = &data.description {
            updates.push("description = ?");
            params.push(libsql::Value::Text(description.clone()));
        }
        if let Some(body) = &data.body {
            updates.push("body = ?");
            params.push(libsql::Value::Text(body.clone()));
        }
        if let Some(category_id) = &data.category_id {
            updates.push("category_id = ?");
            params.push(match category_id {
                Some(id) => libsql::Value::Text(id.to_string()),
                None => libsql::Value::Null,
            });
        }
        if let Some(draft) = data.draft {
            updates.push("draft = ?");
            params.push(libsql::Value::Integer(Self::draft_bool_to_int(draft)));
        }
        if let Some(featured_image_id) = &data.featured_image_id {
            updates.push("featured_image_id = ?");
            params.push(match featured_image_id {
                Some(id) => libsql::Value::Text(id.clone()),
                None => libsql::Value::Null,
            });
        }

        if updates.is_empty() {
            return self.find_by_slug(slug).await?.ok_or_else(|| {
                RepositoryError::InternalError("Article disappeared during update".to_string())
            });
        }

        updates.push("updated_at = ?");
        params.push(libsql::Value::Text(Utc::now().to_rfc3339()));

        params.push(libsql::Value::Text(slug.to_string()));

        let update_query = format!("UPDATE articles SET {} WHERE slug = ?", updates.join(", "));

        conn.execute(&update_query, libsql::params_from_iter(params))
            .await?;

        self.find_by_slug(slug).await?.ok_or_else(|| {
            RepositoryError::InternalError("Failed to retrieve updated article".to_string())
        })
    }

    async fn delete(&self, slug: &str) -> RepoResult<()> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let result = conn
            .execute(
                "DELETE FROM articles WHERE slug = ?",
                libsql::params![slug],
            )
            .await?;

        if result == 0 {
            return Err(RepositoryError::NotFound(format!("Article with slug {}", slug)));
        }

        Ok(())
    }

    async fn get_tags(&self, article_id: Uuid) -> RepoResult<Vec<String>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                r"SELECT t.name
                  FROM tags t
                  JOIN article_tags at ON t.id = at.tag_id
                  WHERE at.article_id = ?
                  ORDER BY t.name",
                libsql::params![article_id.to_string()],
            )
            .await?;

        let mut tags = Vec::new();
        while let Some(row) = rows.next().await? {
            let name: String = row.get(0)?;
            tags.push(name);
        }

        Ok(tags)
    }

    async fn set_tags(&self, article_id: Uuid, tags: &[String]) -> RepoResult<Vec<String>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // Clear existing tags
        conn.execute(
            "DELETE FROM article_tags WHERE article_id = ?",
            libsql::params![article_id.to_string()],
        )
        .await?;

        let mut associated_tags = Vec::new();

        for tag_name in tags {
            // Insert tag if it doesn't exist
            let tag_id = Uuid::new_v4();
            conn.execute(
                "INSERT OR IGNORE INTO tags (id, name) VALUES (?, ?)",
                libsql::params![tag_id.to_string(), tag_name.clone()],
            )
            .await?;

            // Get the tag ID
            let mut tag_rows = conn
                .query(
                    "SELECT id FROM tags WHERE name = ?",
                    libsql::params![tag_name.clone()],
                )
                .await?;

            if let Some(tag_row) = tag_rows.next().await? {
                let existing_tag_id: String = tag_row.get(0)?;

                // Link article to tag
                conn.execute(
                    "INSERT OR IGNORE INTO article_tags (article_id, tag_id) VALUES (?, ?)",
                    libsql::params![article_id.to_string(), existing_tag_id],
                )
                .await?;

                associated_tags.push(tag_name.clone());
            }
        }

        Ok(associated_tags)
    }

    async fn clear_tags(&self, article_id: Uuid) -> RepoResult<()> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        conn.execute(
            "DELETE FROM article_tags WHERE article_id = ?",
            libsql::params![article_id.to_string()],
        )
        .await?;

        Ok(())
    }

    async fn favorite(&self, article_id: Uuid, user_id: Uuid) -> RepoResult<()> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        conn.execute(
            "INSERT OR IGNORE INTO user_favorites (user_id, article_id) VALUES (?, ?)",
            libsql::params![user_id.to_string(), article_id.to_string()],
        )
        .await?;

        Ok(())
    }

    async fn unfavorite(&self, article_id: Uuid, user_id: Uuid) -> RepoResult<()> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        conn.execute(
            "DELETE FROM user_favorites WHERE user_id = ? AND article_id = ?",
            libsql::params![user_id.to_string(), article_id.to_string()],
        )
        .await?;

        Ok(())
    }

    async fn is_favorited(&self, article_id: Uuid, user_id: Uuid) -> RepoResult<bool> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                "SELECT 1 FROM user_favorites WHERE user_id = ? AND article_id = ?",
                libsql::params![user_id.to_string(), article_id.to_string()],
            )
            .await?;

        Ok(rows.next().await?.is_some())
    }

    async fn favorites_count(&self, article_id: Uuid) -> RepoResult<i32> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                "SELECT COUNT(*) FROM user_favorites WHERE article_id = ?",
                libsql::params![article_id.to_string()],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(row.get(0).unwrap_or(0))
        } else {
            Ok(0)
        }
    }
}
