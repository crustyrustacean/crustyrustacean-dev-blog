# Repository Pattern Implementation Guide

This document describes the repository pattern implementation in the `crustyrustacean-dev-blog` codebase.

## Overview

The codebase uses a trait-based repository pattern to abstract database operations from HTTP route handlers. This separation provides:

- **Testability**: Mock repositories for unit testing without a database
- **Single Responsibility**: Routes handle HTTP, repositories handle data
- **Maintainability**: Database changes are localized to the repository layer
- **Consistency**: Common query patterns defined once, used everywhere

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    HTTP Layer (Routes)                       │
│  routes/articles.rs, routes/users.rs, routes/newsletters.rs  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                 Repository Traits                            │
│  UserRepository, ArticleRepository, NewsletterRepository     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              Repository Implementations                      │
│         LibSqlUserRepository, LibSqlArticleRepository        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Database Layer                            │
│              DatabaseConnection (libsql/Turso)               │
└─────────────────────────────────────────────────────────────┘
```

## Repository Traits

All repository traits are defined in `src/lib/repositories/` and use the `async_trait` macro for async methods.

### Core Repositories

| Repository | Trait File | Implementation | Purpose |
|------------|------------|----------------|---------|
| User | `user.rs` | `user_libsql.rs` | User accounts, profiles, follows |
| Article | `article.rs` | `article_libsql.rs` | Blog posts, drafts, favorites |
| Comment | `comment.rs` | `comment_libsql.rs` | Article comments |
| Tag | `tag.rs` | `tag_libsql.rs` | Article tags (many-to-many) |
| Category | `category.rs` | `category_libsql.rs` | Article categories |
| Media | `media.rs` | `media_libsql.rs` | Media library metadata |
| Newsletter | `newsletter.rs` | `newsletter_libsql.rs` | Subscribers, issues, delivery |
| ApiKey | `api_key.rs` | `api_key_libsql.rs` | API authentication keys |
| Token | `token.rs` | `token_libsql.rs` | Password reset, email verification |

### Example Trait Definition

```rust
use async_trait::async_trait;
use crate::repositories::{RepoResult, NewUser, UpdateUser, UserQuery};

#[async_trait]
pub trait UserRepository: Send + Sync {
    // Create
    async fn create(&self, user: NewUser) -> RepoResult<User>;

    // Read
    async fn find_by_id(&self, id: &Uuid) -> RepoResult<Option<User>>;
    async fn find_by_username(&self, username: &str) -> RepoResult<Option<User>>;
    async fn find_by_email(&self, email: &str) -> RepoResult<Option<User>>;
    async fn list(&self, query: UserQuery) -> RepoResult<Vec<User>>;

    // Update
    async fn update(&self, id: &Uuid, data: UpdateUser) -> RepoResult<User>;

    // Delete
    async fn delete(&self, id: &Uuid) -> RepoResult<()>;
}
```

## AppState Integration

Repositories are injected into `AppState` as trait objects:

```rust
pub struct AppState {
    // ... other fields
    pub users: Arc<dyn UserRepository>,
    pub articles: Arc<dyn ArticleRepository>,
    pub comments: Arc<dyn CommentRepository>,
    pub tags: Arc<dyn TagRepository>,
    pub categories: Arc<dyn CategoryRepository>,
    pub media: Arc<dyn MediaRepository>,
    pub newsletters: Arc<dyn NewsletterRepository>,
    pub api_keys: Arc<dyn ApiKeyRepository>,
    pub tokens: Arc<dyn TokenRepository>,
}
```

## Usage in Route Handlers

### Before (Direct Database Access)

```rust
pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<User>, AppError> {
    let conn = state.db.connect()?;

    let mut rows = conn.query(
        "SELECT id, username, email, ... FROM users WHERE id = ?",
        [user_id],
    ).await?;

    let row = rows.next().await?.ok_or(AppError::NotFound)?;
    let user = User {
        id: row.get::<String>(0)?,
        username: row.get::<String>(1)?,
        // ... manual field mapping
    };

    Ok(Json(user))
}
```

### After (Repository Pattern)

```rust
pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<User>, AppError> {
    let user = state.users
        .find_by_id(&user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(user))
}
```

## Error Handling

Repository operations return `RepoResult<T>`, which is an alias for `Result<T, RepositoryError>`.

### RepositoryError Variants

```rust
pub enum RepositoryError {
    /// Entity not found
    NotFound(String),

    /// Duplicate entry (unique constraint violation)
    AlreadyExists(String),

    /// Invalid input data
    InvalidInput(String),

    /// Database error
    Database(String),

