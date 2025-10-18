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
- **Static asset serving** and template system setup

### 🚧 Planned
- Blog article management (create, read, update, delete)
- Comments system for articles
- Tag system for article categorization
- Frontend UI with Tera templating

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
      index.rs      # Template rendering
      mod.rs        # Routes module
static/             # Static web assets (CSS, JS, images)
templates/          # Tera templates for HTML rendering
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

### Authentication
```
POST /api/users                    # User registration
POST /api/users/login              # User login
GET  /api/user                     # Get current user (protected)
PUT  /api/user                     # Update current user (protected)
```

### User Profiles
```
GET    /api/profiles/{username}           # Get user profile
POST   /api/profiles/{username}/follow    # Follow user (protected)
DELETE /api/profiles/{username}/follow    # Unfollow user (protected)
```

### System
```
GET /health_check                  # Health check endpoint
GET /                             # Index page
```

### Authentication Flow
1. **Register**: `POST /api/users` with `{user: {username, email, password}}`
2. **Login**: `POST /api/users/login` with `{user: {email, password}}`
3. **Access Protected Endpoints**: Include `Authorization: Bearer <jwt_token>` header

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

