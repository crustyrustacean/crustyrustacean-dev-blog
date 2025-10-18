# CrustyRustacean Dev Blog

A developer blog application built with [Axum](https://github.com/tokio-rs/axum), [Tera](https://keats.github.io/tera/) templating, and [Turso](https://turso.tech/) database, deployed on [Shuttle](https://shuttle.dev).

## Features

### ✅ Implemented
- **Axum web framework** for high-performance HTTP services
- **User authentication system** with JWT tokens and secure password hashing
- **User registration and login** with comprehensive validation
- **Protected API endpoints** with Bearer token authentication
- **User profiles and social features** (follow/unfollow system)
- **Turso/libSQL database integration** with automated migrations
- **Comprehensive test suite** with 10 integration tests
- **Production-ready security** (Argon2 password hashing, JWT validation)
- **Shuttle deployment ready** with environment configuration
- **Health check endpoint** for monitoring
- **Complete blog functionality**:
  - Dynamic homepage with real article summaries and statistics
  - Articles listing page with responsive design
  - Individual article pages with slug-based routing
  - Full CRUD operations for articles
  - Admin dashboard for content management
- **Professional frontend** with Tera templating:
  - Bootstrap-based responsive design
  - SEO-friendly URLs and metadata
  - User-friendly navigation and error handling
  - Real-time statistics and content updates
- **Content management system**:
  - Article creation and editing interface
  - Tag system for categorization
  - User favorites and social features
  - Authorization-protected admin features

### 🚧 Planned
- Comments system for articles
- Advanced search and filtering
- Rich text editor enhancements
- Social sharing features

## Project Structure

```
src/
  bin/main.rs       # Application entry point (Shuttle)
  lib/
    lib.rs          # Library root with module exports
    auth.rs         # JWT authentication and password hashing
    config.rs       # Configuration management
    database.rs     # Database connection and migrations
    errors.rs       # Error types with HTTP status mapping
    response.rs     # Standardized API response types
    startup.rs      # App initialization and router setup
    state.rs        # Shared application state
    telemetry.rs    # Tracing/logging setup
    models/         # Database entities (User, Article, Comment, Tag)
    routes/         # HTTP route handlers
      health_check.rs # Health check endpoint
      users.rs      # User authentication routes
      index.rs      # Homepage with dynamic content
      articles.rs   # Article management and listing
      auth.rs       # Authentication pages
      profile.rs    # User profile management
      error_pages.rs# Error handling pages
      mod.rs        # Routes module
static/
  css/
    styles.css      # Main stylesheet with responsive design
  js/               # JavaScript for interactive features
  images/           # Static images and assets
templates/          # Tera templates for HTML rendering
  base.html         # Base layout template
  index.html        # Dynamic homepage
  articles/
    list.html       # Articles listing page
    article.html    # Individual article view
    editor.html     # Article creation/editing
    simple.html     # Simplified article view
  auth/
    login.html      # User login page
    register.html   # User registration page
  admin/
    dashboard.html  # Admin interface
  profile/
    profile.html    # User profile page
  errors/
    404.html        # Not found page
tests/
  api/
    auth.rs         # Authentication integration tests (9 tests)
    health_check.rs # Health check tests
    helpers.rs      # Test infrastructure and utilities
    main.rs         # Test module declarations
```

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Cargo Shuttle](https://docs.shuttle.rs/introduction/installation)

### Running Locally

```sh
shuttle run
```

### Running Tests

```sh
cargo test              # Run all tests (10 total)
cargo test auth         # Run authentication tests only
cargo test api::        # Run all API integration tests
```

The test suite includes comprehensive authentication testing:
- ✅ User registration and login flows
- ✅ JWT token validation and security
- ✅ Protected endpoint access control
- ✅ Input validation and error handling
- ✅ Edge cases and malformed requests

### Deploying with Shuttle

```sh
shuttle deploy
```

## API Endpoints

### HTML Pages
```
GET  /                             # Homepage with dynamic articles
GET  /articles                     # Articles listing page
GET  /articles/{slug}              # Individual article view
GET  /login                        # Login page
GET  /register                     # Registration page
GET  /editor                       # Article editor (protected)
GET  /editor/{slug}                # Edit article (protected)
GET  /admin                        # Admin dashboard (protected)
GET  /profiles/{username}          # User profile page
```

### Authentication API
```
POST /api/users                    # User registration
POST /api/users/login              # User login
GET  /api/user                     # Get current user (protected)
PUT  /api/user                     # Update current user (protected)
```

### User Profiles API
```
GET    /api/profiles/{username}           # Get user profile
POST   /api/profiles/{username}/follow    # Follow user (protected)
DELETE /api/profiles/{username}/follow    # Unfollow user (protected)
```

### Articles API
```
GET    /api/articles               # List articles (JSON)
POST   /api/articles               # Create article (protected)
GET    /api/articles/{slug}        # Get article (JSON)
PUT    /api/articles/{slug}        # Update article (protected)
DELETE /api/articles/{slug}        # Delete article (protected)
```

### System
```
GET /health_check                  # Health check endpoint
```

### Blog Features

The application now includes a complete blog system:

1. **Dynamic Homepage**: Real-time article summaries, blog statistics (article count, topics covered, latest post date)
2. **Articles Listing**: Professional grid layout with filtering options and pagination
3. **Individual Articles**: Slug-based URLs for SEO-friendly article pages  
4. **Content Management**: Full CRUD operations with proper authorization
5. **Admin Interface**: Dashboard for managing articles, users, and content
6. **Responsive Design**: Bootstrap-based UI that works on all device sizes

### Authentication Flow
1. **Register**: `POST /api/users` with `{user: {username, email, password}}`
2. **Login**: `POST /api/users/login` with `{user: {email, password}}`
3. **Access Protected Endpoints**: Include `Authorization: Bearer <jwt_token>` header

### Content Management Flow
1. **View Articles**: Browse homepage or `/articles` for all posts
2. **Create Content**: Login and visit `/editor` to write new articles
3. **Manage Content**: Use `/admin` dashboard to edit/delete existing articles
4. **Public Access**: All articles are publicly viewable at `/articles/{slug}`

## Security Features

- **Argon2 Password Hashing**: Industry-standard secure password storage
- **JWT Authentication**: Stateless token-based authentication with 24-hour expiration
- **Input Validation**: Comprehensive validation with proper error messages
- **Environment-based Secrets**: Configurable JWT secret for different environments
- **Protected Endpoints**: Bearer token validation for sensitive operations

## Code Quality & Architecture

- **Unified Error Handling**: Consolidated error architecture eliminates duplication
- **Domain-Driven Design**: Authentication errors properly separated from HTTP concerns
- **Comprehensive Testing**: 10 integration tests covering happy path and failure scenarios
- **Production-Ready**: Industry best practices for security, error handling, and testing
- **Maintainable Codebase**: Clean separation of concerns and consistent patterns

