# Claude Code Development Guide

This document provides guidance for Claude Code when working on the CrustyRustacean Dev Blog project.

## Project Overview

The CrustyRustacean Dev Blog is a production-ready Rust web application built with:
- **Backend**: Axum web framework with libSQL/Turso database
- **Frontend**: Tera templating with Bootstrap and modular JavaScript
- **Deployment**: Shuttle platform
- **Testing**: Comprehensive test suite (242 tests: 223 integration + 19 unit)

## Development Workflow

### 1. Running the Application

```bash
# Start development server
shuttle run

# Run all tests
cargo test

# Run specific test categories
cargo test auth         # Authentication tests
cargo test articles     # Article management tests
cargo test favorites    # Favorites system tests
cargo test tags         # Tags system tests
cargo test categories   # Categories system tests
cargo test comments     # Comments system tests
cargo test drafts       # Drafts management tests
cargo test shortcodes   # Shortcodes system tests
cargo test search       # Search functionality tests
cargo test media        # Media library tests
```

### 2. Project Structure

```
src/
├── bin/main.rs              # Shuttle entry point
└── lib/
    ├── lib.rs              # Library root with module exports
    ├── auth.rs             # JWT authentication
    ├── config.rs           # Configuration management
    ├── database.rs         # Database operations
    ├── errors.rs           # Error handling
    ├── models/             # Data models (User, Article, Comment, Tag, Category, Media)
    ├── routes/             # HTTP route handlers
    ├── response.rs         # API response utilities
    ├── shortcodes.rs       # Shortcode parsing for internal article links
    ├── startup.rs          # App initialization
    ├── state.rs            # Application state
    ├── storage.rs          # OpenDAL storage integration
    └── telemetry.rs        # Logging/tracing
```

### 3. Testing Strategy

This project follows **Test-Driven Development (TDD)**:

1. **Write tests first** for new features
2. **Run tests** to confirm they fail
3. **Implement minimum code** to make tests pass
4. **Refactor** while keeping tests green
5. **Add comprehensive test coverage** for edge cases

#### Test Categories
- **Authentication**: User registration, login, JWT validation
- **Articles**: CRUD operations, favorites, feed system, pagination
- **Tags**: Tag management, editing, filtering
- **Categories**: CRUD operations, article association, filtering
- **Comments**: Comment creation, deletion, authorization
- **Drafts**: Draft status, drafts management, author-only visibility
- **Shortcodes**: Shortcode parsing, database resolution, internal linking
- **Social**: Following, authors discovery, personal feeds
- **Search**: Full-text search across articles
- **Template Rendering**: HTML page rendering, authentication state validation
- **Media Library**: File upload, storage, retrieval, metadata management
- **Markdown Upload**: File upload, parsing, metadata extraction

#### Test Helpers and Utilities

The project includes comprehensive test helpers in `tests/api/helpers.rs` to reduce duplication and improve test maintainability:

**Test Builder Traits:**
- `TestUserBuilder` - Simplify user registration in tests
  ```rust
  // Register with specific details
  let token = app.register_user("username", "email@example.com", "password").await;

  // Register with default email pattern (username@example.com)
  let token = app.register_user_default("username").await;
  ```

- `TestArticleBuilder` - Simplify article creation in tests
  ```rust
  // Create article with all details
  let slug = app.create_article(&token, "Title", "Description", "Body", vec!["tag1", "tag2"]).await;

  // Create article with minimal details
  let slug = app.create_article_simple(&token, "Title").await;
  ```

**HTML Response Validation:**
- `HtmlResponseValidator` trait - Validate HTML page responses
  ```rust
  let body = response.assert_html_response().await;
  // Automatically validates HTML content-type and structure
  ```

- `assert_body_contains()` - Check multiple strings in response
  ```rust
  assert_body_contains(&body, &["Expected Text 1", "Expected Text 2", "Username"]);
  ```

**Test Fixtures:**
- `TestFixture` - Complex multi-user test scenarios
  ```rust
  let fixture = TestFixture::new().await
      .with_user_default("alice")
      .with_user_default("bob").await;
  let alice_token = fixture.get_token("alice");
  ```

