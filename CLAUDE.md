# Claude Code Development Guide

This document provides guidance for Claude Code when working on the CrustyRustacean Dev Blog project.

## Project Overview

The CrustyRustacean Dev Blog is a production-ready Rust web application built with:
- **Backend**: Axum web framework with libSQL/Turso database
- **Frontend**: Tera templating with Bootstrap and modular JavaScript
- **Deployment**: Shuttle platform
- **Testing**: Comprehensive test suite (122 tests: 107 integration + 15 unit)

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
cargo test comments     # Comments system tests
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
    ├── models/             # Data models (User, Article, Comment, Tag, Media)
    ├── routes/             # HTTP route handlers
    ├── response.rs         # API response utilities
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
- **Articles**: CRUD operations, favorites, feed system
- **Tags**: Tag management, editing, filtering
- **Comments**: Comment creation, deletion, authorization
- **Social**: Following, authors discovery, personal feeds
- **Template Rendering**: HTML page rendering, authentication state validation
- **Media Library**: File upload, storage, retrieval, metadata management

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
- **articles**: Blog posts with metadata
- **tags**: Tag definitions
- **article_tags**: Many-to-many relationship
- **comments**: Article comments
- **favorites**: User article favorites
- **follows**: User following relationships
- **media_library**: Uploaded media files with metadata

### 7. Authentication & Security

- **JWT tokens** with 24-hour expiration
- **Argon2** password hashing
- **Bearer token** authentication for protected endpoints
- **Input validation** with comprehensive error messages
- **SQL injection protection** with parameterized queries

### 8. API Endpoints Reference

#### Core Endpoints
```
GET    /                              # Homepage
GET    /articles                      # Articles listing
GET    /articles/{slug}               # Individual article
POST   /api/articles                  # Create article (protected)
PUT    /api/articles/{slug}           # Update article with tags (protected)
GET    /api/tags                      # Get all tags
POST   /api/users                     # User registration
POST   /api/users/login               # User login
GET    /rss                           # RSS feed
POST   /api/media                     # Upload media (protected)
GET    /api/media                     # List media (protected)
GET    /api/media/:id                 # Get media metadata (protected)
PUT    /api/media/:id                 # Update media metadata (protected)
DELETE /api/media/:id                 # Delete media (protected)
GET    /api/media/:id/download        # Download media file
GET    /admin/media                   # Media library page (protected)
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

### 13. Current Status (v1.7.0)

#### Recently Completed
- ✅ **Media Library System (MVP)**: Complete media management functionality
  - File upload with OpenDAL storage integration
  - Full CRUD operations for media files and metadata
  - User-based access control and ownership validation
  - Media library admin page with responsive UI
  - Comprehensive integration tests
- ✅ **Storage Infrastructure**: OpenDAL integration for cloud storage
  - Support for multiple storage backends (S3, local filesystem, etc.)
  - Secure file upload and download
  - Automatic path generation and organization
- ✅ Advanced tag management system (Phases 1-7)
- ✅ Complete tag editing functionality
- ✅ Comprehensive test coverage (122+ tests, all passing)

#### Next Potential Features
- Image processing and optimization
- Media thumbnails and previews
- CDN integration for media delivery
- Rich text editor enhancements with media embedding
- Advanced search and filtering
- Social sharing features
- Email notifications
- Article categories/sections

### 14. Best Practices for Claude Code Sessions

1. **Always run tests** before and after changes
2. **Follow TDD approach** for new features
3. **Update documentation** when adding features
4. **Maintain backward compatibility**
5. **Use structured implementation phases** for complex features
6. **Verify deployment readiness** before completing work

This guide should help ensure consistent, high-quality development on the CrustyRustacean Dev Blog project.