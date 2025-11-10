# Changelog

All notable changes to the CrustyRustacean Dev Blog project will be documented in this file.

## [2.1.0] - 2025-11-09

### Added - Categories System
- **Complete Categories Management**:
  - Full CRUD operations for blog post categories
  - Automatic slug generation from category names
  - Optional one-to-many relationship with articles
  - Category descriptions for additional context
  - Article count tracking per category
  - Safe cascade deletion (preserves articles when category deleted)

- **API Endpoints**:
  - `POST /api/categories` - Create new category (protected)
  - `GET /api/categories` - List all categories with article counts (public)
  - `GET /api/categories/{slug}` - Get specific category (public)
  - `PUT /api/categories/{slug}` - Update category (protected)
  - `DELETE /api/categories/{slug}` - Delete category (protected)
  - `GET /admin/categories` - Categories admin page (protected)

- **Article Integration**:
  - Articles can optionally reference a category by slug
  - Category validation on article creation/update
  - Category-based article filtering: `GET /api/articles?category={slug}`
  - Category field included in article responses
  - Safe cascade behavior: deleting category sets articles' category to null

- **Frontend Integration**:
  - Categories admin page at `/admin/categories`
  - Create/edit/delete categories with modal dialogs
  - Dynamic table with article counts and actions
  - Modular JavaScript (`categories-admin.js`)
  - Responsive Bootstrap design
  - Confirmation dialogs for destructive actions

- **Database Schema**:
  - New `categories` table with id, name, slug, description, timestamps
  - UNIQUE constraints on name and slug
  - `articles.category_id` column for optional association
  - Indexes on category slug and article category_id for efficient queries
  - Automatic timestamp management (created_at, updated_at)

- **Security Features**:
  - Input validation (1-100 character limit on names)
  - Duplicate name and slug prevention
  - Proper authorization on protected endpoints
  - XSS prevention with HTML escaping in frontend
  - SQL injection prevention with parameterized queries

- **Test Coverage**:
  - 21 comprehensive integration tests
  - Basic CRUD operations (5 tests)
  - Authentication requirements (3 tests)
  - Validation and duplicates (4 tests)
  - Article integration scenarios (7 tests)
  - Data integrity verification (2 tests)

### Technical Details
- **TDD Implementation**: Built following test-driven development methodology
- **Consistent Patterns**: Follows project conventions from tags and media library systems
- **Slug Generation**: Automatic URL-friendly slug creation from category names
- **Production Ready**: Comprehensive error handling and security measures
- **Extensibility**: Foundation for category-based navigation and filtering UI
- **Total Tests**: Now 143 tests (122 existing + 21 categories), all passing

## [1.8.0] - 2025-10-25

### Added - Search Functionality
- **Complete Search System**:
  - Full-text search across article titles, descriptions, and body content
  - Real-time search with responsive UI
  - Search results with highlighted relevance
  - Empty state handling for no results
  - Client-side JavaScript module for search interactions

- **API Endpoint**:
  - `GET /api/search` - Search articles by query parameter (public)
  - Returns matching articles with author profiles and metadata
  - Efficient database querying with LIKE pattern matching

- **Frontend Integration**:
  - Search form in navigation bar (base template)
  - Dedicated search results page at `/search`
  - Modular JavaScript (`search.js`) for search interactions
  - Responsive design matching existing UI patterns
  - Real-time result display with article previews

- **Test Coverage**:
  - Comprehensive integration tests for search endpoint
  - Tests for empty queries, no results, and successful searches
  - Validation of search result format and content
  - Multi-article search scenarios

### Technical Details
- **Implementation**: Built following TDD methodology with test-first approach
- **Performance**: Efficient SQL LIKE queries for pattern matching
- **User Experience**: Seamless integration with existing navigation
- **Extensibility**: Foundation for advanced search features (filters, relevance ranking)

## [1.7.0] - 2025-10-24

