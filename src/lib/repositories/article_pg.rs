// src/lib/repositories/article_pg.rs

//! PostgreSQL implementation of the ArticleRepository trait.

use async_trait::async_trait;
use slug::slugify;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::{ArticleQuery, FeedQuery, UserProfile};

use super::article::{
    ArticleRecord, ArticleRepository, ArticleWithDetails, NewArticle, UpdateArticleData,
};
use super::error::{RepoResult, RepositoryError};

/// A dynamically-bound query parameter.
#[derive(Debug, Clone)]
enum Val {
    Text(String),
    Uuid(Uuid),
    Int(i64),
    /// Binds SQL NULL for a UUID-typed column.
    NullUuid,
    /// Binds SQL NULL for a text-typed column.
    NullText,
}

/// Apply stored binds to a query in order.
fn bind_all<'q>(
    mut q: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
    binds: &[Val],
) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
    for v in binds {
        q = match v {
            Val::Text(s) => q.bind(s.clone()),
            Val::Uuid(u) => q.bind(*u),
            Val::Int(i) => q.bind(*i),
            Val::NullUuid => q.bind(None::<Uuid>),
            Val::NullText => q.bind(None::<String>),
        };
    }
    q
}

/// Renumber `$n` placeholders in a SQL fragment starting from `offset + 1`.
fn renumber(clause: &str, offset: usize) -> String {
    let mut out = String::with_capacity(clause.len());
    let mut rest = clause;
    while let Some(pos) = rest.find('$') {
        out.push_str(&rest[..pos]);
        let after = &rest[pos + 1..];
        let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() {
            out.push('$');
            rest = after;
            continue;
        }
        let n: usize = digits.parse().unwrap_or(0);
        out.push_str(&format!("${}", n + offset));
        rest = &after[digits.len()..];
    }
    out.push_str(rest);
    out
}

/// Shift all `$n` placeholders in `sql` by `offset`.
fn shift_placeholders(sql: &str, offset: usize) -> String {
    renumber(sql, offset)
}

/// Convert a sqlx row error into an `InternalError`.
fn internal(e: impl std::error::Error) -> RepositoryError {
    RepositoryError::InternalError(e.to_string())
}

/// PostgreSQL implementation of the ArticleRepository.
#[derive(Debug, Clone)]
pub struct PgArticleRepository {
    db: PgPool,
}

/// Row shape for the "with details" queries: article columns plus author,
/// aggregated tags, favorites count / favorited flag / following flag.
struct DetailsRow {
    article: ArticleRecord,
    author: UserProfile,
    tag_list: Vec<String>,
    favorited: bool,
    favorites_count: i32,
}

/// The SELECT list shared by `get_with_details`, `list`, `get_feed`, and
/// `list_user_drafts`. Placeholders:
///   - `${fav_uid}` and `${fol_uid}` are the current-user id binds (used
///     for the favorited/following status subqueries).
const DETAILS_SELECT: &str = r"SELECT a.id, a.slug, a.title, a.description, a.body, a.author_id,
       a.category_id, a.draft, a.created_at, a.updated_at, a.featured_image_id,
       c.slug as category_slug,
       u.username, u.bio, u.image,
       ARRAY(SELECT t.name FROM article_tags at2
             JOIN tags t ON at2.tag_id = t.id
             WHERE at2.article_id = a.id
             ORDER BY t.name) as tag_list,
       (SELECT COUNT(*) FROM user_favorites WHERE article_id = a.id)::INT as favorites_count,
       EXISTS(SELECT 1 FROM user_favorites
              WHERE article_id = a.id AND user_id = ${fav_uid}) as is_favorited,
       EXISTS(SELECT 1 FROM user_follows
              WHERE follower_id = ${fol_uid} AND following_id = a.author_id) as is_following
FROM articles a
LEFT JOIN categories c ON a.category_id = c.id
JOIN users u ON a.author_id = u.id";

