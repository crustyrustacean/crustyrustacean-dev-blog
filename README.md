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
- **Comprehensive test suite** with 222+ integration tests and 19 unit tests
- **Production-ready security** (Argon2 password hashing, JWT validation)
- **Shuttle deployment ready** with environment configuration
- **Health check endpoint** for monitoring
- **Media library with cloud storage** (OpenDAL integration)
- **Complete blog functionality**:
  - Dynamic homepage with real article summaries and statistics
  - Articles listing page with responsive design
  - Individual article pages with slug-based routing
  - Full CRUD operations for articles
  - Admin dashboard for content management with copy-to-clipboard slug functionality
- **Professional frontend** with Tera templating:
  - Bootstrap-based responsive design
  - SEO-friendly URLs and metadata
  - User-friendly navigation and error handling
  - Real-time statistics and content updates
- **Content management system**:
  - Article creation and editing interface
  - Tag system for categorization
  - Complete user favorites system with interactive UI
  - Authorization-protected admin features
- **Favorites System** (TDD implementation):
  - Favorite/unfavorite articles with real-time updates
  - Personal favorites page for logged-in users
  - Favorites count display and filtering
  - Interactive frontend with JavaScript integration
  - Complete API endpoints with authentication
- **Articles Feed System** (TDD implementation):
  - Personal feed showing articles from followed users
  - Feed page with responsive design and real-time updates
  - Pagination and empty state handling
  - Complete API endpoints with authentication
- **Authors Discovery System** (TDD implementation):
  - Browse and discover authors to follow
  - Search functionality by username or bio
  - Interactive follow/unfollow with real-time updates
  - Pagination and responsive design
  - Complete API endpoints with authentication
- **Search System** (TDD implementation):
  - Full-text search across article titles, descriptions, and body
  - Real-time search with responsive UI
  - Dedicated search results page at /search
  - GET /api/search endpoint for querying articles
  - Empty state handling for no results
  - Modular JavaScript integration
- **Tags System** (TDD implementation):
  - GET /api/tags endpoint for retrieving all tags
  - Alphabetically sorted tag lists
  - Dynamic tag display on homepage and articles listing
  - Tag-based article filtering (via query parameters)
  - **Advanced Tag Management**:
    - Complete tag editing functionality for articles
    - Add, remove, or replace tags on existing articles
    - Flexible tag association with "Replace All Tags" strategy
    - Comprehensive validation and error handling
    - Maintains referential integrity across tag operations
- **RSS Feed**:
  - Standards-compliant RSS 2.0 feed at /rss
  - Automatic article syndication (latest 20 articles)
  - Proper XML escaping for security
  - RFC 2822 date formatting
- **Comments System** (TDD implementation):
  - Add comments to articles with authentication
  - View all comments on article pages
  - Delete own comments with proper authorization
  - Interactive frontend with real-time updates
  - Complete API endpoints (POST, GET, DELETE)
  - 11 comprehensive integration tests
- **Media Library System** (MVP):
  - File upload with multipart form support
  - Image-only validation with 50MB file size limit
  - OpenDAL integration for cloud storage backend
  - Full CRUD operations for media files and metadata
  - Per-user media organization and storage
  - Media library admin page with responsive UI
  - Public media download/display endpoint
  - User-based access control and ownership validation
  - Comprehensive integration tests
- **Categories System** (TDD implementation):
  - Full CRUD operations for blog post categories
  - Automatic slug generation from category names
  - Optional one-to-many relationship with articles
  - Category-based article filtering and organization
  - Categories admin page with interactive UI
  - Complete API endpoints with authentication
  - 21 comprehensive integration tests
- **Draft Status System** (TDD implementation):
  - Save articles as drafts or publish them
  - Draft/published status toggle
  - Author-only visibility for draft articles
  - Drafts excluded from public listings, search, and feeds
  - Draft status indicator in admin dashboard
  - 11 comprehensive integration tests
