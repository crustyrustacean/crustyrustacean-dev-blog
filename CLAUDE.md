# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Local Development
```bash
shuttle run           # Run the application locally with Shuttle
cargo test            # Run all tests
```

### Deployment
```bash
shuttle deploy        # Deploy to Shuttle platform
```

### Testing
```bash
cargo test            # Run unit and integration tests
cargo test api::      # Run only API integration tests
cargo test auth       # Run only authentication tests
cargo test favorites  # Run only favorites API tests
```

## Architecture Overview

This is a developer blog application built with Axum web framework, Tera templating, and Turso/libSQL database, deployed on Shuttle.

### Crate Structure
- **Binary crate**: `src/bin/main.rs` - Shuttle entry point and application bootstrap
- **Library crate**: `src/lib/lib.rs` - Core application logic with module exports

### Key Modules

**Core Infrastructure:**
- `startup.rs` - App struct and router configuration with middleware layers (tracing, CORS, request ID)
- `state.rs` - AppState containing Tera template engine and database connection
- `database.rs` - DatabaseConnection wrapper with migration runner
- `config.rs` - Application configuration management
- `telemetry.rs` - Tracing/logging setup with Bunyan formatter

**API Layer:**
- `routes/` - HTTP handlers organized by feature (health_check, index, users)
- `response.rs` - Standardized API response types
- `errors.rs` - Application error types with proper HTTP status mapping

**Data Layer:**
- `models/` - Database entities (User, Article, Comment, Tag)
- `auth.rs` - JWT authentication and password hashing with Argon2

### Database Schema
The application uses Turso/libSQL with the following main tables:
- `users` - User accounts with authentication
- `articles` - Blog posts with slug-based routing
- `comments` - Article comments
- `tags` - Article categorization
- `article_tags` - Many-to-many article-tag relationships
- `user_favorites` - User article favorites
- `user_follows` - User following relationships

### Authentication System
- **JWT-based authentication** with configurable secret (24-hour expiration)
- **Secure password hashing** using Argon2 with salt generation
- **Complete user lifecycle**: registration, login, profile management
- **Protected endpoints** with Bearer token validation
- **Input validation** with comprehensive error handling
- **Production-ready security** following industry best practices

### Template System
- **Tera templating engine** configured in AppState with proper error handling
- **Dynamic templates** with database integration for real content
- **Template structure**:
  - `templates/base.html` - Base layout with responsive design
  - `templates/index.html` - Homepage with article summaries and statistics
  - `templates/articles/list.html` - Articles listing page
  - `templates/articles/article.html` - Individual article view
  - `templates/articles/editor.html` - Article creation/editing
  - `templates/articles/feed.html` - Personal feed page
  - `templates/auth/login.html` - User login
  - `templates/auth/register.html` - User registration
  - `templates/admin/dashboard.html` - Admin interface
  - `templates/profile/authors.html` - Authors discovery page
- **Static assets** served from `static/` directory (CSS, JS, images)
- **Production-ready features**: SEO-friendly URLs, responsive design, accessibility

### Shuttle Integration
- Uses Shuttle's Turso integration for database provisioning
- Secrets management through Shuttle's secret store
- Automatic deployment and scaling

### API Routes Structure
**HTML Pages:**
```
GET  /                                  # Homepage with dynamic article summaries
GET  /articles                          # Articles listing page
GET  /articles/{slug}                   # Individual article view
GET  /login                             # Login page
GET  /register                          # Registration page
GET  /editor                            # Article editor (protected)
GET  /editor/{slug}                     # Edit existing article (protected)
GET  /admin                             # Admin dashboard (protected)
GET  /profiles/{username}               # User profile page
GET  /profiles                          # Authors discovery page (protected)
GET  /feed                              # Personal feed page (protected)
GET  /favorites                         # User's favorite articles (protected)
GET  /rss                               # RSS feed (XML)
```