impl PgArticleRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Extract the article columns from a details row.
    fn article_from_details_row(row: &sqlx::postgres::PgRow) -> RepoResult<ArticleRecord> {
        Ok(ArticleRecord {
            id: row.try_get("id").map_err(internal)?,
            slug: row.try_get("slug").map_err(internal)?,
            title: row.try_get("title").map_err(internal)?,
            description: row.try_get("description").map_err(internal)?,
            body: row.try_get("body").map_err(internal)?,
            author_id: row.try_get("author_id").map_err(internal)?,
            category_id: row.try_get("category_id").map_err(internal)?,
            category_slug: row.try_get("category_slug").map_err(internal)?,
            draft: row.try_get::<bool, _>("draft").unwrap_or(false),
            featured_image_id: row.try_get("featured_image_id").map_err(internal)?,
            created_at: row.try_get("created_at").map_err(internal)?,
            updated_at: row.try_get("updated_at").map_err(internal)?,
        })
    }

    /// Build a `DetailsRow` (article + author + tags + favorites) from a row
    /// produced by `DETAILS_SELECT`.
    ///
    /// `following_override` replaces the `is_following` column when the caller
    /// knows the value statically (e.g. feed rows are followed by definition,
    /// draft rows are the caller's own articles).
    fn details_from_row(
        row: &sqlx::postgres::PgRow,
        current_user_id: Option<Uuid>,
        following_override: Option<bool>,
    ) -> RepoResult<DetailsRow> {
        let article = Self::article_from_details_row(row)?;

        let tag_list: Vec<String> = row
            .try_get::<Option<Vec<String>>, _>("tag_list")
            .map_err(internal)?
            .unwrap_or_default();

        let favorites_count: i32 = row.try_get("favorites_count").unwrap_or(0);
        let favorited_row: bool = row.try_get("is_favorited").unwrap_or(false);
        let following_row: bool = row.try_get("is_following").unwrap_or(false);

        let favorited = current_user_id.is_some() && favorited_row;
        let following = following_override.unwrap_or(following_row && current_user_id.is_some());

        let author = UserProfile {
            username: row.try_get("username").map_err(internal)?,
            bio: row.try_get("bio").map_err(internal)?,
            image: row.try_get("image").map_err(internal)?,
            following,
        };

        Ok(DetailsRow {
            article,
            author,
            tag_list,
            favorited,
            favorites_count,
        })
    }

    /// Map a `DetailsRow` into the public `ArticleWithDetails` type.
    fn to_with_details(d: DetailsRow) -> ArticleWithDetails {
        ArticleWithDetails {
            article: d.article,
            author: d.author,
            tag_list: d.tag_list,
            favorited: d.favorited,
            favorites_count: d.favorites_count,
        }
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

    /// Build the WHERE clause for `list`/`count` from an `ArticleQuery`.
    fn build_list_filters(
        query: &ArticleQuery,
        include_drafts: bool,
        clauses: &mut Vec<String>,
        binds: &mut Vec<Val>,
    ) {
        if !include_drafts {
            clauses.push("a.draft = FALSE".to_string());
        }
        if let Some(tag) = &query.tag {
            binds.push(Val::Text(tag.clone()));
            let n = binds.len();
            clauses.push(format!(
                "EXISTS (SELECT 1 FROM article_tags at3 JOIN tags t ON at3.tag_id = t.id \
                 WHERE at3.article_id = a.id AND t.name = ${n})",
                n = n
            ));
        }
        if let Some(author) = &query.author {
            binds.push(Val::Text(author.clone()));
            let n = binds.len();
            clauses.push(format!("u.username = ${n}", n = n));
        }
        if let Some(favorited) = &query.favorited {
            binds.push(Val::Text(favorited.clone()));
            let n = binds.len();
            clauses.push(format!(
                "EXISTS (SELECT 1 FROM user_favorites uf JOIN users fu ON uf.user_id = fu.id \
                 WHERE uf.article_id = a.id AND fu.username = ${n})",
                n = n
            ));
        }
        if let Some(category) = &query.category {
            binds.push(Val::Text(category.clone()));
            let n = binds.len();
            clauses.push(format!("c.slug = ${n}", n = n));
        }
    }
}