### Added - Media Library System (MVP)
- **Complete Media Management**:
  - Media file upload with multipart form support
  - Image-only validation with 50MB file size limit
  - Automatic unique filename generation (UUID-based)
  - Per-user media organization and storage
  - Full CRUD operations for media files and metadata
  - OpenDAL integration for cloud storage backend
  - Public media download/display endpoint

- **API Endpoints**:
  - `POST /api/media` - Upload media files with metadata (protected)
  - `GET /api/media` - List user's media with pagination and filtering (protected)
  - `GET /api/media/:id` - Get media metadata (protected)
  - `PUT /api/media/:id` - Update media metadata (protected)
  - `DELETE /api/media/:id` - Delete media file and metadata (protected)
  - `GET /api/media/:id/download` - Download/display media file (public)

- **Frontend Integration**:
  - Media library admin page at `/admin/media`
  - Responsive grid layout for media display
  - Upload form with drag-and-drop support
  - Real-time metadata editing
  - Delete confirmation dialogs
  - Image preview functionality
  - Modular JavaScript (`media-library.js`)
  - Custom CSS styling (`media-library.css`)

- **Database Schema**:
  - New `media_library` table with comprehensive metadata
  - Fields: id, user_id, filename, storage_path, title, alt_text, caption, description
  - Technical metadata: mime_type, file_size, width, height
  - Timestamps: uploaded_at, updated_at
  - Proper indexing and foreign key constraints

- **Storage Integration**:
  - OpenDAL backend for flexible storage options
  - Support for multiple storage providers (S3, local filesystem, etc.)
  - Automatic path generation with user-based organization
  - File metadata tracking (size, content type)
  - Secure upload and download operations

- **Security Features**:
  - User-based access control and ownership validation
  - File type validation (images only)
  - File size limits (50MB)
  - Proper authorization checks on all operations
  - Secure storage path handling

- **Test Coverage**:
  - Comprehensive integration tests for all endpoints
  - Upload, list, retrieve, update, delete scenarios
  - Authorization and ownership validation tests
  - Edge cases and error handling tests

### Technical Details
- **TDD Implementation**: Built following test-driven development methodology
- **Cloud-Native**: OpenDAL integration enables deployment flexibility
- **MVP Status**: Core functionality complete, ready for production use
- **Extensibility**: Architecture supports future enhancements (image processing, CDN integration)
- **Performance**: Efficient query patterns with pagination support

## [1.6.1] - 2025-10-23

### Fixed
- **Editor Page Authentication Bug**: Fixed issue where editing existing articles caused the UI to show Login/Register buttons (user appeared logged out) while updates still worked
  - Root cause: `get_edit_article_page` handler was not passing user context to the template
  - Impact: Users saw inconsistent authentication state in navbar when editing articles
  - Solution: Added user query and context to edit article page handler (matching pattern from other page handlers)
  - Location: `src/lib/routes/articles.rs:475-506`

### Added - Test Infrastructure Improvements
- **New Test Helpers** (`tests/api/helpers.rs`):
  - `HtmlResponseValidator` trait for validating HTML page responses
    - Automatically checks HTML content-type headers
    - Validates HTML document structure
    - Returns body text for further assertions
  - `assert_body_contains()` helper function for cleaner multi-string assertions
    - Simplifies checking multiple expected strings in response body
    - Provides clear error messages when content is missing

- **Enhanced Template Rendering Tests**:
  - Added `test_editor_page_shows_authenticated_user_in_navbar` - Validates user context on edit article page
  - Added `test_new_article_editor_shows_authenticated_user_in_navbar` - Validates user context on new article page
  - These tests specifically check for authentication state bugs in UI rendering

### Changed - Test Code Quality
- **Refactored `admin_dashboard.rs` tests** to eliminate code duplication:
  - Reduced test code by 41% (233 lines → 137 lines, saved 96 lines)
  - Now uses existing `TestUserBuilder` and `TestArticleBuilder` traits
  - Applied new `HtmlResponseValidator` and `assert_body_contains` helpers
  - Improved consistency with other test files
  - Enhanced maintainability and readability

