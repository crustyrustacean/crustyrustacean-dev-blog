# Claude Code Development Guide

This document provides guidance for Claude Code when working on the CrustyRustacean Dev Blog project.

## Project Overview

The CrustyRustacean Dev Blog is a production-ready Rust web application built with:
- **Backend**: Axum web framework with libSQL/Turso database
- **Frontend**: Tera templating with Bootstrap and modular JavaScript
- **Deployment**: Shuttle platform
- **Testing**: Comprehensive integration test suite (81+ tests)

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
    ├── models/             # Data models (User, Article, Comment, Tag)
    ├── routes/             # HTTP route handlers
    ├── response.rs         # API response utilities
    ├── startup.rs          # App initialization
    ├── state.rs            # Application state
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

### 7. Authentication & Security

- **JWT tokens** with 24-hour expiration
- **Argon2** password hashing
- **Bearer token** authentication for protected endpoints
- **Input validation** with comprehensive error messages
- **SQL injection protection** with parameterized queries

### 8. API Endpoints Reference

#### Core Endpoints
```
GET  /                              # Homepage
GET  /articles                      # Articles listing
GET  /articles/{slug}               # Individual article
POST /api/articles                  # Create article (protected)
PUT  /api/articles/{slug}           # Update article with tags (protected)
GET  /api/tags                      # Get all tags
POST /api/users                     # User registration
POST /api/users/login               # User login
GET  /rss                           # RSS feed
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

### 13. Current Status (v1.1.0)

#### Recently Completed
- ✅ Advanced tag management system (Phases 1-7)
- ✅ Complete tag editing functionality
- ✅ Comprehensive test coverage for tag operations
- ✅ Code quality improvements with helper functions

#### Next Potential Features
- Rich text editor enhancements
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