    /// Unexpected error
    Internal(String),
}
```

### Conversion to HTTP Errors

`RepositoryError` automatically converts to `AppError` for route handlers:

```rust
impl From<RepositoryError> for AppError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound(msg) => AppError::NotFound,
            RepositoryError::AlreadyExists(msg) => AppError::Conflict(msg),
            RepositoryError::InvalidInput(msg) => AppError::BadRequest(msg),
            RepositoryError::Database(msg) => AppError::InternalError(msg),
            RepositoryError::Internal(msg) => AppError::InternalError(msg),
        }
    }
}
```

## Query Types

### NewXxx Structs

Used for creating new entities:

```rust
pub struct NewArticle {
    pub title: String,
    pub description: String,
    pub body: String,
    pub author_id: Uuid,
    pub category_id: Option<Uuid>,
    pub draft: bool,
    pub tags: Vec<String>,
}
```

### UpdateXxx Structs

Used for partial updates with `Option` fields:

```rust
pub struct UpdateArticle {
    pub title: Option<String>,
    pub description: Option<String>,
    pub body: Option<String>,
    pub category_id: Option<Option<Uuid>>,  // Double Option for nullable fields
    pub draft: Option<bool>,
    pub tags: Option<Vec<String>>,
}

impl Default for UpdateArticle {
    fn default() -> Self {
        Self {
            title: None,
            description: None,
            body: None,
            category_id: None,
            draft: None,
            tags: None,
        }
    }
}
```

### Query Structs

Used for filtering and pagination:

```rust
pub struct ArticleQuery {
    pub author_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub tag: Option<String>,
    pub draft: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
```

## Creating a New Repository

### 1. Define the Trait

Create `src/lib/repositories/my_entity.rs`:

```rust
use async_trait::async_trait;
use uuid::Uuid;
use crate::repositories::{RepoResult, RepositoryError};

// Data structures
pub struct MyEntity {
    pub id: Uuid,
    pub name: String,
}

pub struct NewMyEntity {
    pub name: String,
}

// Trait definition
#[async_trait]
pub trait MyEntityRepository: Send + Sync {
    async fn create(&self, entity: NewMyEntity) -> RepoResult<MyEntity>;
    async fn find_by_id(&self, id: &Uuid) -> RepoResult<Option<MyEntity>>;
    async fn delete(&self, id: &Uuid) -> RepoResult<()>;
}
```

### 2. Implement for LibSQL

Create `src/lib/repositories/my_entity_libsql.rs`:

```rust
use async_trait::async_trait;
use libsql::Connection;
use uuid::Uuid;

use super::{MyEntity, MyEntityRepository, NewMyEntity, RepoResult, RepositoryError};
use crate::DatabaseConnection;

pub struct LibSqlMyEntityRepository {
    db: DatabaseConnection,
}

impl LibSqlMyEntityRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl MyEntityRepository for LibSqlMyEntityRepository {
    async fn create(&self, entity: NewMyEntity) -> RepoResult<MyEntity> {
        let conn = self.db.connect()
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let id = Uuid::new_v4();

        conn.execute(
            "INSERT INTO my_entities (id, name) VALUES (?, ?)",
            libsql::params![id.to_string(), entity.name],
        ).await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(MyEntity { id, name: entity.name })
    }

    // ... other methods
}
```

### 3. Register in mod.rs

Add to `src/lib/repositories/mod.rs`:

```rust
mod my_entity;
mod my_entity_libsql;

pub use my_entity::*;
pub use my_entity_libsql::*;
```

### 4. Add to AppState

Update `src/lib/state.rs`:

```rust
pub struct AppState {
    // ... existing fields
    pub my_entities: Arc<dyn MyEntityRepository>,
}
```

## Best Practices

### 1. Keep Repositories Focused

Each repository should handle a single aggregate root. Avoid cross-repository operations within repository methods.

### 2. Use Transactions for Multi-Step Operations

For operations spanning multiple tables, handle transactions at the service layer or route handler level.

### 3. Return Domain Types

Repositories should return domain types, not database row types. Handle all field mapping inside the repository.

### 4. Handle NULL Properly

Use `Option<T>` for nullable database columns and handle `None` appropriately.

### 5. Validate in Repository

Basic validation (e.g., uniqueness checks) should happen in the repository. Business validation belongs in route handlers or services.

## Migration System

The codebase uses a versioned migration system in `src/lib/migrations/`:

- SQL files: `sql/001_create_users.sql`, `sql/002_create_articles.sql`, etc.
- Version tracking: `_migrations` table tracks applied migrations
- Idempotent: Uses `CREATE TABLE IF NOT EXISTS` and `CREATE INDEX IF NOT EXISTS`

See `src/lib/migrations/mod.rs` for the migration runner implementation.