- **Drafts Management Panel**:
  - Dedicated admin page at `/admin/drafts` for managing draft articles
  - One-click publish functionality
  - Edit, preview, and delete actions for drafts
  - Draft count widget on landing page for authenticated users
  - Real-time UI updates with Bootstrap modals
  - 8+ comprehensive integration tests
- **Shortcode System**:
  - Internal article linking with `[[article:slug]]` syntax
  - Optional custom link text: `[[article:slug|Custom Text]]`
  - Automatic resolution to markdown links at save time
  - Copy-to-clipboard slug functionality in admin dashboard
  - Smart handling of missing articles with fallback links
  - 15+ comprehensive tests
- **Markdown File Upload**:
  - Upload `.md` files directly in article editor
  - Automatic parsing and extraction of title, description, body
  - 1MB file size limit with validation
  - Smart defaults ("Untitled Article" when no title found)
  - Progress indicators and error handling
  - 6 comprehensive integration tests
- **Articles Pagination**:
  - 10 articles per page on `/articles` listing
  - Bootstrap-styled pagination controls
  - Page number display with previous/next navigation
  - Query parameter support: `?page=N`
  - 4 comprehensive pagination tests

### 🚧 Planned
- Image processing and optimization
- Media thumbnails and CDN integration
- Advanced search features (filters, relevance ranking)
- Rich text editor enhancements with media embedding
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
    models/         # Database entities (User, Article, Comment, Tag, Category, Media)
    routes/         # HTTP route handlers
    shortcodes.rs   # Shortcode parsing for internal article links
      health_check.rs # Health check endpoint
      users.rs      # User authentication routes
      index.rs      # Homepage with dynamic content
      articles.rs   # Article management and listing
      auth.rs       # Authentication pages
      profile.rs    # User profile management
      tags.rs       # Tags API endpoint
      categories.rs # Categories management endpoints
      rss.rs        # RSS feed generation
      error_pages.rs# Error handling pages
      mod.rs        # Routes module
static/
  css/
    styles.css      # Main stylesheet with responsive design
  js/               # Modular JavaScript architecture (20 modules)
    admin.js        # Admin dashboard with copy-to-clipboard functionality
    article-init.js # Article initialization
    article-list.js # Articles listing interactions
    article-page.js # Individual article page features
    article.js      # General article functionality
    auth.js         # Authentication forms and validation
    authors.js      # Authors discovery features
    base.js         # Global functionality and utilities
    comments.js     # Comments system interactions
    editor.js       # Article editor functionality
    error404.js     # Error page enhancements
    favorites.js    # Favorites system
    feed.js         # Personal feed functionality
    homepage.js     # Homepage dynamic features
    profile.js      # User profile interactions
    search.js       # Search functionality
    categories-admin.js # Categories admin page functionality
    drafts.js       # Drafts management page functionality
    media-library.js # Media library management
    utils.js        # Shared utilities and helpers
  images/           # Static images and assets
templates/          # Tera templates for HTML rendering
  base.html         # Base layout template
  index.html        # Dynamic homepage
  articles/
    list.html       # Articles listing page
    article.html    # Individual article view
    article-old.html# Legacy article template
    modern.html     # Modern article template
    editor.html     # Article creation/editing
    simple.html     # Simplified article view
  auth/
    login.html      # User login page
    register.html   # User registration page
  admin/
    dashboard.html  # Admin interface
    categories.html # Categories management page
    drafts.html     # Drafts management page
    media.html      # Media library page
  profile/
    profile.html    # User profile page
    favorites.html  # User favorites page
    authors.html    # Authors discovery page
  errors/
    404.html        # Not found page