#[async_trait]
impl ArticleRepository for PgArticleRepository {
    async fn create(&self, article: &NewArticle) -> RepoResult<ArticleRecord> {
        let slug = self.generate_unique_slug(&article.title).await?;

        let row = sqlx::query(
            r"INSERT INTO articles (slug, title, description, body, author_id, category_id, draft)
              VALUES ($1, $2, $3, $4, $5, $6, $7)
              RETURNING id, slug, title, description, body, author_id, category_id,
                        draft, featured_image_id, created_at, updated_at",
        )
        .bind(&slug)
        .bind(&article.title)
        .bind(&article.description)
        .bind(&article.body)
        .bind(article.author_id)
        .bind(article.category_id)
        .bind(article.draft)
        .fetch_one(&self.db)
        .await?;

        Ok(ArticleRecord {
            id: row.try_get("id").map_err(internal)?,
            slug: row.try_get("slug").map_err(internal)?,
            title: row.try_get("title").map_err(internal)?,
            description: row.try_get("description").map_err(internal)?,
            body: row.try_get("body").map_err(internal)?,
            author_id: row.try_get("author_id").map_err(internal)?,
            category_id: row.try_get("category_id").map_err(internal)?,
            category_slug: None,
            draft: row.try_get("draft").map_err(internal)?,
            featured_image_id: row.try_get("featured_image_id").map_err(internal)?,
            created_at: row.try_get("created_at").map_err(internal)?,
            updated_at: row.try_get("updated_at").map_err(internal)?,
        })
    }

    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<ArticleRecord>> {
        let row = sqlx::query(
            r"SELECT a.id, a.slug, a.title, a.description, a.body, a.author_id,
                     a.category_id, a.draft, a.created_at, a.updated_at, a.featured_image_id,
                     c.slug as category_slug
              FROM articles a
              LEFT JOIN categories c ON a.category_id = c.id
              WHERE a.id = $1",
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?;

        row.map(|r| {
            Ok(ArticleRecord {
                id: r.try_get("id").map_err(internal)?,
                slug: r.try_get("slug").map_err(internal)?,
                title: r.try_get("title").map_err(internal)?,
                description: r.try_get("description").map_err(internal)?,
                body: r.try_get("body").map_err(internal)?,
                author_id: r.try_get("author_id").map_err(internal)?,
                category_id: r.try_get("category_id").map_err(internal)?,
                category_slug: r.try_get("category_slug").map_err(internal)?,
                draft: r.try_get("draft").map_err(internal)?,
                featured_image_id: r.try_get("featured_image_id").map_err(internal)?,
                created_at: r.try_get("created_at").map_err(internal)?,
                updated_at: r.try_get("updated_at").map_err(internal)?,
            })
        })
        .transpose()
    }

    async fn find_by_slug(&self, slug: &str) -> RepoResult<Option<ArticleRecord>> {
        let row = sqlx::query(
            r"SELECT a.id, a.slug, a.title, a.description, a.body, a.author_id,
                     a.category_id, a.draft, a.created_at, a.updated_at, a.featured_image_id,
                     c.slug as category_slug
              FROM articles a
              LEFT JOIN categories c ON a.category_id = c.id
              WHERE a.slug = $1",
        )
        .bind(slug)
        .fetch_optional(&self.db)
        .await?;

        row.map(|r| {
            Ok(ArticleRecord {
                id: r.try_get("id").map_err(internal)?,
                slug: r.try_get("slug").map_err(internal)?,
                title: r.try_get("title").map_err(internal)?,
                description: r.try_get("description").map_err(internal)?,
                body: r.try_get("body").map_err(internal)?,
                author_id: r.try_get("author_id").map_err(internal)?,
                category_id: r.try_get("category_id").map_err(internal)?,
                category_slug: r.try_get("category_slug").map_err(internal)?,
                draft: r.try_get("draft").map_err(internal)?,
                featured_image_id: r.try_get("featured_image_id").map_err(internal)?,
                created_at: r.try_get("created_at").map_err(internal)?,
                updated_at: r.try_get("updated_at").map_err(internal)?,
            })
        })
        .transpose()
    }

    async fn get_with_details(
        &self,
        slug: &str,
        current_user_id: Option<Uuid>,
    ) -> RepoResult<Option<ArticleWithDetails>> {
        // When anonymous, bind NULL Uuids: the EXISTS subqueries simply
        // return FALSE, so the extra binds are harmless.
        let viewer = current_user_id.unwrap_or(Uuid::nil());

        let sql = DETAILS_SELECT.replace("${fav_uid}", "$1").replace("${fol_uid}", "$2");

        let sql = format!("{sql} WHERE a.slug = $3");

        let row = sqlx::query(&sql)
            .bind(viewer)
            .bind(viewer)
            .bind(slug)
            .fetch_optional(&self.db)
            .await?;

        match row {
            Some(row) => {
                let d = Self::details_from_row(&row, current_user_id, None)?;
                Ok(Some(Self::to_with_details(d)))
            }
            None => Ok(None),
        }
    }

    async fn list(
        &self,
        query: &ArticleQuery,
        current_user_id: Option<Uuid>,
        include_drafts: bool,
    ) -> RepoResult<Vec<ArticleWithDetails>> {
        let limit = query.limit.unwrap_or(20);
        let offset = query.offset.unwrap_or(0);

        let viewer = current_user_id.unwrap_or(Uuid::nil());

        let mut clauses: Vec<String> = Vec::new();
        let mut binds: Vec<Val> = Vec::new();

        Self::build_list_filters(query, include_drafts, &mut clauses, &mut binds);

        // Binds: $1/$2 viewer ids, then filters, then limit/offset.
        let mut all_binds = vec![Val::Uuid(viewer), Val::Uuid(viewer)];
        all_binds.extend(binds);

        let where_clause = if clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", clauses.join(" AND "))
        };

        // Filters were numbered from 1 in build_list_filters; shift by the
        // two viewer binds that precede them.
        let where_clause = shift_placeholders(&where_clause, 2);

        let limit_n = all_binds.len() + 1;
        let offset_n = all_binds.len() + 2;
        all_binds.push(Val::Int(limit as i64));
        all_binds.push(Val::Int(offset as i64));

        let sql = format!(
            "{} {} ORDER BY a.created_at DESC LIMIT ${} OFFSET ${}",
            DETAILS_SELECT
                .replace("${fav_uid}", "$1")
                .replace("${fol_uid}", "$2"),
            where_clause,
            limit_n,
            offset_n
        );

        let rows = bind_all(sqlx::query(&sql), &all_binds)
            .fetch_all(&self.db)
            .await?;

        let mut articles = Vec::with_capacity(rows.len());
        for row in &rows {
            let d = Self::details_from_row(row, current_user_id, None)?;
            articles.push(Self::to_with_details(d));
        }
        Ok(articles)
    }