**API Endpoints:**
```
GET  /health_check                      # Health check endpoint
POST /api/users                         # User registration
POST /api/users/login                   # User login
GET  /api/user                          # Get current user
PUT  /api/user                          # Update current user
GET  /api/profiles                      # List all users/authors with follow status (protected)
GET  /api/profiles/{username}           # Get user profile
POST /api/profiles/{username}/follow    # Follow user
DELETE /api/profiles/{username}/follow  # Unfollow user
GET  /api/articles                      # List articles (JSON)
GET  /api/articles/feed                 # Get personal feed (protected)
POST /api/articles                      # Create article (protected)
GET  /api/articles/{slug}               # Get article (JSON)
PUT  /api/articles/{slug}               # Update article (protected)
DELETE /api/articles/{slug}             # Delete article (protected)
POST /api/articles/{slug}/favorite      # Favorite article (protected)
DELETE /api/articles/{slug}/favorite    # Unfavorite article (protected)
GET  /api/tags                          # Get all tags
```

### Testing Strategy
- **Comprehensive integration tests** in `tests/api/` directory
- **Authentication test suite** covering happy path and failure scenarios:
  - User registration and login flows
  - JWT token validation and protected endpoints
  - Input validation and error handling
  - Security edge cases and malformed requests
- **Favorites API test suite** with comprehensive TDD coverage:
  - Favorite/unfavorite article operations
  - Authentication requirements and error handling
  - Idempotent behavior and edge cases
  - Multi-user scenarios and favorites counting
  - Query filtering by favorited user
- **Feed API test suite** with complete TDD implementation:
  - Personal feed from followed users only
  - Authentication requirements and empty state handling
  - Pagination functionality and follow-based filtering
  - Multi-user scenarios and following relationships
- **Authors Discovery test suite** with full TDD coverage:
  - Authors listing with follow status accuracy
  - Search functionality by username and bio
  - Pagination and authentication requirements
  - HTML page rendering and interactive features
- **Tags API test suite** with complete TDD implementation:
  - Get all tags endpoint with alphabetical ordering
  - Empty state handling and unique tag filtering
  - Frontend integration with dynamic tag display
- **RSS Feed test suite** with comprehensive coverage:
  - Valid XML generation and proper content-type headers
  - Article inclusion with correct formatting and escaping
  - XML special character escaping for security
  - Empty feed handling when no articles exist
- **Test infrastructure** with isolated databases and proper setup
- **Test helpers** in `tests/api/helpers.rs` for consistent test environments
- **HTTP client testing** using reqwest for real request/response validation

### Error Handling Architecture
- **Unified Application Errors**: Single `AppError` type for consistent HTTP responses
- **Domain-Specific Errors**: `AuthError` for authentication with automatic conversion to `AppError`
- **Consistent JSON Format**: Standardized error response structure across all endpoints
- **Proper HTTP Status Mapping**: Semantic error types map to appropriate status codes
- **Template Error Integration**: Tera and Axum template errors handled seamlessly

### Current Implementation Status
- ✅ **User Authentication**: Complete with registration, login, and JWT validation
- ✅ **Database Integration**: Turso/libSQL with automated migrations
- ✅ **Security**: Argon2 password hashing, JWT tokens, input validation
- ✅ **Error Handling**: Unified error architecture with proper separation of concerns
- ✅ **Testing**: 65 integration tests covering authentication, favorites, feed, authors discovery, tags, RSS, and error scenarios
- ✅ **API Endpoints**: User management and profile operations
- ✅ **Code Quality**: Eliminated error handling duplication, improved maintainability
- ✅ **Blog Features**: Complete article CRUD operations with slug-based routing
- ✅ **Frontend Templates**: Dynamic Tera templates with database integration
- ✅ **Homepage**: Dynamic article summaries, blog statistics, responsive design
- ✅ **Articles System**: Full listing page, individual article views, admin interface
- ✅ **Content Management**: Article creation, editing, deletion with proper authorization
- ✅ **Favorites System**: Complete TDD implementation with frontend and backend integration
- ✅ **Articles Feed System**: Personal feed showing articles from followed users with TDD implementation
- ✅ **Authors Discovery System**: Browse and search for authors to follow with comprehensive TDD coverage
- ✅ **Social Features**: Complete follow/unfollow system with real-time updates and interactive UI
- ✅ **Tags System**: Complete tags endpoint with TDD implementation and dynamic frontend display
- ✅ **RSS Feed**: Standards-compliant RSS 2.0 feed with proper XML escaping and article syndication
- ✅ **Production Ready**: Complete blog functionality with user-friendly navigation and social features
- 🚧 **Advanced Features**: Comments system, advanced filtering, enhanced search (planned)