**Usage Example:**
```rust
#[tokio::test]
async fn test_admin_dashboard_shows_articles() {
    let app = spawn_app().await;

    // Register user and create article (using helpers)
    let token = app.register_user_default("testuser").await;
    app.create_article_simple(&token, "Test Article").await;

    // Access dashboard
    let response = app.client
        .get(format!("{}/admin", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Validate HTML response
    let body = response.assert_html_response().await;
    assert_body_contains(&body, &["Admin Dashboard", "Test Article", "testuser"]);
}
```

### 4. Code Quality Standards

#### Rust Best Practices
- Use `clippy` for linting: `cargo clippy`
- Follow Rust naming conventions
- Implement proper error handling with custom error types
- Use `serde` for JSON serialization with proper field naming

#### Database Operations
- Always use transactions for multi-step operations
- Implement proper error handling for database failures
- Use parameterized queries to prevent SQL injection
- Include database migrations for schema changes

#### API Design
- Follow RESTful conventions
- Use proper HTTP status codes
- Include comprehensive validation
- Maintain backward compatibility

### 5. Feature Implementation Guide

#### Adding New Features

1. **Plan the implementation** (create phases if complex)
2. **Write integration tests** covering:
   - Happy path scenarios
   - Error conditions
   - Edge cases
   - Authorization checks
3. **Implement backend logic**:
   - Add/update data models in `src/lib/models/`
   - Create/update route handlers in `src/lib/routes/`
   - Add database operations if needed
4. **Update frontend** (if applicable):
   - Modify Tera templates in `templates/`
   - Add JavaScript functionality in `static/js/`
   - Update CSS styling in `static/css/`
5. **Run tests** and ensure all pass
6. **Update documentation** (README.md, CHANGELOG.md)

#### Example: Tag Editing Implementation (Phases 1-7 Complete)

The tag editing feature was implemented following a structured approach:

**Phase 1**: Backend data model updates
- Added `tag_list` field to `UpdateArticle` struct
- Proper serde configuration for camelCase compatibility

**Phase 2**: Backend tag update logic
- Implemented "Replace All Tags" strategy
- Database transaction safety for tag associations

**Phase 3**: Code quality improvements
- Extracted reusable helper functions
- Reduced code duplication

**Phase 4-7**: Testing and validation
- Comprehensive test coverage for all scenarios
- Authorization and validation checks
- Idempotency testing

### 6. Database Schema

Key tables and relationships:
- **users**: User accounts and authentication
- **articles**: Blog posts with metadata (includes draft column)
- **tags**: Tag definitions
- **article_tags**: Many-to-many relationship
- **categories**: Category definitions with slug
- **comments**: Article comments
- **favorites**: User article favorites
- **follows**: User following relationships
- **media_library**: Uploaded media files with metadata
- **newsletter_subscribers**: Email subscribers with confirmation/unsubscribe tokens
- **newsletter_issues**: Newsletter content and metadata
- **newsletter_delivery_logs**: Tracking newsletter deliveries

### 7. Authentication & Security

- **JWT tokens** with 24-hour expiration
- **Argon2** password hashing
- **Bearer token** authentication for protected endpoints
- **Input validation** with comprehensive error messages
- **SQL injection protection** with parameterized queries

### 8. API Endpoints Reference

