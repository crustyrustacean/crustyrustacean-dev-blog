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
- Tera templating engine configured in AppState
- Templates located in `templates/` directory
- Static assets served from `static/` directory

### Shuttle Integration
- Uses Shuttle's Turso integration for database provisioning
- Secrets management through Shuttle's secret store
- Automatic deployment and scaling

### API Routes Structure
```
GET  /health_check                      # Health check endpoint
GET  /                                  # Index page
POST /api/users                         # User registration
POST /api/users/login                   # User login
GET  /api/user                          # Get current user
PUT  /api/user                          # Update current user
GET  /api/profiles/{username}           # Get user profile
POST /api/profiles/{username}/follow    # Follow user
DELETE /api/profiles/{username}/follow  # Unfollow user
```

### Testing Strategy
- **Comprehensive integration tests** in `tests/api/` directory
- **Authentication test suite** covering happy path and failure scenarios:
  - User registration and login flows
  - JWT token validation and protected endpoints
  - Input validation and error handling
  - Security edge cases and malformed requests
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
- ✅ **Testing**: 10 integration tests covering authentication flows and error scenarios
- ✅ **API Endpoints**: User management and profile operations
- ✅ **Code Quality**: Eliminated error handling duplication, improved maintainability
- 🚧 **Blog Features**: Article CRUD, comments, tags (planned)
- 🚧 **Frontend**: Tera templates and UI (planned)