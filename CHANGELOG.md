# Changelog

All notable changes to the CrustyRustacean Dev Blog project will be documented in this file.

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