#### Core Endpoints
```
# HTML Pages
GET    /                              # Homepage with draft widget
GET    /articles                      # Articles listing (paginated)
GET    /articles?page=N               # Specific page of articles
GET    /articles/{slug}               # Individual article
GET    /admin                         # Admin dashboard (protected)
GET    /admin/drafts                  # Drafts management page (protected)
GET    /admin/categories              # Categories management page (protected)
GET    /admin/media                   # Media library page (protected)
GET    /admin/newsletters             # Newsletter management page (protected)
GET    /newsletter                    # Newsletter subscription page
GET    /newsletter/confirmed/{token}  # Subscription confirmation page
GET    /newsletter/unsubscribed/{token} # Unsubscribe confirmation page
GET    /search                        # Search results page

# Articles API
POST   /api/articles                  # Create article with draft status (protected)
POST   /api/articles/upload-markdown  # Upload markdown file to editor (protected)
GET    /api/articles                  # List articles (excludes drafts)
GET    /api/articles/drafts           # List user's draft articles (protected)
GET    /api/articles/{slug}           # Get article (author can view own drafts)
PUT    /api/articles/{slug}           # Update article with tags and draft status (protected)
DELETE /api/articles/{slug}           # Delete article (protected)

# Categories API
POST   /api/categories                # Create category (protected)
GET    /api/categories                # List all categories with article counts
GET    /api/categories/{slug}         # Get specific category
PUT    /api/categories/{slug}         # Update category (protected)
DELETE /api/categories/{slug}         # Delete category (protected)

# Other APIs
GET    /api/tags                      # Get all tags
GET    /api/search?q={query}          # Search articles (excludes drafts)
POST   /api/users                     # User registration
POST   /api/users/login               # User login
GET    /rss                           # RSS feed

# Media Library API
POST   /api/media                     # Upload media (protected)
GET    /api/media                     # List media (protected)
GET    /api/media/:id                 # Get media metadata (protected)
PUT    /api/media/:id                 # Update media metadata (protected)
DELETE /api/media/:id                 # Delete media (protected)
GET    /api/media/:id/download        # Download media file

# Newsletter API
POST   /api/newsletters/subscribe           # Subscribe to newsletter
POST   /api/newsletters/confirm/{token}     # Confirm subscription
GET    /api/newsletters/confirm/{token}     # Confirm subscription (GET support)
POST   /api/newsletters/unsubscribe/{token} # Unsubscribe from newsletter
GET    /api/newsletters/unsubscribe/{token} # Unsubscribe from newsletter (GET support)
POST   /api/admin/newsletters               # Create newsletter issue (protected)
GET    /api/admin/newsletters               # List newsletter issues (protected)
GET    /api/admin/newsletters/{id}          # Get newsletter issue (protected)
PUT    /api/admin/newsletters/{id}          # Update newsletter issue (protected)
DELETE /api/admin/newsletters/{id}          # Delete newsletter issue (protected)
POST   /api/admin/newsletters/{id}/send     # Send newsletter (protected)
GET    /api/admin/newsletters/stats         # Get newsletter statistics (protected)
```

### 9. Common Tasks

#### Adding a New API Endpoint
1. Add route handler to appropriate file in `src/lib/routes/`
2. Update router configuration in `src/lib/startup.rs`
3. Add integration tests in `tests/api/`
4. Update API documentation in README.md

#### Database Migrations
1. Add migration logic to `src/lib/database.rs`
2. Test migration with fresh database
3. Ensure backward compatibility

#### Frontend Updates
1. Modify templates in `templates/`
2. Add JavaScript in `static/js/` (modular approach)
3. Update CSS in `static/css/styles.css`
4. Test responsive design

### 10. Deployment

```bash
# Deploy to Shuttle
shuttle deploy

# Check deployment status
shuttle status
```

Ensure all secrets are configured in `Secrets.toml`:
- `JWT_SECRET`
- `TURSO_DATABASE_URL`
- `TURSO_AUTH_TOKEN`

### 11. Troubleshooting

#### Common Issues
- **Database connection**: Check Turso credentials
- **JWT validation**: Verify JWT_SECRET configuration
- **Test failures**: Run `cargo test` to identify issues
- **Static assets**: Ensure `Shuttle.toml` includes assets

#### Debug Commands
```bash
cargo check                    # Check for compilation errors
cargo clippy                   # Lint code
cargo test -- --nocapture     # Run tests with output
shuttle logs                   # View deployment logs
```

### 12. Development Philosophy

This project emphasizes:
- **Test-Driven Development**: Write tests first
- **Production Readiness**: Comprehensive error handling
- **Code Quality**: Clean, maintainable code
- **Security**: Industry best practices
- **Documentation**: Keep docs up-to-date

### 13. Current Status (v2.9.0)

#### Recently Completed (Nov 2025)
- ✅ **Newsletter Subscription and Delivery Service**: Complete newsletter system
  - Email subscription with double opt-in confirmation
  - Newsletter creation and management (admin only)
  - Send newsletters to confirmed subscribers
  - Newsletter statistics dashboard
  - Public subscription page at `/newsletter`
  - Admin management page at `/admin/newsletters`
  - Re-subscription support for unsubscribed users
  - 16 comprehensive integration tests

- ✅ **Article Preview Modal**: Publishing preview feature
  - Real-time preview of articles before publishing
  - Client-side markdown rendering with marked.js
  - Large modal showing formatted article as it will appear
  - Preview title, description, tags, and rendered body content
  - HTML escaping for security
  - Fast preview without server-side processing

