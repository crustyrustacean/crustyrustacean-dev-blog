// src/lib/repositories/user_pg.rs

//! PostgreSQL implementation of the UserRepository trait.

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::{AdminUsersQuery, ProfilesQuery, Role, User, UserProfile};

use super::error::{RepoResult, RepositoryError};
use super::user::{NewUser, UpdateUser, UserRepository, UserSummary};

/// A dynamically-bound query parameter.
#[derive(Debug, Clone)]
enum Val {
    Text(String),
    Uuid(Uuid),
    Bool(bool),
    Int(i64),
}

/// Small helper that builds a parameterised SQL fragment while keeping
/// placeholder numbering ($1, $2, ...) in sync with the bind order.
#[derive(Default)]
struct Frag {
    sql: Vec<String>,
    binds: Vec<Val>,
}

impl Frag {
    fn push(&mut self, clause: impl Into<String>, bind: Option<Val>) {
        if let Some(v) = bind {
            let n = self.binds.len() + 1;
            self.sql.push(format!("{} ${}", clause.into(), n));
            self.binds.push(v);
        } else {
            self.sql.push(clause.into());
        }
    }

    fn is_empty(&self) -> bool {
        self.sql.is_empty()
    }

    fn join(&self, sep: &str) -> String {
        self.sql.join(sep)
    }
}

/// Apply the stored binds to a query in order.
fn bind_all<'q>(
    mut q: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
    binds: &[Val],
) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
    for v in binds {
        q = match v {
            Val::Text(s) => q.bind(s.clone()),
            Val::Uuid(u) => q.bind(*u),
            Val::Bool(b) => q.bind(*b),
            Val::Int(i) => q.bind(*i),
        };
    }
    q
}

/// Convert a sqlx row error into an `InternalError`.
fn internal(e: impl std::error::Error) -> RepositoryError {
    RepositoryError::InternalError(e.to_string())
}

/// PostgreSQL implementation of the UserRepository.
#[derive(Debug, Clone)]
pub struct PgUserRepository {
    db: PgPool,
}

impl PgUserRepository {
    /// Create a new PostgreSQL user repository.
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Map a database row to a `User`.
    fn user_from_row(row: &sqlx::postgres::PgRow) -> RepoResult<User> {
        let role_str: String = row
            .try_get("role")
            .unwrap_or_else(|_| "subscriber".into());
        let role: Role = role_str
            .parse()
            .map_err(|_| RepositoryError::InternalError("Invalid role in database".to_string()))?;

        Ok(User {
            id: row.try_get("id").map_err(internal)?,
            username: row.try_get("username").map_err(internal)?,
            email: row.try_get("email").map_err(internal)?,
            password_hash: row.try_get("password_hash").map_err(internal)?,
            bio: row.try_get("bio").map_err(internal)?,
            image: row.try_get("image").map_err(internal)?,
            disabled: row.try_get::<bool, _>("disabled").unwrap_or(false),
            role,
            email_verified: row.try_get::<bool, _>("email_verified").unwrap_or(false),
            theme_preference: row.try_get("theme_preference").map_err(internal)?,
            created_at: row.try_get("created_at").map_err(internal)?,
            updated_at: row.try_get("updated_at").map_err(internal)?,
        })
    }

    /// Shared SELECT column list for user queries.
    const USER_COLUMNS: &'static str = "id, username, email, password_hash, bio, image, disabled, role, email_verified, theme_preference, created_at, updated_at";
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create(&self, user: &NewUser) -> RepoResult<User> {
        let row = sqlx::query(&format!(
            r"INSERT INTO users (username, email, password_hash, role)
              VALUES ($1, $2, $3, $4)
              RETURNING {}",
            Self::USER_COLUMNS
        ))
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(user.role.to_string())
        .fetch_one(&self.db)
        .await?;

        Ok(Self::user_from_row(&row)?)
    }

    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<User>> {
        let row = sqlx::query(&format!(
            "SELECT {} FROM users WHERE id = $1",
            Self::USER_COLUMNS
        ))
        .bind(id)
        .fetch_optional(&self.db)
        .await?;

        row.map(|r| Self::user_from_row(&r)).transpose()
    }