tests/
  api/
    auth.rs         # Authentication integration tests
    favorites.rs    # Favorites API integration tests (11 tests)
    feed.rs         # Feed API integration tests (5 tests)
    feed_page.rs    # Feed page integration tests (2 tests)
    authors.rs      # Authors discovery integration tests (8 tests)
    tags.rs         # Tags API integration tests (3 tests)
    rss.rs          # RSS feed integration tests (4 tests)
    comments.rs     # Comments API integration tests (11 tests)
    search.rs       # Search API integration tests
    categories.rs   # Categories API integration tests (21 tests)
    drafts.rs       # Drafts management integration tests (8 tests)
    media.rs        # Media library integration tests
    shortcodes.rs   # Shortcodes system integration tests (15+ tests)
    admin_dashboard.rs # Admin dashboard integration tests
    template_rendering.rs # Template rendering integration tests
    health_check.rs # Health check tests
    helpers.rs      # Test infrastructure and utilities
    main.rs         # Test module declarations
  shortcodes.rs     # Shortcodes unit tests
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
cargo test              # Run all tests (241 total: 222 integration + 19 unit)
cargo test auth         # Run authentication tests only
cargo test favorites    # Run favorites API tests only
cargo test feed         # Run feed API tests only
cargo test authors      # Run authors discovery tests only
cargo test tags         # Run tags API tests only
cargo test categories   # Run categories API tests only
cargo test rss          # Run RSS feed tests only
cargo test comments     # Run comments API tests only
cargo test search       # Run search API tests only
cargo test media        # Run media library tests only
cargo test drafts       # Run drafts management tests only
cargo test shortcodes   # Run shortcodes system tests only
cargo test api::        # Run all API integration tests
```

The test suite includes comprehensive testing:
- ✅ User registration and login flows
- ✅ JWT token validation and security
- ✅ Protected endpoint access control
- ✅ Input validation and error handling
- ✅ Edge cases and malformed requests
- ✅ Complete favorites API functionality
- ✅ Personal feed functionality with following relationships
- ✅ Authors discovery with search and pagination
- ✅ Tags API with alphabetical sorting and filtering
- ✅ Categories API with CRUD operations and article integration
- ✅ Draft status system with author-only visibility
- ✅ Drafts management with publish/delete operations
- ✅ Shortcodes system with parsing and database resolution
- ✅ Markdown file upload with metadata extraction
- ✅ Articles pagination with page navigation
- ✅ RSS feed generation with XML validation
- ✅ Comments API with authorization and validation
- ✅ Search API with full-text querying across articles
- ✅ Media library with file upload and storage operations
- ✅ Template rendering with authentication state validation
- ✅ Idempotent operations and multi-user scenarios

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
GET  /search                       # Search results page
GET  /login                        # Login page
GET  /register                     # Registration page
GET  /editor                       # Article editor (protected)
GET  /editor/{slug}                # Edit article (protected)
GET  /admin                        # Admin dashboard (protected)
GET  /admin/categories             # Categories management page (protected)
GET  /admin/drafts                 # Drafts management page (protected)
GET  /admin/media                  # Media library admin page (protected)
GET  /profiles/{username}          # User profile page
GET  /profiles                     # Authors discovery page (protected)
GET  /feed                         # Personal feed page (protected)
GET  /favorites                    # User's favorite articles (protected)
GET  /rss                          # RSS feed (XML)
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
GET    /api/profiles                      # List all users/authors with follow status (protected)
GET    /api/profiles/{username}           # Get user profile
POST   /api/profiles/{username}/follow    # Follow user (protected)
DELETE /api/profiles/{username}/follow    # Unfollow user (protected)
```

### Articles API
```
GET    /api/articles                    # List articles (JSON, paginated, excludes drafts)
GET    /api/articles?page=N             # Get specific page of articles
GET    /api/articles/feed               # Get personal feed (protected, includes own drafts)
GET    /api/articles/drafts             # List user's draft articles (protected)
POST   /api/articles                    # Create article (protected, supports draft status)
POST   /api/articles/upload-markdown    # Upload markdown file to editor (protected)
GET    /api/articles/{slug}             # Get article (JSON, author can view own drafts)
PUT    /api/articles/{slug}             # Update article with tag editing and draft status (protected)
DELETE /api/articles/{slug}             # Delete article (protected)
```

