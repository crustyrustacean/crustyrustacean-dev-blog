# Changelog

All notable changes to the CrustyRustacean Dev Blog project will be documented in this file.

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
