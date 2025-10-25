# CrustyRustacean Dev Blog - Codebase Index

**Generated:** 2025-10-25
**Version:** 1.8.0
**Repository:** https://github.com/crustyrustacean/crustyrustacean-dev-blog

---

## Table of Contents
1. [Project Overview](#project-overview)
2. [Technology Stack](#technology-stack)
3. [Architecture Overview](#architecture-overview)
4. [Module Index](#module-index)
5. [Route Handlers Reference](#route-handlers-reference)
6. [Data Models](#data-models)
7. [Database Schema](#database-schema)
8. [Testing Infrastructure](#testing-infrastructure)
9. [Frontend Assets](#frontend-assets)
10. [API Reference](#api-reference)
11. [Security Features](#security-features)
12. [Development Guide](#development-guide)

---

## Project Overview

A production-ready developer blog application built with modern Rust web technologies, demonstrating professional software architecture patterns including:
- Test-Driven Development (TDD)
- Domain-Driven Design (DDD)
- Comprehensive integration testing
- RESTful API design
- Modern responsive UI/UX

**Key Features:**
- User authentication with JWT tokens
- Article management with CRUD operations
- Comments system with authorization
- Favorites and social following
- Tag-based categorization
- Full-text search functionality
- RSS feed syndication
- Media library with cloud storage (OpenDAL)
- SEO optimization (sitemap, robots.txt)
- Admin dashboard
- Responsive Bootstrap UI

---

## Technology Stack

### Backend Core
| Technology | Version | Purpose |
|------------|---------|---------|
| Axum | 0.8.6 | High-performance async web framework |
| Tokio | 1.47.1 | Async runtime |
| Tower-HTTP | 0.6.6 | HTTP middleware (CORS, tracing, security) |
| Shuttle | 0.57.0 | Deployment platform |

### Database & Persistence
| Technology | Version | Purpose |
|------------|---------|---------|
| LibSQL/Turso | 0.9.24 | SQLite-compatible edge database |
| Serde/Serde_json | 1.0 | Serialization/deserialization |

### Authentication & Security
| Technology | Version | Purpose |
|------------|---------|---------|
| Argon2 | 0.5.3 | Password hashing |
| jsonwebtoken | 10.1.0 | JWT token operations |
| Validator | 0.20.0 | Input validation |

### Frontend & Content
| Technology | Version | Purpose |
|------------|---------|---------|
| Tera | 1.20.0 | Jinja-like templating engine |
| Pulldown-cmark | 0.13.0 | Markdown to HTML conversion |
| Bootstrap | 5.x | CSS framework |

### Storage & Media
| Technology | Version | Purpose |
|------------|---------|---------|
| OpenDAL | 0.51.0 | Unified data access layer for cloud storage |
| UUID | 1.11.0 | Unique identifier generation |

### Observability
| Technology | Version | Purpose |
|------------|---------|---------|
| Tracing | 0.1.41 | Structured logging framework |
| Tracing-bunyan-formatter | 0.3.10 | JSON-formatted logs |

---

## Architecture Overview

### Application Structure

```
┌─────────────────────────────────────────────────────────┐
│                    HTTP Request                          │
└───────────────────┬─────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────┐
│                 Middleware Layers                        │
│  - Request ID tracking                                   │
│  - Structured logging (Bunyan JSON)                      │
│  - CORS handling                                         │
│  - Security headers                                      │
└───────────────────┬─────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────┐
│                   Router Layer                           │
│  - Route matching                                        │
│  - Method routing                                        │
│  - Static file serving                                   │
└───────────────────┬─────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────┐
│                 Route Handlers                           │
│  - Authentication extractors                             │
│  - Input validation                                      │
│  - Business logic                                        │
└───────────────────┬─────────────────────────────────────┘
                    │
        ┌───────────┴───────────┐
        ▼                       ▼
┌──────────────┐      ┌──────────────────┐
│   Database   │      │  Tera Templates  │
│  Operations  │      │    Rendering     │
└──────────────┘      └──────────────────┘
```

### Entry Point Flow

**src/bin/main.rs** → Shuttle entry point
  ↓
**Database initialization** → Migrations
  ↓
**AppState creation** → Tera + DB + JWT keys
  ↓
**Router setup** → startup.rs
  ↓
**Middleware layers** → Tower-HTTP
  ↓
**Route registration** → All endpoints
  ↓
**Server start** → TcpListener

---

## Module Index

### Core Infrastructure Modules

#### **src/lib/lib.rs** (30 lines)
**Purpose:** Library root with module declarations and re-exports

**Exports:**
- `auth` - Authentication and JWT operations
- `config` - Application configuration
- `database` - Database connection and migrations
- `errors` - Unified error types
- `markdown` - Markdown processing
- `models` - Data structures (User, Article, Comment, Tag, Media)
- `response` - Standardized API responses
- `routes` - HTTP route handlers
- `startup` - Application initialization
- `state` - Shared application state
- `storage` - OpenDAL storage integration
- `telemetry` - Logging and tracing

---

#### **src/lib/auth.rs** (166 lines)
**Purpose:** JWT authentication and password hashing

**Key Structures:**
- `Keys` - JWT encoding/decoding keys (RS256)
- `Claims` - JWT token payload (user_id, username, exp)
- `AuthenticatedUser` - Axum extractor for protected routes
- `OptionalUser` - Axum extractor for optional auth

**Key Functions:**
- `hash_password(password: &str) -> Result<String>` - Argon2 hashing
- `verify_password(password: &str, hash: &str) -> Result<bool>` - Password verification
- `generate_jwt(user: &User, keys: &Keys) -> Result<String>` - Token generation
- `validate_jwt(token: &str, keys: &Keys) -> Result<Claims>` - Token validation

**Security Features:**
- Argon2 with default parameters (memory-hard)
- RS256 JWT signing
- 24-hour token expiration
- PEM key support

**Location:** `src/lib/auth.rs`

---

#### **src/lib/database.rs** (176 lines)
**Purpose:** Database connection wrapper and schema migrations

**Key Structures:**
- `DatabaseConnection` - Wrapper around libsql::Database

**Schema Tables:**
1. **users** - User accounts
   - Columns: id, username, email, password_hash, bio, image_url, created_at, updated_at
   - Unique: username, email

2. **articles** - Blog posts
   - Columns: id, slug, title, description, body, author_id, created_at, updated_at
   - Unique: slug
   - Index: author_id

3. **comments** - Article comments
   - Columns: id, article_id, author_id, body, created_at, updated_at
   - Index: article_id

4. **tags** - Tag catalog
   - Columns: id, name
   - Unique: name

5. **article_tags** - Article-tag relationships
   - Columns: article_id, tag_id
   - Primary key: (article_id, tag_id)

6. **user_favorites** - User favorite articles
   - Columns: user_id, article_id
   - Primary key: (user_id, article_id)

7. **user_follows** - User follow relationships
   - Columns: follower_id, followee_id
   - Primary key: (follower_id, followee_id)

8. **media_library** - Uploaded media files
   - Columns: id, user_id, filename, storage_path, title, alt_text, caption, description
   - Technical: mime_type, file_size, width, height
   - Timestamps: uploaded_at, updated_at
   - Foreign key: user_id references users(id)

**Key Functions:**
- `new(url: &str, auth_token: &str) -> Result<Self>` - Create connection
- `run_migrations() -> Result<()>` - Execute schema migrations

**Location:** `src/lib/database.rs`

---

#### **src/lib/errors.rs** (79 lines)
**Purpose:** Unified error handling with HTTP status mapping

**Error Variants:**
- `BadRequest(String)` → 400
- `Unauthorized(String)` → 401
- `Forbidden(String)` → 403
- `NotFound(String)` → 404
- `Conflict(String)` → 409
- `UnprocessableEntity(Vec<String>)` → 422
- `InternalServerError(String)` → 500
- `TemplateError(tera::Error)` → 500

**Features:**
- Automatic conversion to HTTP responses
- JSON error format for API endpoints
- Implements From<T> for common error types
- Integration with Axum's IntoResponse

**Location:** `src/lib/errors.rs`

---

#### **src/lib/startup.rs** (143 lines)
**Purpose:** Application initialization and router configuration

**Key Structure:**
- `App { config, router }` - Main application type

**Router Configuration:**
1. **HTML Routes** (14 endpoints)
   - Homepage, articles, profiles, admin, auth pages

2. **API Routes** (20+ endpoints)
   - Users, profiles, articles, comments, tags, favorites

3. **Content Routes**
   - RSS feed, sitemap, robots.txt

4. **Static Files**
   - /static/* → ServeDir

**Middleware Stack (bottom to top):**
1. Security headers (X-Content-Type-Options, X-Frame-Options, HSTS)
2. Structured tracing with Bunyan JSON formatter
3. Request ID propagation
4. Request ID generation (UUID v4)
5. CORS (permissive)

**Location:** `src/lib/startup.rs:47-136`

---

#### **src/lib/state.rs**
**Purpose:** Shared application state

**Structure:**
```rust
#[derive(Clone)]
pub struct AppState {
    pub tera: Arc<Tera>,           // Template engine
    pub db: DatabaseConnection,     // Database connection
    pub keys: Keys,                 // JWT signing keys
}
```

**Usage:** Injected into all route handlers via Axum's State extractor

**Location:** `src/lib/state.rs`

---

#### **src/lib/config.rs**
**Purpose:** Application configuration management

**Structure:**
```rust
pub struct AppConfig {
    pub jwt_secret: String,
}
```

**Methods:**
- `jwt_secret_bytes() -> Vec<u8>` - Convert secret to bytes for key generation
- `Default` implementation for testing

**Location:** `src/lib/config.rs`

---

#### **src/lib/telemetry.rs** (51 lines)
**Purpose:** Structured logging and tracing setup

**Key Components:**
- `MakeRequestUuid` - UUID v4 generator for request IDs
- Bunyan JSON formatter for production logging
- Environment-based log level filtering

**Functions:**
- `get_subscriber() -> impl Subscriber` - Creates tracing subscriber with JSON formatting

**Location:** `src/lib/telemetry.rs`

---

#### **src/lib/markdown.rs** (113 lines)
**Purpose:** Markdown to HTML conversion with advanced features

**Features Enabled:**
- Strikethrough (~~text~~)
- Tables
- Footnotes
- Task lists (- [ ] / - [x])
- Smart punctuation (quotes, dashes)
- Heading attributes
- Metadata blocks
- Old footnotes compatibility

**Key Function:**
- `markdown_to_html(content: &str) -> String` - Convert markdown to safe HTML

**Location:** `src/lib/markdown.rs`

---

#### **src/lib/response.rs** (63 lines)
**Purpose:** Standardized API response types

**Structures:**
- `ApiResponse<T>` - Success responses with data
- `ErrorResponse` - Error responses with messages
- `ValidationError` - Input validation failures

**Features:**
- Consistent JSON format
- Generic type support
- Serde serialization

**Location:** `src/lib/response.rs`

---

### Data Models

#### **src/lib/models/user.rs** (88 lines)

**Structures:**

1. **User** - Database entity
   - Fields: id, username, email, password_hash, bio, image_url, created_at, updated_at
   - Methods: `new()`, database queries

2. **UserProfile** - Public profile
   - Fields: username, bio, image_url, following
   - Used in API responses

3. **UserRegistration** - Registration input
   - Validation: email format, username/password length
   - Derives: Deserialize, Validate

4. **UserLogin** - Login credentials
   - Fields: email, password
   - Validation: email format, required fields

5. **UserUpdate** - Profile update input
   - Optional fields: username, email, password, bio, image_url

**Location:** `src/lib/models/user.rs`

---

#### **src/lib/models/article.rs** (84 lines)

**Structures:**

1. **Article** - Database entity
   - Fields: id, slug, title, description, body, author_id, created_at, updated_at
   - Automatic slug generation

2. **ArticleResponse** - API response format
   - Includes: article data, author profile, tags, favorites count, favorited status

3. **CreateArticle** - Creation input
   - Validation: title/description/body required
   - Fields: title, description, body, tag_list

4. **UpdateArticle** - Update input
   - Optional fields: title, description, body, tag_list

**Business Logic:**
- Automatic slug generation from title
- Slug uniqueness with counter suffix
- Tag association on creation/update

**Location:** `src/lib/models/article.rs`

---

#### **src/lib/models/comment.rs** (43 lines)

**Structures:**

1. **Comment** - Database entity
   - Fields: id, article_id, author_id, body, created_at, updated_at

2. **CommentResponse** - API response
   - Includes: comment data, author profile

3. **CreateComment** - Creation input
   - Validation: body required, non-empty

**Location:** `src/lib/models/comment.rs`

---

#### **src/lib/models/tag.rs** (46 lines)

**Structures:**

1. **Tag** - Tag entity
   - Fields: id, name

2. **ArticleTag** - Article-tag junction
   - Fields: article_id, tag_id

3. **UserFavorite** - Favorite tracking
   - Fields: user_id, article_id

4. **UserFollow** - Follow relationships
   - Fields: follower_id, followee_id

**Location:** `src/lib/models/tag.rs`

---

#### **src/lib/models/media.rs** (78 lines)

**Structures:**

1. **Media** - Media file entity
   - Fields: id, user_id, filename, storage_path, title, alt_text, caption, description
   - Technical fields: mime_type, file_size, width, height, uploaded_at, updated_at

2. **MediaResponse** - API response format
   - Includes all media fields with proper serialization

3. **UpdateMedia** - Metadata update input
   - Optional fields: title, alt_text, caption, description

4. **MediaQuery** - List query parameters
   - Pagination: limit, offset
   - Filtering: mime_type

5. **SingleMediaResponse** - Single media response wrapper
   - Format: `{"media": {...}}`

6. **MultipleMediaResponse** - Media list response
   - Format: `{"media": [...], "media_count": N}`

**Location:** `src/lib/models/media.rs`

---

#### **src/lib/storage.rs** (120 lines)

**Purpose:** OpenDAL integration for cloud storage

**Key Structures:**
- `StorageService` - Wrapper around OpenDAL operator
- `FileMetadata` - File size and content type information

**Key Functions:**
- `new() -> Result<Self>` - Initialize storage backend
- `upload(path: &str, data: Vec<u8>, content_type: Option<String>) -> Result<FileMetadata>` - Upload file
- `download(path: &str) -> Result<Vec<u8>>` - Download file
- `delete(path: &str) -> Result<()>` - Delete file
- `generate_media_path(user_id: &str, filename: &str) -> String` - Generate storage path

**Storage Configuration:**
- Configurable backend (S3, local filesystem, etc.)
- Automatic path organization by user
- File metadata tracking

**Location:** `src/lib/storage.rs`

---

### Route Handlers

#### **src/lib/routes/users.rs** (538 lines)
**Purpose:** User authentication and profile management

**Endpoints:**

1. **POST /api/users** - `register_user()`
   - Validates input (email, username, password)
   - Hashes password with Argon2
   - Creates user record
   - Returns JWT token

2. **POST /api/users/login** - `login_user()`
   - Validates credentials
   - Verifies password
   - Generates JWT token
   - Returns user data + token

3. **GET /api/user** - `get_current_user()`
   - Protected: requires JWT
   - Returns current user profile

4. **PUT /api/user** - `update_current_user()`
   - Protected: requires JWT
   - Updates user profile fields
   - Re-hashes password if changed

5. **GET /api/profiles** - `list_profiles()`
   - Protected: requires JWT
   - Search by username/bio
   - Pagination support
   - Includes follow status

6. **GET /api/profiles/{username}** - `get_profile()`
   - Public profile view
   - Includes follow status (if authenticated)

7. **POST /api/profiles/{username}/follow** - `follow_user()`
   - Protected: requires JWT
   - Creates follow relationship
   - Prevents self-following

8. **DELETE /api/profiles/{username}/follow** - `unfollow_user()`
   - Protected: requires JWT
   - Removes follow relationship

**Location:** `src/lib/routes/users.rs`

---

#### **src/lib/routes/articles.rs** (1,578 lines)
**Purpose:** Article CRUD operations and listing

**Endpoints:**

1. **POST /api/articles** - `create_article()`
   - Protected: requires JWT
   - Validates input (title, description, body)
   - Generates unique slug from title
   - Associates tags
   - Converts markdown to HTML
   - Returns article response

2. **GET /api/articles** - `list_articles()`
   - Public endpoint
   - Filtering: tag, author, favorited
   - Pagination: limit, offset
   - Includes favorites count
   - Returns article list + count

3. **GET /api/articles/{slug}** - `get_article()`
   - Public endpoint
   - Returns single article with:
     - Author profile
     - Tags
     - Favorites count
     - Favorited status (if authenticated)

4. **PUT /api/articles/{slug}** - `update_article()`
   - Protected: requires JWT
   - Authorization: must be author
   - Updates title, description, body, tags
   - Re-generates slug if title changed

5. **DELETE /api/articles/{slug}** - `delete_article()`
   - Protected: requires JWT
   - Authorization: must be author
   - Cascades to comments, tags, favorites

6. **POST /api/articles/{slug}/favorite** - `favorite_article()`
   - Protected: requires JWT
   - Idempotent operation
   - Returns updated article

7. **DELETE /api/articles/{slug}/favorite** - `unfavorite_article()`
   - Protected: requires JWT
   - Idempotent operation
   - Returns updated article

8. **GET /api/articles/feed** - `get_articles_feed()`
   - Protected: requires JWT
   - Returns articles from followed authors
   - Pagination support
   - Ordered by created_at DESC

**HTML Page Handlers:**

9. **GET /articles** - `get_articles_list_page()`
   - Renders articles listing template
   - Bootstrap grid layout
   - Tag filtering UI
   - Pagination controls

10. **GET /articles/{slug}** - `get_article_page()`
    - Renders individual article template
    - Markdown to HTML conversion
    - Comments section
    - Favorite button (if authenticated)

11. **GET /editor** - `get_editor_page()`
    - Protected: requires JWT
    - Article creation form
    - Markdown editor
    - Tag input

12. **GET /editor/{slug}** - `get_edit_article_page()`
    - Protected: requires JWT
    - Authorization: must be author
    - Pre-populated edit form

13. **GET /feed** - `get_articles_feed_page()`
    - Protected: requires JWT
    - Personal feed template
    - Articles from followed authors

**Location:** `src/lib/routes/articles.rs`

---

#### **src/lib/routes/comments.rs** (284 lines)
**Purpose:** Comments system with authorization

**Endpoints:**

1. **POST /api/articles/{slug}/comments** - `add_comment()`
   - Protected: requires JWT
   - Validates: article exists, body non-empty
   - Creates comment record
   - Returns comment with author profile

2. **GET /api/articles/{slug}/comments** - `get_comments()`
   - Public endpoint
   - Returns all comments for article
   - Includes author profiles
   - Ordered by created_at DESC

3. **DELETE /api/articles/{slug}/comments/{id}** - `delete_comment()`
   - Protected: requires JWT
   - Authorization: must be comment author
   - Removes comment record

**Location:** `src/lib/routes/comments.rs`

---

#### **src/lib/routes/tags.rs** (219 lines)
**Purpose:** Tag management API

**Endpoints:**

1. **GET /api/tags** - `get_tags()`
   - Public endpoint
   - Returns all tags alphabetically
   - Format: `{"tags": ["tag1", "tag2"]}`

2. **PUT /api/tags/{name}** - `update_tag()`
   - Protected: requires JWT
   - Renames tag across all articles
   - Returns updated tag

3. **DELETE /api/tags/{name}** - `delete_tag()`
   - Protected: requires JWT
   - Removes tag and associations
   - Cascades to article_tags

**HTML Page Handlers:**

4. **GET /admin/tags** - `get_tags_admin_page()`
   - Protected: requires JWT
   - Tag management interface
   - Rename/delete operations

**Location:** `src/lib/routes/tags.rs`

---

#### **src/lib/routes/profile.rs** (264 lines)
**Purpose:** User profile and discovery pages

**Endpoints:**

1. **GET /profiles/{username}** - `get_profile_page()`
   - Public profile view
   - User's articles
   - Follow button (if authenticated)
   - Bio and metadata

2. **GET /profiles** - `get_authors_page()`
   - Protected: requires JWT
   - Browse all authors
   - Search functionality
   - Follow/unfollow actions
   - Pagination

3. **GET /favorites** - `get_my_favorites_page()`
   - Protected: requires JWT
   - User's favorited articles
   - Unfavorite actions
   - Responsive grid

**Location:** `src/lib/routes/profile.rs`

---

#### **src/lib/routes/index.rs** (323 lines)
**Purpose:** Dynamic homepage

**Endpoint:**

1. **GET /** - `get_index()`
   - Dynamic article listing
   - Blog statistics:
     - Total article count
     - Number of topics (tags)
     - Latest post date
   - Tag cloud sidebar
   - Featured articles
   - Pagination
   - Responsive design

**Template Context:**
- `articles` - Recent articles
- `total_count` - Total articles
- `topics_count` - Unique tags
- `latest_date` - Most recent post
- `tags` - All tags for sidebar
- `current_page` - Pagination state

**Location:** `src/lib/routes/index.rs`

---

#### **src/lib/routes/rss.rs** (153 lines)
**Purpose:** RSS 2.0 feed generation

**Endpoint:**

1. **GET /rss** - `get_rss_feed()`
   - Standards-compliant RSS 2.0
   - Latest 20 articles
   - XML escaping for security
   - RFC 2822 date formatting
   - Channel metadata
   - Per-item descriptions

**Feed Structure:**
```xml
<rss version="2.0">
  <channel>
    <title>CrustyRustacean Dev Blog</title>
    <link>...</link>
    <description>...</description>
    <item>...</item>
  </channel>
</rss>
```

**Location:** `src/lib/routes/rss.rs`

---

#### **src/lib/routes/sitemap.rs** (146 lines)
**Purpose:** XML sitemap for SEO

**Endpoint:**

1. **GET /sitemap.xml** - `get_sitemap()`
   - Dynamic sitemap generation
   - Includes all articles
   - Priority and changefreq
   - Last modification dates

**Location:** `src/lib/routes/sitemap.rs`

---

#### **src/lib/routes/robots.rs** (15 lines)
**Purpose:** Robots.txt for web crawlers

**Endpoint:**

1. **GET /robots.txt** - `get_robots_txt()`
   - Allows all crawlers
   - Sitemap reference

**Location:** `src/lib/routes/robots.rs`

---

#### **src/lib/routes/auth.rs** (96 lines)
**Purpose:** Authentication page rendering

**Endpoints:**

1. **GET /login** - `get_login_page()`
   - Login form template
   - Bootstrap styling
   - Error handling

2. **GET /register** - `get_register_page()`
   - Registration form
   - Validation messages
   - Terms acceptance

**Location:** `src/lib/routes/auth.rs`

---

#### **src/lib/routes/error_pages.rs** (46 lines)
**Purpose:** Error page handlers

**Endpoints:**

1. **Fallback** - `handle_404_simple()`
   - 404 Not Found page
   - Helpful navigation links
   - Custom styling

**Location:** `src/lib/routes/error_pages.rs`

---

#### **src/lib/routes/health_check.rs** (9 lines)
**Purpose:** System health monitoring

**Endpoint:**

1. **GET /health_check** - `health_check()`
   - Returns HTTP 200 OK
   - Used by monitoring systems
   - No authentication required

**Location:** `src/lib/routes/health_check.rs`

---

#### **src/lib/routes/search.rs**
**Purpose:** Full-text search across articles

**Endpoints:**

1. **GET /api/search** - `search_articles()`
   - Public endpoint
   - Query param: q (search query)
   - Searches across title, description, and body
   - Returns matching articles with author profiles
   - Uses SQL LIKE pattern matching
   - Includes metadata and tags

**HTML Page Handlers:**

2. **GET /search** - `get_search_page()`
   - Search results page
   - Displays matching articles
   - Empty state for no results
   - Responsive grid layout

**Location:** `src/lib/routes/search.rs`

---

#### **src/lib/routes/media.rs** (720 lines)
**Purpose:** Media library management with cloud storage

**Endpoints:**

1. **POST /api/media** - `upload_media()`
   - Protected: requires JWT
   - Multipart form upload
   - Fields: file, title, alt_text, caption, description
   - Validates: image files only, 50MB limit
   - Generates unique filename (UUID)
   - Uploads to OpenDAL storage
   - Creates database record
   - Returns media response

2. **GET /api/media** - `list_media()`
   - Protected: requires JWT
   - Query params: limit, offset, mime_type
   - Returns user's media files
   - Pagination support
   - Includes total count

3. **GET /api/media/:id** - `get_media_metadata()`
   - Protected: requires JWT
   - Returns single media metadata
   - Ownership verification

4. **PUT /api/media/:id** - `update_media_metadata()`
   - Protected: requires JWT
   - Authorization: must be owner
   - Updates: title, alt_text, caption, description
   - Returns updated media

5. **DELETE /api/media/:id** - `delete_media()`
   - Protected: requires JWT
   - Authorization: must be owner
   - Deletes from storage (OpenDAL)
   - Deletes database record
   - Returns 204 No Content

6. **GET /api/media/:id/download** - `download_media()`
   - Public endpoint
   - Downloads file from storage
   - Returns file with proper headers
   - Content-Type and Content-Disposition

**HTML Page Handlers:**

7. **GET /admin/media** - `get_media_library_page()`
   - Protected: requires JWT
   - Media library admin interface
   - Upload and management UI
   - Grid display of media

**Security Features:**
- File type validation (images only)
- File size limits (50MB)
- User-based access control
- Ownership verification for all operations

**Location:** `src/lib/routes/media.rs`

---

## Database Schema

### Entity Relationship Diagram

```
┌──────────────┐         ┌──────────────┐
│    users     │◄────┐   │   articles   │
│─────────────│     │   │──────────────│
│ id (PK)      │     └───┤ author_id(FK)│
│ username (U) │         │ slug (U)     │
│ email (U)    │         │ title        │
│ password_hash│         │ description  │
│ bio          │         │ body         │
│ image_url    │         └──────┬───────┘
│ created_at   │                │
│ updated_at   │         ┌──────┴───────────┐
└──────┬───────┘         │                  │
       │           ┌─────▼─────┐     ┌──────▼──────┐
       │           │  comments │     │ article_tags│
       │           │───────────│     │─────────────│
       │           │article_id │     │article_id(FK)│
       │           │author_id  │     │tag_id (FK)  │
       │           │body       │     └──────┬──────┘
       │           └───────────┘            │
       │                              ┌─────▼────┐
       │                              │   tags   │
       ├────────────┐                 │──────────│
       │            │                 │id (PK)   │
 ┌─────▼────┐  ┌───▼──────┐         │name (U)  │
 │user_follows│ │user_     │         └──────────┘
 │            │ │favorites │
 │follower_id │ │user_id   │
 │followee_id │ │article_id│
 └────────────┘ └──────────┘
```

### Table Details

#### **users**
```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    bio TEXT,
    image_url TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

#### **articles**
```sql
CREATE TABLE articles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    body TEXT NOT NULL,
    author_id INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (author_id) REFERENCES users(id)
);
CREATE INDEX idx_articles_author ON articles(author_id);
```

#### **comments**
```sql
CREATE TABLE comments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    article_id INTEGER NOT NULL,
    author_id INTEGER NOT NULL,
    body TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (article_id) REFERENCES articles(id),
    FOREIGN KEY (author_id) REFERENCES users(id)
);
CREATE INDEX idx_comments_article ON comments(article_id);
```

#### **tags**
```sql
CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);
```

#### **article_tags**
```sql
CREATE TABLE article_tags (
    article_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (article_id, tag_id),
    FOREIGN KEY (article_id) REFERENCES articles(id),
    FOREIGN KEY (tag_id) REFERENCES tags(id)
);
```

#### **user_favorites**
```sql
CREATE TABLE user_favorites (
    user_id INTEGER NOT NULL,
    article_id INTEGER NOT NULL,
    PRIMARY KEY (user_id, article_id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (article_id) REFERENCES articles(id)
);
```

#### **user_follows**
```sql
CREATE TABLE user_follows (
    follower_id INTEGER NOT NULL,
    followee_id INTEGER NOT NULL,
    PRIMARY KEY (follower_id, followee_id),
    FOREIGN KEY (follower_id) REFERENCES users(id),
    FOREIGN KEY (followee_id) REFERENCES users(id)
);
```

#### **media_library**
```sql
CREATE TABLE media_library (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    storage_path TEXT NOT NULL,
    title TEXT,
    alt_text TEXT,
    caption TEXT,
    description TEXT,
    mime_type TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    width INTEGER,
    height INTEGER,
    uploaded_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
CREATE INDEX idx_media_user ON media_library(user_id);
```

---

## Testing Infrastructure

### Test Organization

**Location:** `tests/api/`
**Total Lines:** ~4,777
**Total Tests:** 81+

### Test Helpers (`tests/api/helpers.rs` - 351 lines)

**Key Components:**

1. **TestApp** - Test application instance
   ```rust
   pub struct TestApp {
       pub address: String,
       pub port: u16,
       pub client: reqwest::Client,
   }
   ```

2. **spawn_app()** - Creates test instance
   - Temporary SQLite database
   - Random available port
   - Full middleware stack
   - Async HTTP client

3. **Test Traits**
   - `TestUserBuilder` - User creation fixtures
   - `TestArticleBuilder` - Article fixtures

4. **Test Macros**
   - `assert_status!(response, expected_status)` - Status assertions
   - `bearer_request!(client, url, token)` - Authenticated requests
   - `parse_json!(response) -> T` - JSON parsing

**LazyLock Tracing:** One-time initialization prevents test pollution

### Test Coverage by Module

| Test File | LOC | Tests | Coverage |
|-----------|-----|-------|----------|
| articles.rs | ~699 | 25+ | Article CRUD, pagination, filtering, authorization |
| tags.rs | ~478 | 20+ | Tag retrieval, sorting, admin operations |
| favorites.rs | ~437 | 18+ | Favorite/unfavorite, listing, edge cases |
| comments.rs | ~401 | 20+ | Comment CRUD, authorization, validation |
| auth.rs | ~381 | 15+ | Registration, login, token validation |
| feed.rs | ~232 | 15+ | Personal feed with follows |
| authors.rs | ~215 | 15+ | Author discovery, search, pagination |
| rss.rs | ~205 | 10+ | RSS generation, XML validation |
| sitemap.rs | ~330 | 12+ | Sitemap generation |
| admin_dashboard.rs | ~232 | 12+ | Admin functionality |
| template_rendering.rs | ~432 | 15+ | Template compilation |
| markdown_integration.rs | ~173 | 12+ | Markdown processing |
| feed_page.rs | ~80 | 8+ | Feed page rendering |
| robots.rs | ~80 | 5+ | Robots.txt generation |
| health_check.rs | - | 5+ | Health endpoint |

### Testing Patterns

1. **AAA Pattern:** Arrange → Act → Assert
2. **Test Isolation:** Each test spawns fresh app
3. **Multi-user Scenarios:** Tests with multiple users
4. **Edge Cases:** Empty states, invalid inputs
5. **Authorization Tests:** Permission validation
6. **Idempotency Tests:** Repeated operations
7. **Integration Tests:** Full stack testing

### Running Tests

```bash
# All tests
cargo test

# Specific module
cargo test articles
cargo test auth
cargo test favorites

# Verbose output
TEST_LOG=true cargo test

# Single test
cargo test test_create_article
```

---

## Frontend Assets

### JavaScript Architecture (17 modules)

**Location:** `static/js/`

| File | Purpose | Dependencies |
|------|---------|--------------|
| **base.js** | Global utilities, API helpers, auth state | None (base) |
| **utils.js** | Shared utility functions | None |
| **auth.js** | Login/register form handling | base.js |
| **homepage.js** | Homepage dynamic features | base.js |
| **article-init.js** | Article page initialization | base.js |
| **article-list.js** | Articles listing interactions | base.js |
| **article-page.js** | Individual article features | base.js, comments.js |
| **article.js** | General article functions | base.js |
| **editor.js** | Article editor (create/edit) | base.js |
| **comments.js** | Comments system | base.js |
| **favorites.js** | Favorites functionality | base.js |
| **profile.js** | Profile page interactions | base.js |
| **authors.js** | Authors discovery | base.js |
| **feed.js** | Personal feed page | base.js |
| **admin.js** | Admin dashboard | base.js |
| **tags-admin.js** | Tag management | base.js |
| **media-library.js** | Media library management | base.js |
| **search.js** | Search functionality | base.js |
| **error404.js** | 404 page enhancements | None |

### Template Structure (22 files)

**Location:** `templates/`

**Base Template:** `base.html`
- HTML5 structure
- Bootstrap integration
- Navigation bar
- Footer
- Block system for content

**Template Inheritance:**
```
base.html
  ├── index.html (homepage)
  ├── articles/
  │   ├── list.html (articles listing)
  │   ├── article.html (article view)
  │   ├── editor.html (create/edit)
  │   └── feed.html (personal feed)
  ├── auth/
  │   ├── login.html
  │   └── register.html
  ├── profile/
  │   ├── profile.html
  │   ├── favorites.html
  │   └── authors.html
  ├── admin/
  │   ├── dashboard.html
  │   └── tags.html
  ├── media/
  │   └── library.html (media library admin)
  └── errors/
      └── 404.html
```

### CSS Architecture

**Location:** `static/css/styles.css`

**Components:**
- Bootstrap integration
- Custom theme variables
- Responsive utilities
- Component-specific styles
- Mobile-first approach

---

## API Reference

### Authentication Flow

1. **Register:**
   ```http
   POST /api/users
   Content-Type: application/json

   {
     "user": {
       "username": "john",
       "email": "john@example.com",
       "password": "securepassword"
     }
   }
   ```

   Response:
   ```json
   {
     "user": {
       "username": "john",
       "email": "john@example.com",
       "token": "eyJ..."
     }
   }
   ```

2. **Login:**
   ```http
   POST /api/users/login
   Content-Type: application/json

   {
     "user": {
       "email": "john@example.com",
       "password": "securepassword"
     }
   }
   ```

3. **Authenticated Requests:**
   ```http
   GET /api/user
   Authorization: Bearer eyJ...
   ```

### Complete Endpoint List

#### Authentication & Users
- `POST /api/users` - Register
- `POST /api/users/login` - Login
- `GET /api/user` - Get current user (protected)
- `PUT /api/user` - Update profile (protected)

#### Profiles
- `GET /api/profiles` - List all profiles (protected)
- `GET /api/profiles/{username}` - Get profile
- `POST /api/profiles/{username}/follow` - Follow (protected)
- `DELETE /api/profiles/{username}/follow` - Unfollow (protected)

#### Articles
- `POST /api/articles` - Create (protected)
- `GET /api/articles` - List with filters
- `GET /api/articles/feed` - Personal feed (protected)
- `GET /api/articles/{slug}` - Get single
- `PUT /api/articles/{slug}` - Update (protected, author only)
- `DELETE /api/articles/{slug}` - Delete (protected, author only)

#### Favorites
- `POST /api/articles/{slug}/favorite` - Favorite (protected)
- `DELETE /api/articles/{slug}/favorite` - Unfavorite (protected)

#### Comments
- `POST /api/articles/{slug}/comments` - Add (protected)
- `GET /api/articles/{slug}/comments` - List
- `DELETE /api/articles/{slug}/comments/{id}` - Delete (protected, author only)

#### Tags
- `GET /api/tags` - List all tags
- `PUT /api/tags/{name}` - Update (protected)
- `DELETE /api/tags/{name}` - Delete (protected)

#### Search
- `GET /api/search` - Search articles by query
- `GET /search` - Search results page

#### Media Library
- `POST /api/media` - Upload media (protected)
- `GET /api/media` - List user's media (protected)
- `GET /api/media/:id` - Get media metadata (protected)
- `PUT /api/media/:id` - Update media metadata (protected)
- `DELETE /api/media/:id` - Delete media (protected)
- `GET /api/media/:id/download` - Download media file

#### Content
- `GET /rss` - RSS feed
- `GET /sitemap.xml` - Sitemap
- `GET /robots.txt` - Robots

#### System
- `GET /health_check` - Health check

---

## Security Features

### Password Security
- **Algorithm:** Argon2 (memory-hard, GPU-resistant)
- **Parameters:** Default Argon2 parameters
- **Storage:** Only password hash stored, never plaintext

### JWT Authentication
- **Algorithm:** RS256 (RSA with SHA-256)
- **Expiration:** 24 hours
- **Claims:** user_id, username, exp
- **Storage:** Client-side (localStorage/sessionStorage)

### Input Validation
- **Library:** Validator crate with derive macros
- **Coverage:**
  - Email format validation
  - Password strength (min 8 chars)
  - Username format (alphanumeric + underscores)
  - Required field validation
  - Length constraints

### HTTP Security Headers
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `Strict-Transport-Security: max-age=31536000; includeSubDomains`

### SQL Injection Protection
- Parameterized queries via libsql
- No string interpolation in SQL

### Authorization
- Route-level protection via `AuthenticatedUser` extractor
- Resource ownership checks (articles, comments)
- Automatic 401/403 responses

---

## Development Guide

### Prerequisites
- Rust 1.75+ (2024 edition)
- Cargo
- Shuttle CLI

### Setup

1. **Clone repository:**
   ```bash
   git clone https://github.com/crustyrustacean/crustyrustacean-dev-blog.git
   cd crustyrustacean-dev-blog
   ```

2. **Install Shuttle:**
   ```bash
   cargo install cargo-shuttle
   ```

3. **Run locally:**
   ```bash
   shuttle run
   ```

4. **Access application:**
   - Homepage: http://localhost:8000
   - API: http://localhost:8000/api/*

### Development Workflow

1. **Run tests:**
   ```bash
   cargo test
   ```

2. **Check code:**
   ```bash
   cargo clippy
   cargo fmt --check
   ```

3. **Deploy:**
   ```bash
   shuttle deploy
   ```

### Project Layout

```
crustyrustacean-dev-blog/
├── src/
│   ├── bin/main.rs              # Shuttle entry point
│   └── lib/                     # Library code
│       ├── lib.rs               # Module root
│       ├── auth.rs              # Authentication
│       ├── database.rs          # Database layer
│       ├── errors.rs            # Error handling
│       ├── models/              # Data models
│       └── routes/              # HTTP handlers
├── templates/                   # Tera templates
├── static/                      # CSS, JS, images
├── tests/                       # Integration tests
├── Cargo.toml                   # Dependencies
└── Shuttle.toml                 # Deployment config
```

### Key Patterns

1. **Error Handling:** Always use `AppError` enum
2. **Authentication:** Use `AuthenticatedUser` extractor
3. **Database Queries:** Parameterized via libsql
4. **Responses:** Use `ApiResponse<T>` for consistency
5. **Validation:** Derive `Validate` on input structs
6. **Testing:** Write integration tests for all features

### Adding New Features

1. **Create route handler** in `src/lib/routes/`
2. **Add to router** in `src/lib/startup.rs`
3. **Write tests** in `tests/api/`
4. **Update templates** if needed
5. **Add JavaScript** if interactive
6. **Document in README**

---

## Quick Reference

### File Locations

| Component | Location |
|-----------|----------|
| Main entry | `src/bin/main.rs` |
| Route handlers | `src/lib/routes/*.rs` |
| Data models | `src/lib/models/*.rs` |
| Templates | `templates/**/*.html` |
| JavaScript | `static/js/*.js` |
| Tests | `tests/api/*.rs` |
| Configuration | `Cargo.toml`, `Shuttle.toml` |

### Key Commands

| Task | Command |
|------|---------|
| Run locally | `shuttle run` |
| Run tests | `cargo test` |
| Deploy | `shuttle deploy` |
| Format code | `cargo fmt` |
| Lint code | `cargo clippy` |
| Watch tests | `cargo watch -x test` |

### Important Constants

| Constant | Value | Location |
|----------|-------|----------|
| JWT expiration | 24 hours | `src/lib/auth.rs` |
| Default pagination | 20 items | Various routes |
| RSS item limit | 20 articles | `src/lib/routes/rss.rs` |
| Max password length | 128 chars | `src/lib/models/user.rs` |

---

## Recent Changes (v1.8.0)

- **Search Functionality**: Full-text search across articles
  - Real-time search with responsive UI
  - Search across title, description, and body content
  - Dedicated search results page
  - Modular JavaScript integration
  - Comprehensive integration tests

## Previous Changes (v1.7.0)

- **Media Library System (MVP)**: Complete media management with OpenDAL integration
  - File upload with multipart form support
  - Full CRUD operations for media files and metadata
  - Cloud storage integration via OpenDAL
  - Media library admin page with responsive UI
  - Comprehensive integration tests
- **Storage Infrastructure**: OpenDAL backend for flexible storage
  - Support for multiple providers (S3, local filesystem, etc.)
  - Automatic path generation and organization
  - File metadata tracking
- **Security**: User-based access control and ownership validation
- **Testing**: Comprehensive test coverage for media operations

---

**Index Generated:** 2025-10-25
**Version:** 1.8.0
**Documentation:** https://github.com/crustyrustacean/crustyrustacean-dev-blog