- ✅ **Admin Dashboard Pagination**: Paginated article management
  - Pagination controls on admin dashboard for better performance
  - Consistent navigation experience across the application
  - Improved layout and user experience
  - Fixed pagination bug with draft articles

- ✅ **Pagination System**: Articles listing pagination (10 per page)
  - Bootstrap-styled pagination controls
  - Page navigation with query parameters
  - Total count calculation for accurate page numbers
  - 4 comprehensive integration tests

- ✅ **Draft Widget**: Landing page draft count widget
  - Shows draft count for authenticated authors
  - "View Drafts" link to drafts management page
  - Conditional display based on authentication
  - 4 comprehensive integration tests

- ✅ **Markdown File Upload**: Direct markdown file upload in editor
  - Upload `.md` files with automatic parsing
  - Metadata extraction (title, description, body)
  - 1MB file size limit with validation
  - 6 comprehensive integration tests

- ✅ **Drafts Management System**: Complete drafts workflow
  - Dedicated admin page at `/admin/drafts`
  - One-click publish, edit, preview, delete functionality
  - Real-time UI updates with Bootstrap modals
  - 8+ comprehensive integration tests

- ✅ **Draft Status Feature**: Draft/published status system
  - Save articles as drafts or publish them
  - Author-only visibility for draft articles
  - Drafts excluded from public listings, search, and feeds
  - 11 comprehensive integration tests

- ✅ **Shortcode System**: Internal article linking
  - `[[article:slug]]` or `[[article:slug|Custom Text]]` syntax
  - Automatic resolution to markdown links at save time
  - Copy-to-clipboard slug functionality in admin dashboard
  - 15+ comprehensive tests (unit and integration)

- ✅ **Categories System**: Complete category management
  - Full CRUD operations for blog post categories
  - Automatic slug generation from category names
  - Optional one-to-many relationship with articles
  - Category-based article filtering
  - Categories admin page with interactive UI
  - 21 comprehensive integration tests

- ✅ **Search System**: Full-text search functionality
  - Search across article titles, descriptions, and body
  - Real-time search with responsive UI
  - Dedicated search results page
  - Excludes draft articles from results

- ✅ **Media Library System**: Complete media management
  - File upload with OpenDAL storage integration
  - Full CRUD operations for media files and metadata
  - User-based access control and ownership validation
  - Media library admin page with responsive UI

- ✅ **Advanced Tag Management**: Complete tag editing
  - Add, remove, or replace tags on existing articles
  - "Replace All Tags" strategy for flexible management
  - Maintains referential integrity

- ✅ **Comments System**: Full commenting functionality
  - Add, view, and delete comments on articles
  - Proper authorization (author-only deletion)
  - Real-time UI updates

- ✅ **Social Features**: Complete social system
  - Follow/unfollow users
  - Authors discovery page with search
  - Personal feed showing articles from followed authors
  - Favorites system with dedicated page

- ✅ **Test Coverage**: Comprehensive test suite
  - 258 total tests (239 integration + 19 unit)
  - All features tested following TDD methodology
  - 100% passing test suite

#### Next Potential Features
- Image processing and optimization
- Media thumbnails and previews
- CDN integration for media delivery
- Rich text editor enhancements with media embedding
- Advanced search features (filters, relevance ranking, date ranges)
- Social sharing features (Twitter, Facebook, LinkedIn)
- Email notifications (new comments, new followers)
- Article series/collections feature
- Table of contents generation for long articles
- Article version history/revisions

### 14. Key Feature Guides

#### Draft Workflow
The draft system allows authors to save articles in progress before publishing:

**Backend Implementation:**
- `draft` column in articles table (INTEGER: 0=published, 1=draft)
- Draft status field in all article models (Article, CreateArticle, UpdateArticle)
- Author-only visibility enforced at route handler level
- Drafts automatically excluded from public endpoints (list, search, feed)

**Frontend Integration:**
- Draft checkbox in article editor (`/editor`)
- Drafts management page at `/admin/drafts`
- Draft count widget on homepage for authenticated users
- One-click publish functionality

**Key Rules:**
1. Only article author can view their own drafts
2. Drafts never appear in public listings or search results
3. Default status is published (backward compatible)
4. Draft status can be toggled at any time via update endpoint