    async fn find_by_username(&self, username: &str) -> RepoResult<Option<User>> {
        let row = sqlx::query(&format!(
            "SELECT {} FROM users WHERE username = $1",
            Self::USER_COLUMNS
        ))
        .bind(username)
        .fetch_optional(&self.db)
        .await?;

        row.map(|r| Self::user_from_row(&r)).transpose()
    }

    async fn find_by_email(&self, email: &str) -> RepoResult<Option<User>> {
        let row = sqlx::query(&format!(
            "SELECT {} FROM users WHERE email = $1",
            Self::USER_COLUMNS
        ))
        .bind(email)
        .fetch_optional(&self.db)
        .await?;

        row.map(|r| Self::user_from_row(&r)).transpose()
    }

    async fn list(&self, query: &AdminUsersQuery) -> RepoResult<Vec<UserSummary>> {
        let limit = query.limit.unwrap_or(20);
        let offset = query.offset.unwrap_or(0);

        let mut frag = Frag::default();

        // Search filter (single bind reused for both columns)
        if let Some(s) = &query.search
            && !s.trim().is_empty()
        {
            frag.binds.push(Val::Text(format!("%{}%", s.trim())));
            let n = frag.binds.len();
            frag.sql.push(format!(
                "(u.username ILIKE ${} OR u.email ILIKE ${})",
                n, n
            ));
        }

        // Status filter
        match query.status.as_deref() {
            Some("active") => frag.push("u.disabled = FALSE", None),
            Some("disabled") => frag.push("u.disabled = TRUE", None),
            _ => {} // "all" or None - no filter
        }

        let where_clause = if frag.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", frag.join(" AND "))
        };

        let limit_idx = frag.binds.len() + 1;
        let offset_idx = limit_idx + 1;

        let query_str = format!(
            r"SELECT u.id, u.username, u.email, u.bio, u.image, u.disabled, u.role,
                     u.email_verified, u.created_at, COUNT(a.id)::INT as article_count
              FROM users u
              LEFT JOIN articles a ON u.id = a.author_id
              {}
              GROUP BY u.id
              ORDER BY u.created_at DESC
              LIMIT ${} OFFSET ${}",
            where_clause, limit_idx, offset_idx
        );

        frag.binds.push(Val::Int(limit as i64));
        frag.binds.push(Val::Int(offset as i64));

        let rows = bind_all(sqlx::query(&query_str), &frag.binds)
            .fetch_all(&self.db)
            .await?;

        let mut users = Vec::with_capacity(rows.len());
        for row in &rows {
            let role_str: String = row
                .try_get("role")
                .unwrap_or_else(|_| "subscriber".into());
            let role: Role = role_str.parse().map_err(|_| {
                RepositoryError::InternalError("Invalid role in database".to_string())
            })?;

            users.push(UserSummary {
                id: row.try_get("id").map_err(internal)?,
                username: row.try_get("username").map_err(internal)?,
                email: row.try_get("email").map_err(internal)?,
                bio: row.try_get("bio").map_err(internal)?,
                image: row.try_get("image").map_err(internal)?,
                disabled: row.try_get::<bool, _>("disabled").unwrap_or(false),
                role,
                email_verified: row.try_get::<bool, _>("email_verified").unwrap_or(false),
                created_at: row.try_get("created_at").map_err(internal)?,
                article_count: row.try_get::<i32, _>("article_count").unwrap_or(0),
            });
        }