### Favorites API
```
POST   /api/articles/{slug}/favorite   # Favorite article (protected)
DELETE /api/articles/{slug}/favorite   # Unfavorite article (protected)
GET    /api/articles?favorited={user}  # Get articles favorited by user
```

### Tags API
```
GET /api/tags                      # Get all tags (alphabetically sorted)
```

### Categories API
```
POST   /api/categories             # Create new category (protected)
GET    /api/categories             # List all categories with article counts
GET    /api/categories/{slug}      # Get specific category
PUT    /api/categories/{slug}      # Update category (protected)
DELETE /api/categories/{slug}      # Delete category (protected)
```

### Search API
```
GET /api/search?q={query}          # Search articles by query (excludes drafts)
```

### Comments API
```
POST   /api/articles/{slug}/comments       # Add comment to article (protected)
GET    /api/articles/{slug}/comments       # Get all comments for article
DELETE /api/articles/{slug}/comments/{id}  # Delete comment (protected, author only)
```

### Media Library API
```
POST   /api/media                  # Upload media file (protected)
GET    /api/media                  # List user's media with pagination (protected)
GET    /api/media/:id              # Get media metadata (protected)
PUT    /api/media/:id              # Update media metadata (protected)
DELETE /api/media/:id              # Delete media file (protected)
GET    /api/media/:id/download     # Download/display media file
```

### System
```
GET /health_check                  # Health check endpoint
```

### Blog Features

The application now includes a complete blog system:

1. **Dynamic Homepage**: Real-time article summaries, blog statistics (article count, topics covered, latest post date), draft count widget for authors
2. **Articles Listing**: Professional grid layout with pagination (10 per page), filtering options, and page navigation
3. **Individual Articles**: Slug-based URLs for SEO-friendly article pages
4. **Content Management**: Full CRUD operations with proper authorization
5. **Admin Interface**: Dashboard for managing articles, users, and content with copy-to-clipboard slug functionality
6. **Draft System**: Save articles as drafts, manage drafts separately, one-click publishing, author-only visibility
7. **Drafts Management**: Dedicated admin page for viewing, editing, publishing, and deleting draft articles
8. **Shortcode System**: Internal article linking with `[[article:slug]]` syntax for easy cross-referencing
9. **Markdown Upload**: Upload markdown files directly in editor with automatic parsing and form population
10. **Favorites System**: Interactive favorite/unfavorite with personal favorites page
11. **Personal Feed**: Curated feed showing articles from authors you follow
12. **Authors Discovery**: Browse and search for authors to follow with interactive UI
13. **Social Features**: Complete follow/unfollow system with real-time updates
14. **Tags System**: Dynamic tag display with filtering and alphabetical organization
15. **Categories System**: Organize posts with categories, filter by category, admin management UI
16. **Search System**: Full-text search across articles with real-time results (excludes drafts)
17. **RSS Feed**: Standards-compliant syndication for RSS readers and aggregators
18. **Comments System**: Add, view, and delete comments on articles with proper authorization
19. **Media Library**: Upload and manage media files with cloud storage integration
20. **Responsive Design**: Bootstrap-based UI that works on all device sizes

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
- **Comprehensive Testing**: 241 total tests (222 integration + 19 unit) covering happy path and failure scenarios
- **Test-Driven Development**: Tags, RSS, Comments, Search, Categories, Drafts, and Media Library features built with TDD approach
- **Production-Ready**: Industry best practices for security, error handling, and testing
- **Maintainable Codebase**: Clean separation of concerns and consistent patterns
- **Cloud-Native Storage**: OpenDAL integration for flexible storage backend options
- **Modular JavaScript**: 20+ JavaScript modules with separation of concerns
- **Content Management**: Complete draft workflow with author-only visibility
- **Internal Linking**: Shortcode system for seamless article cross-referencing