**Testing Considerations:**
- Test author-only access to draft articles
- Verify drafts excluded from public listings
- Test draft status toggling (draft ↔ published)
- Verify unauthorized users cannot view drafts

#### Shortcode System
Shortcodes enable easy internal article linking with automatic title resolution:

**Syntax:**
- Basic: `[[article:slug]]` → `[Article Title](/articles/slug)`
- Custom text: `[[article:slug|Custom Text]]` → `[Custom Text](/articles/slug)`

**Processing Flow:**
1. Author writes article with shortcodes
2. On save, `process_shortcodes()` parses shortcode patterns
3. Database lookup retrieves article titles
4. Shortcodes replaced with markdown links
5. Processed markdown stored in database

**Implementation:**
- Parsing: `src/lib/shortcodes.rs` with regex-based parser
- Integration: Called in article create/update handlers
- Admin UI: Copy-to-clipboard buttons for slugs in dashboard

**Key Functions:**
- `parse_shortcodes(text: &str) -> Vec<Shortcode>` - Extract shortcodes
- `process_shortcodes(text: &str, db: &DatabaseConnection) -> Result<String>` - Resolve and replace

**Testing Considerations:**
- Test basic shortcode parsing and resolution
- Test custom text variations
- Test multiple shortcodes in same article
- Test non-existent article references (fallback behavior)
- Test malformed shortcode patterns

#### Pagination System
Articles listing uses pagination for better performance and UX:

**Implementation:**
- 10 articles per page
- Query parameter: `?page=N` (1-indexed)
- Total count query for accurate page calculation
- Bootstrap-styled pagination controls

**Backend:**
- `LIMIT` and `OFFSET` in SQL queries
- Total count query: `SELECT COUNT(*) FROM articles WHERE draft = 0`
- Page metadata passed to template context

**Frontend:**
- Pagination controls in `templates/articles/list.html`
- Previous/Next buttons with page numbers
- Disabled state for first/last pages
- Current page highlighted

#### Categories System
Categories provide a way to organize articles by topic:

**Implementation:**
- One-to-many relationship (one category, many articles)
- Optional: articles can have no category
- Automatic slug generation from category names
- CRUD operations via `/api/categories` endpoints

**Key Features:**
- Category-based filtering: `GET /api/articles?category={slug}`
- Article count per category
- Safe cascade deletion (articles' category_id set to NULL)
- Admin UI at `/admin/categories`

**Database:**
- `categories` table with id, name, slug, description
- `articles.category_id` foreign key (nullable)
- Unique constraints on name and slug

### 15. Best Practices for Claude Code Sessions

1. **Always run tests** before and after changes
2. **Follow TDD approach** for new features
3. **Update documentation** when adding features (README.md, CHANGELOG.md, CLAUDE.md)
4. **Maintain backward compatibility**
5. **Use structured implementation phases** for complex features
6. **Verify deployment readiness** before completing work
7. **Use test helpers** (`TestUserBuilder`, `TestArticleBuilder`, etc.) to reduce duplication
8. **Follow existing patterns** for consistency (especially for new features)
9. **Consider draft status** when implementing article-related features
10. **Use shortcodes** for internal article references in documentation

### 16. Recent Architecture Patterns

The project has established several patterns that should be followed:

**Modular JavaScript:**
- Each feature has its own JS file in `static/js/`
- Use module pattern with clear separation of concerns
- Bootstrap modals for confirmations
- Real-time UI updates with fade animations

**Admin Pages:**
- Use `/admin/{feature}` URL pattern
- Consistent table layout with Bootstrap
- Action buttons (Edit, Delete, etc.) on each row
- Empty state messaging when no data exists

**Test Organization:**
- Feature-specific test files in `tests/api/`
- Use test helpers for common operations
- Group related tests together
- Test happy path, error cases, and authorization

**API Response Format:**
- Consistent JSON structure with proper HTTP status codes
- Validation errors return detailed messages
- Authorization failures return 401/403 appropriately
- Use `ApiError` type for error handling

**Database Migrations:**
- Add migration logic to `src/lib/database.rs`
- Use `CREATE TABLE IF NOT EXISTS`
- Include proper indexes for performance
- Default values for backward compatibility

This guide should help ensure consistent, high-quality development on the CrustyRustacean Dev Blog project.