### Technical Details
- **Test Coverage**: Now 122 tests (107 integration + 15 unit), all passing
- **Code Quality**: Follows DRY principle with reusable test helpers
- **Maintainability**: Template rendering bugs now caught by automated tests
- **Consistency**: All page handler tests now use common validation patterns

## [1.1.0] - 2025-10-22

### Added - Advanced Tag Management System
- **Complete Tag Editing Functionality**:
  - Enhanced `PUT /api/articles/{slug}` endpoint with comprehensive tag editing
  - Support for adding, removing, and replacing tags on existing articles
  - "Replace All Tags" strategy for flexible tag management
  - Maintains referential integrity across all tag operations
- **Backend Implementation**:
  - Added `tag_list` field to `UpdateArticle` struct with proper serde serialization
  - Implemented robust tag association logic with database transaction safety
  - Enhanced error handling for tag-related operations
  - Code quality improvements with reusable tag handling functions
- **Test Coverage**: Comprehensive integration tests for tag editing scenarios
  - Updating article tags (add/remove/replace)
  - Removing all tags from articles
  - Adding tags to previously tagless articles
  - Authorization and validation checks
  - Idempotency testing for tag operations
- **Development Process**: Followed structured 7-phase implementation plan
  - Phase 1: Backend data model updates
  - Phase 2: Tag update logic implementation
  - Phase 3: Code quality improvements and refactoring
  - Phase 4-7: Testing, validation, and comprehensive coverage

### Enhanced
- **Article Management**: Improved article editing workflow with seamless tag operations
- **Data Integrity**: Enhanced database operations for tag associations
- **Code Quality**: Reduced code duplication with helper functions for tag operations
- **API Consistency**: Maintained backward compatibility while adding new functionality

### Technical Details
- **Test-Driven Development**: Implemented using TDD methodology with comprehensive test coverage
- **Database Safety**: All tag operations wrapped in proper transaction handling
- **Performance**: Optimized tag association queries for better performance
- **Maintainability**: Clean separation of concerns with reusable tag management utilities

## [1.0.0] - 2025-10-19

### Added - Comments System (TDD Implementation)
- **Complete Comments API**:
  - `POST /api/articles/{slug}/comments` - Add comment to article (protected)
  - `GET /api/articles/{slug}/comments` - Get all comments for article
  - `DELETE /api/articles/{slug}/comments/{id}` - Delete comment (protected, author only)
- **Interactive Frontend Integration**:
  - Real-time comment submission and display
  - Delete functionality for comment authors
  - Responsive comment UI with Bootstrap styling
  - Proper authentication and authorization handling
- **Test Coverage**: 11 comprehensive integration tests
  - Comment creation and retrieval
  - Authorization and ownership validation
  - Error handling and edge cases
  - Multi-user comment scenarios

### Added - JavaScript Architecture Refactoring
- **Modular JavaScript Structure**:
  - Extracted all inline JavaScript to separate files
  - Created 19 JavaScript modules for different features
  - Improved code organization and maintainability
  - Enhanced error handling and user feedback
- **New JavaScript Files**:
  - `admin.js` - Admin dashboard functionality
  - `article-init.js` - Article initialization
  - `article-list.js` - Articles listing interactions
  - `article-page.js` - Individual article page features
  - `article.js` - General article functionality
  - `auth.js` - Authentication forms and validation
  - `authors.js` - Authors discovery features
  - `base.js` - Global functionality and utilities
  - `comments.js` - Comments system interactions
  - `editor.js` - Article editor functionality
  - `error404.js` - Error page enhancements
  - `favorites.js` - Favorites system
  - `feed.js` - Personal feed functionality
  - `homepage.js` - Homepage dynamic features
  - `profile.js` - User profile interactions
  - `utils.js` - Shared utilities and helpers

### Added - Template Rendering System
- **Enhanced Template Architecture**:
  - Fixed article template rendering bugs
  - Added comprehensive template rendering tests
  - Improved template performance and reliability
  - Better separation of concerns between templates and JavaScript