    async fn count(&self, query: &ArticleQuery, include_drafts: bool) -> RepoResult<i32> {
        let mut clauses: Vec<String> = Vec::new();
        let mut binds: Vec<Val> = Vec::new();

        Self::build_list_filters(query, include_drafts, &mut clauses, &mut binds);

        let where_clause = if clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", clauses.join(" AND "))
        };

        let query_str = format!(
            r"SELECT COUNT(*)::INT
              FROM articles a
              LEFT JOIN categories c ON a.category_id = c.id
              JOIN users u ON a.author_id = u.id
              {where_clause}"
        );

        let row = bind_all(sqlx::query(&query_str), &binds)
            .fetch_one(&self.db)
            .await?;
        let count: i32 = row.try_get(0)?;

        Ok(count)
    }

    async fn get_feed(
        &self,
        user_id: Uuid,
        query: &FeedQuery,
    ) -> RepoResult<Vec<ArticleWithDetails>> {
        let limit = query.limit.unwrap_or(20);
        let offset = query.offset.unwrap_or(0);

        let sql = format!(
            "{} WHERE a.draft = FALSE
                     AND a.author_id IN (SELECT following_id FROM user_follows WHERE follower_id = $2)
              ORDER BY a.created_at DESC
              LIMIT $3 OFFSET $4",
            DETAILS_SELECT
                .replace("${fav_uid}", "$1")
                .replace("${fol_uid}", "$2")
        );

        let rows = sqlx::query(&sql)
            .bind(user_id) // $1 favorites status
            .bind(user_id) // $2 follows / feed filter
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await?;

        let mut articles = Vec::with_capacity(rows.len());
        for row in &rows {
            // Feed articles are followed by definition.
            let d = Self::details_from_row(row, Some(user_id), Some(true))?;
            articles.push(Self::to_with_details(d));
        }
        Ok(articles)
    }

    async fn count_feed(&self, user_id: Uuid) -> RepoResult<i32> {
        let (count,): (i32,) = sqlx::query_scalar(
            r"SELECT COUNT(*)::INT
              FROM articles a
              WHERE a.draft = FALSE
                AND a.author_id IN (SELECT following_id FROM user_follows WHERE follower_id = $1)",
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(count)
    }

    async fn list_user_drafts(&self, user_id: Uuid) -> RepoResult<Vec<ArticleWithDetails>> {
        let sql = format!(
            "{} WHERE a.draft = TRUE AND a.author_id = $2
              ORDER BY a.updated_at DESC",
            DETAILS_SELECT
                .replace("${fav_uid}", "$1")
                .replace("${fol_uid}", "$2")
        );

        let rows = sqlx::query(&sql)
            .bind(user_id)
            .bind(user_id)
            .fetch_all(&self.db)
            .await?;

        let mut articles = Vec::with_capacity(rows.len());
        for row in &rows {
            // The viewer is the author; following status is meaningless here.
            let d = Self::details_from_row(row, Some(user_id), Some(false))?;
            articles.push(Self::to_with_details(d));
        }
        Ok(articles)
    }

    async fn slug_exists(&self, slug: &str) -> RepoResult<bool> {
        let exists: Option<i32> = sqlx::query_scalar("SELECT 1 FROM articles WHERE slug = $1")
            .bind(slug)
            .fetch_optional(&self.db)
            .await?;

        Ok(exists.is_some())
    }

    async fn update(&self, slug: &str, data: &UpdateArticleData) -> RepoResult<ArticleRecord> {
        let existing = self.find_by_slug(slug).await?;
        if existing.is_none() {
            return Err(RepositoryError::NotFound(format!("Article with slug {}", slug)));
        }

        let mut updates: Vec<String> = Vec::new();
        let mut binds: Vec<Val> = Vec::new();

        macro_rules! set_text {
            ($field:expr, $value:expr) => {{
                binds.push(Val::Text($value.clone()));
                let n = binds.len();
                updates.push(format!("{} = ${}", $field, n));
            }};
        }

        if let Some(title) = &data.title {
            set_text!("title", title);
        }
        if let Some(description) = &data.description {
            set_text!("description", description);
        }
        if let Some(body) = &data.body {
            set_text!("body", body);
        }
        if let Some(draft) = data.draft {
            binds.push(Val::Int(if draft { 1 } else { 0 }));
            let n = binds.len();
            updates.push(format!("draft = ${}::BOOLEAN", n));
        }
        if let Some(category_id) = &data.category_id {
            match category_id {
                Some(id) => binds.push(Val::Uuid(*id)),
                None => binds.push(Val::NullUuid),
            }
            let n = binds.len();
            updates.push(format!("category_id = ${}::UUID", n));
        }
        if let Some(featured) = &data.featured_image_id {
            match featured {
                Some(id) => binds.push(Val::Text(id.clone())),
                None => binds.push(Val::NullText),
            }
            let n = binds.len();
            updates.push(format!("featured_image_id = ${}::TEXT", n));
        }

        if updates.is_empty() {
            return self.find_by_slug(slug).await?.ok_or_else(|| {
                RepositoryError::InternalError("Article disappeared during update".to_string())
            });
        }

        updates.push("updated_at = NOW()".to_string());
        binds.push(Val::Text(slug.to_string()));
        let n = binds.len();

        let query_str = format!(
            "UPDATE articles SET {} WHERE slug = ${}",
            updates.join(", "),
            n
        );

        bind_all(sqlx::query(&query_str), &binds)
            .execute(&self.db)
            .await?;

        self.find_by_slug(slug).await?.ok_or_else(|| {
            RepositoryError::InternalError("Failed to retrieve updated article".to_string())
        })
    }

    async fn delete(&self, slug: &str) -> RepoResult<()> {
        let result = sqlx::query("DELETE FROM articles WHERE slug = $1")
            .bind(slug)
            .execute(&self.db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!("Article with slug {}", slug)));
        }

        Ok(())
    }

    async fn get_tags(&self, article_id: Uuid) -> RepoResult<Vec<String>> {
        let tags: Vec<String> = sqlx::query_scalar(
            r"SELECT t.name
              FROM tags t
              JOIN article_tags at ON t.id = at.tag_id
              WHERE at.article_id = $1
              ORDER BY t.name",
        )
        .bind(article_id)
        .fetch_all(&self.db)
        .await?;

        Ok(tags)
    }

    async fn set_tags(&self, article_id: Uuid, tags: &[String]) -> RepoResult<Vec<String>> {
        // Clear existing tags
        sqlx::query("DELETE FROM article_tags WHERE article_id = $1")
            .bind(article_id)
            .execute(&self.db)
            .await?;

        let mut associated_tags = Vec::new();

        for tag_name in tags {
            // Insert tag if it doesn't exist, then link
            let tag_id: Uuid = sqlx::query_scalar(
                r"WITH ins AS (
                       INSERT INTO tags (name) VALUES ($1)
                       ON CONFLICT (name) DO NOTHING
                       RETURNING id
                   )
                   SELECT id FROM ins
                   UNION ALL
                   SELECT id FROM tags WHERE name = $1
                   LIMIT 1",
            )
            .bind(tag_name)
            .fetch_one(&self.db)
            .await?;

            sqlx::query(
                "INSERT INTO article_tags (article_id, tag_id) VALUES ($1, $2) \
                 ON CONFLICT DO NOTHING",
            )
            .bind(article_id)
            .bind(tag_id)
            .execute(&self.db)
            .await?;

            associated_tags.push(tag_name.clone());
        }

        Ok(associated_tags)
    }

    async fn clear_tags(&self, article_id: Uuid) -> RepoResult<()> {
        sqlx::query("DELETE FROM article_tags WHERE article_id = $1")
            .bind(article_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    async fn favorite(&self, article_id: Uuid, user_id: Uuid) -> RepoResult<()> {
        sqlx::query(
            "INSERT INTO user_favorites (user_id, article_id) VALUES ($1, $2) \
             ON CONFLICT DO NOTHING",
        )
        .bind(user_id)
        .bind(article_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn unfavorite(&self, article_id: Uuid, user_id: Uuid) -> RepoResult<()> {
        sqlx::query("DELETE FROM user_favorites WHERE user_id = $1 AND article_id = $2")
            .bind(user_id)
            .bind(article_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    async fn is_favorited(&self, article_id: Uuid, user_id: Uuid) -> RepoResult<bool> {
        let exists: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM user_favorites WHERE user_id = $1 AND article_id = $2",
        )
        .bind(user_id)
        .bind(article_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(exists.is_some())
    }

    async fn favorites_count(&self, article_id: Uuid) -> RepoResult<i32> {
        let count: i32 = sqlx::query_scalar(
            "SELECT COUNT(*)::INT FROM user_favorites WHERE article_id = $1",
        )
        .bind(article_id)
        .fetch_one(&self.db)
        .await?;

        Ok(count)
    }
}