        Ok(users)
    }

    async fn count(&self, query: &AdminUsersQuery) -> RepoResult<i32> {
        let mut frag = Frag::default();

        if let Some(s) = &query.search
            && !s.trim().is_empty()
        {
            frag.binds.push(Val::Text(format!("%{}%", s.trim())));
            let n = frag.binds.len();
            frag.sql.push(format!(
                "(username ILIKE ${} OR email ILIKE ${})",
                n, n
            ));
        }

        match query.status.as_deref() {
            Some("active") => frag.push("disabled = FALSE", None),
            Some("disabled") => frag.push("disabled = TRUE", None),
            _ => {}
        }

        let query_str = if frag.is_empty() {
            "SELECT COUNT(*)::INT FROM users".to_string()
        } else {
            format!("SELECT COUNT(*)::INT FROM users WHERE {}", frag.join(" AND "))
        };

        let row = bind_all(sqlx::query(&query_str), &frag.binds)
            .fetch_one(&self.db)
            .await?;
        let count: i32 = row.try_get(0)?;
        Ok(count)

    }

    async fn list_profiles(
        &self,
        query: &ProfilesQuery,
        current_user_id: Option<Uuid>,
    ) -> RepoResult<Vec<UserProfile>> {
        let limit = query.limit.unwrap_or(20).min(100);
        let offset = query.offset.unwrap_or(0);

        // Only show users with Author or Admin role (not Subscribers)
        let mut clauses: Vec<String> = vec![
            "u.disabled = FALSE".to_string(),
            "(u.role = 'author' OR u.role = 'admin')".to_string(),
        ];
        let mut binds: Vec<Val> = Vec::new();

        let mut following_subquery = String::from("FALSE");

        if let Some(current_id) = current_user_id {
            binds.push(Val::Uuid(current_id));
            let n = binds.len();
            clauses.push(format!("u.id != ${}", n));
            // Reuse the same parameter for the following-status EXISTS
            following_subquery = format!(
                "EXISTS(SELECT 1 FROM user_follows WHERE follower_id = ${} AND following_id = u.id)",
                n
            );
        }

        if let Some(s) = &query.search
            && !s.trim().is_empty()
        {
            binds.push(Val::Text(format!("%{}%", s.trim())));
            let n = binds.len();
            clauses.push(format!("(u.username ILIKE ${} OR u.bio ILIKE ${})", n, n));
        }

        let limit_idx = binds.len() + 1;
        let offset_idx = limit_idx + 1;
        binds.push(Val::Int(limit as i64));
        binds.push(Val::Int(offset as i64));

        let query_str = format!(
            r"SELECT u.id, u.username, u.bio, u.image,
                     ({}) as is_following
              FROM users u
              WHERE {}
              ORDER BY u.username ASC
              LIMIT ${} OFFSET ${}",
            following_subquery,
            clauses.join(" AND "),
            limit_idx,
            offset_idx
        );

        let rows = bind_all(sqlx::query(&query_str), &binds)
            .fetch_all(&self.db)
            .await?;

        let mut profiles = Vec::with_capacity(rows.len());
        for row in &rows {
            let following = current_user_id.is_some()
                && row.try_get::<bool, _>("is_following").unwrap_or(false);

            profiles.push(UserProfile {
                username: row.try_get("username").map_err(internal)?,
                bio: row.try_get("bio").map_err(internal)?,
                image: row.try_get("image").map_err(internal)?,
                following,
            });
        }

        Ok(profiles)
    }

    async fn count_profiles(&self, query: &ProfilesQuery) -> RepoResult<i32> {
        let mut clauses: Vec<String> = vec![
            "disabled = FALSE".to_string(),
            "(role = 'author' OR role = 'admin')".to_string(),
        ];
        let mut binds: Vec<Val> = Vec::new();

        if let Some(s) = &query.search
            && !s.trim().is_empty()
        {
            binds.push(Val::Text(format!("%{}%", s.trim())));
            let n = binds.len();
            clauses.push(format!("(username ILIKE ${} OR bio ILIKE ${})", n, n));
        }

        let query_str = format!(
            "SELECT COUNT(*)::INT FROM users WHERE {}",
            clauses.join(" AND ")
        );

        let row = bind_all(sqlx::query(&query_str), &binds)
            .fetch_one(&self.db)
            .await?;
        let count: i32 = row.try_get(0)?;
        Ok(count)
    }

    async fn get_profile(
        &self,
        username: &str,
        current_user_id: Option<Uuid>,
    ) -> RepoResult<Option<UserProfile>> {
        let row = sqlx::query(
            "SELECT id, username, bio, image FROM users WHERE username = $1 AND disabled = FALSE",
        )
        .bind(username)
        .fetch_optional(&self.db)
        .await?;

        if let Some(row) = row {
            let user_id: Uuid = row.try_get("id").map_err(internal)?;

            let following = if let Some(current_id) = current_user_id {
                self.is_following(current_id, user_id).await?
            } else {
                false
            };

            Ok(Some(UserProfile {
                username: row.try_get("username").map_err(internal)?,
                bio: row.try_get("bio").map_err(internal)?,
                image: row.try_get("image").map_err(internal)?,
                following,
            }))
        } else {
            Ok(None)
        }
    }

    async fn update(&self, id: Uuid, data: &UpdateUser) -> RepoResult<User> {
        // Check if user exists
        if self.find_by_id(id).await?.is_none() {
            return Err(RepositoryError::NotFound(format!("User with id {}", id)));
        }

        let mut frag = Frag::default();

        if let Some(username) = &data.username {
            frag.push("username =", Some(Val::Text(username.clone())));
        }
        if let Some(email) = &data.email {
            frag.push("email =", Some(Val::Text(email.clone())));
        }
        if let Some(bio) = &data.bio {
            frag.push("bio =", Some(Val::Text(bio.clone())));
        }
        if let Some(image) = &data.image {
            frag.push("image =", Some(Val::Text(image.clone())));
        }
        if let Some(disabled) = data.disabled {
            frag.push("disabled =", Some(Val::Bool(disabled)));
        }
        if let Some(role) = &data.role {
            frag.push("role =", Some(Val::Text(role.to_string())));
        }
        if let Some(theme) = &data.theme_preference {
            frag.push("theme_preference =", Some(Val::Text(theme.clone())));
        }

        if frag.is_empty() {
            // Nothing to update, return existing user
            return self.find_by_id(id).await?.ok_or_else(|| {
                RepositoryError::InternalError("User disappeared during update".to_string())
            });
        }

        frag.sql.push("updated_at = NOW()".to_string());

        let id_idx = frag.binds.len() + 1;
        let update_query = format!(
            "UPDATE users SET {} WHERE id = ${}",
            frag.join(", "),
            id_idx
        );
        frag.binds.push(Val::Uuid(id));

        bind_all(sqlx::query(&update_query), &frag.binds)
            .execute(&self.db)
            .await?;

        self.find_by_id(id).await?.ok_or_else(|| {
            RepositoryError::InternalError("Failed to retrieve updated user".to_string())
        })
    }

    async fn update_password(&self, id: Uuid, password_hash: &str) -> RepoResult<()> {
        let result = sqlx::query(
            "UPDATE users SET password_hash = $1, password_changed_at = NOW(), updated_at = NOW() WHERE id = $2",
        )
        .bind(password_hash)
        .bind(id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!("User with id {}", id)));
        }

        Ok(())
    }

    async fn verify_email(&self, id: Uuid) -> RepoResult<()> {
        let result =
            sqlx::query("UPDATE users SET email_verified = TRUE, updated_at = NOW() WHERE id = $1")
                .bind(id)
                .execute(&self.db)
                .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!("User with id {}", id)));
        }

        Ok(())
    }

    async fn update_theme_preference(&self, id: Uuid, theme: &str) -> RepoResult<()> {
        let result =
            sqlx::query("UPDATE users SET theme_preference = $1, updated_at = NOW() WHERE id = $2")
                .bind(theme)
                .bind(id)
                .execute(&self.db)
                .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!("User with id {}", id)));
        }

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> RepoResult<()> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!("User with id {}", id)));
        }

        Ok(())
    }

    async fn follow(&self, follower_id: Uuid, following_id: Uuid) -> RepoResult<()> {
        sqlx::query(
            "INSERT INTO user_follows (follower_id, following_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(follower_id)
        .bind(following_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn unfollow(&self, follower_id: Uuid, following_id: Uuid) -> RepoResult<()> {
        sqlx::query("DELETE FROM user_follows WHERE follower_id = $1 AND following_id = $2")
            .bind(follower_id)
            .bind(following_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    async fn is_following(&self, follower_id: Uuid, following_id: Uuid) -> RepoResult<bool> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM user_follows WHERE follower_id = $1 AND following_id = $2)",
        )
        .bind(follower_id)
        .bind(following_id)
        .fetch_one(&self.db)
        .await?;

        Ok(exists)
    }

    async fn get_following(&self, user_id: Uuid) -> RepoResult<Vec<Uuid>> {
        let rows: Vec<Uuid> =
            sqlx::query_scalar("SELECT following_id FROM user_follows WHERE follower_id = $1")
                .bind(user_id)
                .fetch_all(&self.db)
                .await?;

        Ok(rows)
    }

    async fn get_followers(&self, user_id: Uuid) -> RepoResult<Vec<Uuid>> {
        let rows: Vec<Uuid> =
            sqlx::query_scalar("SELECT follower_id FROM user_follows WHERE following_id = $1")
                .bind(user_id)
                .fetch_all(&self.db)
                .await?;

        Ok(rows)
    }
}