- **New Template Files**:
  - `article-old.html` - Legacy article template for compatibility
  - `modern.html` - Modern article template with enhanced features
- **Test Coverage**: New template rendering integration tests
  - Template compilation and rendering verification
  - Data binding and context validation
  - Error handling in template system

### Fixed
- **Template Rendering**: Resolved article template rendering bugs
- **Dark Mode**: Eliminated rogue dark mode styling issues
- **Article Styling**: Fixed missing article styling and text color issues
- **Static Assets**: Resolved missing static folder configuration in `Shuttle.toml`
- **Markdown Rendering**: Fixed errors in markdown processing
- **Code Quality**: Cleaned up clippy lints for better code quality

### Changed
- **Test Suite**: Expanded to 81 integration tests (from 76)
- **JavaScript Architecture**: Complete refactoring from inline to modular approach
- **Template System**: Enhanced with better error handling and performance
- **Codebase Maintainability**: Improved separation of concerns and code organization
- **Version**: Bumped to 1.0.0 indicating production readiness

### Technical Details
- **Production Ready**: Comprehensive testing and bug fixes for stable release
- **Maintainable Codebase**: Modular JavaScript architecture improves long-term maintenance
- **Enhanced Performance**: Template rendering optimizations and JavaScript modularization
- **Better UX**: Improved error handling and user feedback across all features

## [0.7.0] - 2025-10-18

### Added - Tags System (TDD Implementation)
- **GET /api/tags endpoint** - Retrieves all unique tags from articles
  - Returns tags in alphabetical order
  - Proper JSON response format: `{"tags": ["tag1", "tag2", ...]}`
  - Empty array when no tags exist
- **Frontend Integration**:
  - Dynamic tag loading on homepage sidebar
  - Dynamic tag display on articles listing page
  - Clickable tag badges for filtering articles
  - Loading states and error handling
- **Test Coverage**: 3 integration tests
  - Empty state handling
  - Unique tag retrieval from multiple articles
  - Alphabetical ordering verification

### Added - RSS Feed
- **GET /rss endpoint** - Standards-compliant RSS 2.0 feed
  - Latest 20 articles syndicated automatically
  - Proper XML structure with channel and item elements
  - RFC 2822 date formatting for pubDate
  - Atom self-reference link for feed discovery
  - Author attribution with email fallback
  - Article permalinks and descriptions
- **Security Features**:
  - XML special character escaping (`&`, `<`, `>`, `"`, `'`)
  - Prevents XSS and XML injection attacks
  - Proper content-type header: `application/rss+xml; charset=utf-8`
- **Test Coverage**: 4 integration tests + 1 unit test
  - Valid XML structure verification
  - Article inclusion and formatting
  - XML escaping functionality
  - Empty feed handling

### Changed
- **Test Suite**: Expanded from 59 to 65 integration tests
- **Documentation**: Updated README.md and CLAUDE.md with new features
- **Routes Module**: Added tags.rs and rss.rs handlers
- **Project Structure**: Enhanced with RSS and tags capabilities

### Technical Details
- **TDD Approach**: Both features developed test-first
- **Zero Breaking Changes**: All existing tests continue to pass
- **Production Ready**: Proper error handling and security measures

## [0.6.0] - 2025-10-18

### Added - Authors Discovery System
- Browse and discover authors to follow
- Search functionality by username or bio
- Interactive follow/unfollow with real-time updates
- Pagination and responsive design
- Complete API endpoints with authentication (8 tests)

### Added - Articles Feed System
- Personal feed showing articles from followed users
- Feed page with responsive design
- Pagination and empty state handling (5 tests)

### Added - Favorites System
- Favorite/unfavorite articles with real-time updates
- Personal favorites page for logged-in users
- Favorites count display and filtering (11 tests)

## [0.5.0] - Earlier

### Added
- Complete blog functionality with CRUD operations
- User authentication with JWT tokens
- Turso/libSQL database integration
- Professional frontend with Tera templates
- Comprehensive test suite (42 tests)
- Security features (Argon2, JWT validation)
