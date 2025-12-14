# Changelog

All notable changes to the CrustyRustacean Dev Blog project will be documented in this file.

## [2.16.0] - 2025-12-14

### Removed - Bootstrap Dependency
- **Complete Bootstrap Removal**:
  - Removed Bootstrap 5.1.3 CSS CDN dependency (~25KB)
  - Removed Bootstrap JS bundle dependency (~80KB)
  - Eliminated external framework dependency for improved performance and control

### Added - Modern CSS Architecture
- **Standards-Compliant CSS System**:
  - Custom CSS grid system with responsive breakpoints (576px, 768px, 992px, 1200px, 1400px)
  - CSS Layers (`@layer`) for organized specificity control: reset, base, components, utilities, theme
  - CSS Custom Properties (variables) for consistent theming throughout the application
  - Modern CSS features: `color-mix()`, `color-scheme`, CSS Grid, Flexbox
  - Fallback support for browsers that don't support `color-mix()`

- **New CSS Files**:
  - `variables.css` - CSS custom properties and theme variable mappings
  - `base.css` - Theme-agnostic base styles using CSS layers
  - `bootstrap-replacement.css` - Complete Bootstrap-compatible utility classes and grid system

- **Vanilla JavaScript Components**:
  - `components.js` - Native JS replacements for Bootstrap components
  - Modal, Dropdown, Collapse, Tab, and Alert components
  - Maintains `window.bootstrap` namespace for API compatibility with existing code
  - Lightweight implementation without jQuery dependency

### Added - Modular Theme System
- **TOML-Based Theme Configuration**:
  - Themes defined in `themes/{theme-id}/theme.toml` files
  - Theme validation with comprehensive error checking
  - Support for color schemes: light, dark, auto (system preference)
  - High contrast theme support for accessibility
  - External stylesheet support for CDN integrations

- **Built-in Themes**:
  - `default` (Crusty Light) - Warm orange accents with clean light design
  - `dark` (Crusty Dark) - GitHub-inspired dark theme for low-light environments
  - `high-contrast` - Maximum readability with strong color contrast for accessibility

- **Theme Features**:
  - Dynamic CSS variable generation from TOML configuration
  - Client-side theme switching via `data-theme` attribute
  - System preference detection with `prefers-color-scheme` media query
  - Theme preview colors for picker UI
  - Font customization (body, heading, monospace)
  - Configurable border-radius, shadows, and color palette

- **Theme System Implementation** (`src/lib/theme.rs`):
  - `ThemeConfig` - Parse and validate theme TOML files
  - `ThemeRegistry` - Manage collection of available themes
  - Theme validation (required colors, valid color schemes, hex color format)
  - CSS generation with custom selectors for client-side switching
  - Comprehensive unit tests for theme parsing and validation

### Technical Details
- **Performance**: Reduced CSS/JS bundle size by ~105KB (Bootstrap removal)
- **Standards Compliance**: Uses modern CSS features with graceful fallbacks
- **Accessibility**: High contrast theme with WCAG-compliant color ratios
- **Maintainability**: Single source of truth for theme colors in TOML files
- **Testing**: Theme validation tests ensure consistent theme quality
- **Browser Support**: Fallbacks for older browsers without `color-mix()` support

### Migration Notes
- No breaking changes to existing functionality
- Existing templates continue to work with Bootstrap-compatible class names
- JavaScript components maintain API compatibility via `window.bootstrap` namespace

## [2.15.0] - 2025-12-13

### Added - Newsletter Email Delivery & Subscriber Management
- **Complete Newsletter Email Delivery**:
  - Implemented `send_newsletter_confirmation` and `send_newsletter_issue` methods in EmailService
  - Newsletter confirmation emails sent automatically on subscription
  - Idempotent newsletter delivery (checks delivery_logs before sending)
  - Personalized content based on followed authors' recent articles
  - Responsive HTML email templates for both confirmation and newsletter emails

- **Subscriber Management in Admin Panel**:
  - `GET /api/admin/newsletters/subscribers` - List all subscribers
  - `DELETE /api/admin/newsletters/subscribers/:id` - Remove subscribers
  - Subscriber list shows email, name, and status (Confirmed/Pending/Unsubscribed)
  - Delete functionality with responsive design for mobile

- **Improved Unsubscribe Experience**:
  - Unsubscribe link now redirects to styled `/newsletter/unsubscribed/{token}` page
  - Users see a nicely formatted "You've Been Unsubscribed" page instead of raw JSON
  - Page performs actual unsubscription (similar to confirmation page pattern)

### Fixed - Newsletter Subscription Bug
- **Critical Fix**: Confirmation page now actually confirms newsletter subscriptions
  - Previously the page displayed success without updating database (`confirmed = 1`)
  - Users who clicked confirmation link were never actually confirmed
  - Now properly updates database so subscribers receive newsletters

### Added - Real User Profile Pages
- **Complete Profile System**:
  - `GET /profiles/{username}` - Public user profile pages with real data
  - `GET /profile` - Redirect to authenticated user's own profile
  - Fetches actual articles, follower/following counts from database
  - Returns 404 if user doesn't exist (instead of mock data)
  - Optional authentication for follow button functionality

- **Profile Page Optimizations**:
  - Replaced N+1 queries with single optimized SQL query using JOINs and GROUP BY
  - Consolidated profile stats (articles, followers, following) into single query
  - Added pagination support with page query parameter (10 articles/page)

### Fixed - User Experience Improvements
- **Login Redirect**: Non-admin users now redirect to `/articles` instead of `/admin` after login
- **404 Pages**: Now preserve login state and show logged-in user in navbar
- **Edit Profile Link**: Fixed link from `/settings` to `/account` (the actual route)
- **Footer Socials**: Fixed missing links in social media icons in footer

### Added - Authors Discovery Enhancements
- **Redesigned Authors Page**:
  - New sidebar layout for better navigation
  - Fully functional "Load More" pagination
  - Toast notifications for follow/unfollow actions
  - Enhanced visual design with hover effects and animations
  - Informational sidebar explaining the follow feature
  - Improved search UX with clear button and result count
  - Quick links to related pages (Feed, Favorites, Articles)

- **Authors List Fix**: Now filters to show only authors and admins (excludes subscribers)

### Added - SEO Improvements
- **Sitemap Fixes**:
  - Corrected lastmod date format for W3C compliance (RFC 3339 with colon in timezone)
  - Exclude draft articles from sitemap (fixes Google Search Console 404 errors)

- **Search Engine Optimization**:
  - Canonical URL tags on all public pages
  - Meta description tags for better search snippets
  - Robots noindex for paginated article list pages
  - JSON-LD structured data for articles (BlogPosting schema)

- **Code Quality**:
  - Consolidated duplicate `escape_xml` functions into shared `xml.rs` module

### Changed - Test Infrastructure
- **Improved Test Helpers**:
  - Added `TestCommentBuilder` trait with `add_comment` helper method
  - Added `create_draft_article` and `create_draft_article_simple` to TestArticleBuilder
  - Added `assert_navbar_authenticated` for navbar auth state assertions
  - Added `assert_js_loaded` and `APP_VERSION` for JS cache-busting verification

- **Test Code Reduction**:
  - Refactored `template_rendering.rs` (39% code reduction)
  - Refactored `sitemap.rs` (20% code reduction)
  - Refactored `rss.rs` (15% code reduction)
  - Refactored `feed_page.rs` to use helpers
  - Net reduction: 247 lines removed while maintaining coverage

### Test Coverage
- Added 6 new comprehensive newsletter tests:
  - Idempotent delivery
  - Delivery logs verification
  - Resubscribe flow
  - Send with no subscribers
  - Update newsletter
  - Delete newsletter
- Added tests for subscriber management endpoints (list and delete)
- **Total test count**: 323 tests (all passing)

### Technical Details
- **Performance**: Optimized database queries reduce load times on profile pages
- **Security**: Proper email confirmation prevents spam subscriptions
- **User Experience**: Polished unsubscribe flow and improved navigation
- **Code Quality**: Cleaner test infrastructure with reusable helpers

## [2.12.3] - 2025-11-23

### Changed - Error Handling Refactor
- **Unified Error Architecture**:
  - Refactored entire codebase to use common `ApiError` type throughout
  - Standardized all error responses to use `ApiResponse` type
  - Eliminated duplication in error handling across routes
  - Consolidated error types from multiple domains into single `ApiError` enum

- **Improved Error Types**:
  - BadRequest, NotFound, Unauthorized, Forbidden, Conflict, UnprocessableEntity, InternalServerError
  - Template-specific error handling with Tera error integration
  - Database error sanitization with user-friendly messages
  - Automatic error logging for debugging while protecting sensitive details

- **Enhanced Developer Experience**:
  - Consistent error handling patterns across all route handlers
  - Simplified error propagation with `?` operator
  - Better separation of concerns between domain and HTTP errors
  - Improved code maintainability and readability

### Technical Details
- **Files Updated**: All route handlers (account.rs, admin_users.rs, api_keys.rs, articles.rs, auth.rs, categories.rs, comments.rs, index.rs, media.rs, newsletters.rs, profile.rs, rss.rs, sitemap.rs, tags.rs, users.rs)
- **Code Quality**: Eliminated error handling duplication, cleaner error propagation
- **Testing**: All 283 tests continue to pass with new error handling
- **Performance**: No performance impact, cleaner code paths

## [2.12.0] - 2025-11-19

### Added - Role-Based Access Control (RBAC) System
- **Production-Ready RBAC Implementation**:
  - Three distinct user roles: Admin, Author, Subscriber
  - Hierarchical permission management (Admin > Author > Subscriber)
  - Role-based route protection with custom extractors
  - First user automatically promoted to admin
  - Comprehensive role management API

- **Role Enum and Permissions**:
  - Created Role enum with Admin, Author, Subscriber variants
  - Implemented hierarchical permission checking
  - Role helper methods: `is_admin()`, `is_author()`, `is_subscriber()`
  - 5 comprehensive unit tests for role logic

- **Database Schema Updates**:
  - Added `role` column to users table (TEXT, default 'subscriber')
  - Backward-compatible migration for existing databases
  - All user queries updated to include role field
  - Indexed for efficient role-based queries

- **Authentication & Authorization**:
  - Extended `AuthenticatedUser` extractor to fetch and cache user role
  - Created `AdminUser` extractor for admin-only route protection
  - Created `AuthorUser` extractor for author/admin route protection
  - All role checks performed server-side via database lookup
  - Role information never exposed in JWT claims

- **Route Protection Updates**:
  - **Admin-Only Endpoints**: `/api/admin/users/*` (user management)
  - **Author-Required Endpoints**: Article CRUD, category CRUD, tag management, newsletter management
  - **Subscriber Endpoints**: Read-only access, profile management, following, favorites

- **Role Management Features**:
  - Admins can promote/demote users between roles
  - Role management UI in admin users page
  - Color-coded role badges (Admin: red, Author: blue, Subscriber: gray)
  - Role dropdown with descriptions in edit user modal
  - Role updates via `PUT /api/admin/users/:id` with 'role' field

- **First-User Admin Assignment**:
  - Automatic admin role for first registered user
  - Subsequent users default to subscriber role
  - Atomic user count check during registration

- **User Interface Updates**:
  - Role column in admin users table
  - Role selection dropdown in user edit modal
  - Color-coded role badges for visual distinction
  - Role descriptions for clarity

### Test Coverage
- Added 16 comprehensive RBAC integration tests:
  - First user gets admin role
  - Subsequent users get subscriber role
  - Role included in login/current user responses
  - Subscriber cannot create articles/categories
  - Author can create articles/categories after promotion
  - Admin has full access to all operations
  - Admin can manage user roles (promote/demote)
  - Non-admins cannot access admin endpoints
  - Non-admins cannot modify user roles
  - Invalid role values are rejected
- Updated all existing tests for RBAC compatibility
- Added role promotion helpers to test infrastructure
- **Total test count**: 283 tests (264 integration + 19 unit)

### Security Features
- **Server-Side Role Verification**: Role fetched fresh from database on each request
- **No JWT Role Claims**: Role information never exposed in token claims
- **Privilege Escalation Protection**: Guards against horizontal and vertical privilege escalation
- **Disabled User Protection**: Disabled users cannot authenticate (role check fails)
- **Hierarchical Permissions**: Clear permission hierarchy prevents unauthorized access

### Breaking Changes
- **Article/Category Creation**: Now requires Author role (or higher)
- **Admin Endpoints**: Require Admin role instead of any authentication
- **Tag Management**: Now requires Author role (or higher)
- **Newsletter Management**: Now requires Author role (or higher)

### Migration Notes
For existing deployments:
1. Database migration adds 'role' column automatically
2. Existing users default to 'subscriber' role
3. Admins should manually promote appropriate users to 'author' or 'admin'
4. First new user registration after deployment gets 'admin' automatically

### Technical Details
- **Performance**: Role fetched via indexed database query (minimal overhead)
- **Caching**: Role cached in request lifetime via extractor
- **No N+1 Queries**: Existing database indexes support role queries efficiently
- **Code Quality**: Clean separation of authentication and authorization concerns

### Benefits
- **Granular Access Control**: Fine-grained permission management
- **Security**: Production-ready role-based security model
- **Scalability**: Support for multi-tenant scenarios with different permission levels
- **User Management**: Admins can easily manage user permissions
- **Compliance**: Clear audit trail for role-based actions

## [2.9.0] - 2025-11-17

### Added - Newsletter Subscription and Delivery Service
- **Newsletter Subscription System** (TDD Implementation):
  - Email subscription with comprehensive validation
  - Double opt-in confirmation pattern for GDPR compliance
  - Unique confirmation and unsubscribe tokens (UUID-based)
  - Support for re-subscription after unsubscribing
  - Idempotent confirmation handling
  - Public subscription page at `/newsletter` with responsive Bootstrap UI
  - Confirmation success page at `/newsletter/confirmed/{token}`
  - Unsubscribe success page at `/newsletter/unsubscribed/{token}` with re-subscribe option

- **Newsletter Management** (Admin Features):
  - Create, read, update, delete newsletter issues
  - Draft and sent status tracking
  - Scheduled newsletter support (scheduled_at field)
  - Admin dashboard at `/admin/newsletters` with statistics
  - Real-time newsletter statistics (total subscribers, confirmed subscribers, issues sent)
  - Send newsletters to all confirmed subscribers
  - Delivery logging for each newsletter send
  - Recipient count tracking

- **Database Schema**:
  - `newsletter_subscribers` table with confirmation/unsubscribe tokens
  - `newsletter_issues` table for newsletter content and metadata
  - `newsletter_delivery_logs` table for tracking deliveries
  - Proper indexes for performance optimization

- **API Endpoints**:
  - Public: `/api/newsletters/subscribe`, `/api/newsletters/confirm/{token}`, `/api/newsletters/unsubscribe/{token}`
  - Admin (protected): `/api/admin/newsletters` (CRUD), `/api/admin/newsletters/{id}/send`, `/api/admin/newsletters/stats`
  - Both POST and GET support for confirmation and unsubscribe links (email-friendly)

### Test Coverage
- Added 16 comprehensive integration tests:
  - Newsletter subscription with validation
  - Duplicate subscription handling
  - Confirmation flow with double opt-in
  - Already confirmed subscription handling (idempotent)
  - Unsubscribe functionality
  - Invalid token handling
  - Newsletter creation (authentication required)
  - Newsletter listing and retrieval
  - Newsletter statistics
  - Sending newsletters to confirmed subscribers
  - Page accessibility tests (newsletter and admin pages)

### Technical Details
- **Security**: Email validation, token-based confirmation, authentication for admin endpoints
- **User Experience**: Responsive Bootstrap templates, AJAX form submission, clear success/error messages
- **Data Integrity**: Unique constraints, proper foreign keys, transaction safety
- **Performance**: Indexed queries for subscriber and newsletter lookups
- **Code Quality**: Zero compiler warnings, comprehensive error handling

### Benefits
- **Audience Engagement**: Build and manage email subscriber list
- **Content Distribution**: Send newsletters to confirmed subscribers
- **Privacy Compliance**: Double opt-in pattern respects user consent
- **Analytics**: Track subscriber counts and newsletter delivery
- **Professional UX**: Polished subscription and management interfaces

## [2.8.0] - 2025-11-12

### Added - Article Preview Modal
- **Publishing Preview Feature**:
  - Preview button in article editor with eye icon
  - Large modal (modal-xl) showing formatted article preview
  - Client-side markdown rendering using marked.js library
  - Real-time preview of article exactly as it will appear when published
  - Displays formatted title, description, tags, and rendered body content
  - HTML escaping for security (title, description, tags)
  - Scrollable content for long articles
  - Close button to return to editing

- **Implementation Details**:
  - `showPreview()` JavaScript function for rendering preview
  - Integration with existing editor workflow
  - No server-side processing needed for preview
  - Maintains draft status during preview

### Benefits
- **Author Confidence**: See exactly how article will look before publishing
- **Quality Assurance**: Catch formatting issues before going live
- **User Experience**: Seamless preview without leaving the editor
- **Performance**: Fast client-side rendering with no API calls

### Technical Details
- **Library**: marked.js for markdown-to-HTML conversion
- **Security**: HTML escaping prevents XSS attacks in preview
- **Responsive**: Large modal adapts to different screen sizes
- **Integration**: Works with both new articles and edits

## [2.7.0] - 2025-11-11

### Fixed - Admin Dashboard Pagination Bug
- **Draft Articles Pagination Issue**:
  - Fixed bug where pagination on admin dashboard caused errors with draft articles
  - Resolved database query issues when mixing published and draft articles
  - Improved filtering logic for draft status in paginated queries
  - Enhanced error handling for edge cases

### Technical Details
- **Issue**: Pagination broke when users had both published and draft articles
- **Solution**: Updated query logic to properly handle draft filtering with pagination
- **Testing**: Added test coverage for pagination with mixed article types
- **Impact**: Admin dashboard now reliably displays all articles with correct pagination

## [2.6.0] - 2025-11-11

### Added - Admin Dashboard Pagination
- **Paginated Article Management**:
  - Added pagination controls to admin dashboard
  - Displays articles in manageable chunks for better performance
  - Navigation controls for browsing through multiple pages
  - Consistent with articles listing pagination pattern
  - Improved layout and user experience on dashboard

- **Benefits**:
  - Faster page load times for users with many articles
  - Better organization of content management interface
  - Consistent pagination experience across the application
  - Improved usability for high-volume content creators

### Technical Details
- **Implementation**: Follows same pagination pattern as `/articles` page
- **Performance**: Reduces initial load time by limiting displayed articles
- **User Experience**: Easy navigation between pages of articles
- **Integration**: Seamlessly integrated with existing admin dashboard

## [2.5.0] - 2025-11-11

### Added - Pagination and Draft Widget
- **Articles Page Pagination**:
  - Modified `/articles` route to show 10 articles per page (optimized from 20)
  - Bootstrap-styled pagination controls with page numbers, previous/next buttons
  - Total count query for accurate page calculation
  - Pagination metadata passed to template context
  - Responsive pagination UI matching existing design patterns
  - Query parameter support: `GET /articles?page=N`

- **Landing Page Draft Widget**:
  - New widget showing draft article count for authenticated users
  - Modified total articles count to exclude drafts (published articles only)
  - Database query to count user's draft articles
  - Conditional display based on authentication status
  - "View Drafts" link navigates to `/admin/drafts` when drafts exist
  - Empty state handling when user has zero drafts
  - Responsive layout integrated with existing widgets

### Test Coverage
- Added 4 comprehensive pagination tests:
  - Default page display (10 articles per page)
  - Multi-page navigation with correct article subsets
  - API endpoint pagination validation
  - No pagination controls when articles fit on one page
- Added 4 draft widget tests:
  - Widget visibility for authenticated users with drafts
  - Widget hidden for guest users
  - Widget hidden when user has zero drafts
  - Proper separation of published vs draft article counts
- All new tests passing (8/8)
- **Total test count**: 222 integration tests + 19 unit tests

### Technical Details
- **User Experience**: Improved browsability with pagination controls
- **Performance**: Reduced initial page load by limiting results per page
- **Author Dashboard**: Real-time draft count visibility on homepage
- **Implementation**: Built following existing TDD patterns

## [2.4.0] - 2025-11-11

### Added - Markdown File Upload
- **Direct Markdown File Upload in Editor**:
  - Upload `.md` files directly in the article editor
  - Automatic parsing and extraction of title, description, and body content
  - File validation (markdown extension required, 1MB size limit)
  - Progress indicator during upload
  - Success/error messages for user feedback
  - Seamless integration with existing editor workflow

- **API Endpoint**:
  - `POST /api/articles/upload-markdown` - Process markdown file uploads (protected)
  - Smart metadata extraction from markdown frontmatter or content
  - Defaults to "Untitled Article" when title not found in markdown
  - Returns parsed title, description, and full body content as JSON

- **Frontend Features**:
  - File upload UI in editor template
  - JavaScript handling for file selection, upload, and form population
  - Automatic form field population with extracted data
  - Error handling for invalid files and size limits

### Test Coverage
- 6 comprehensive integration tests:
  - Happy path with proper markdown parsing
  - Markdown without title (defaults to "Untitled Article")
  - Authentication requirement validation
  - Invalid file type rejection
  - File size limit enforcement (1MB)
  - Complex markdown with code blocks and formatting

### Technical Details
- **Workflow Enhancement**: Quick import of markdown drafts for editing
- **Parser**: Custom `parse_markdown_metadata()` function for content extraction
- **Security**: File type and size validation prevents abuse
- **Integration**: Works seamlessly with existing article creation workflow

## [2.3.0] - 2025-11-11

### Added - Drafts Management System
- **Complete Drafts Management Panel**:
  - Dedicated drafts admin page at `/admin/drafts`
  - Table view showing all user's draft articles
  - Display: title, description, tags, created/updated timestamps
  - One-click publish functionality to make drafts public
  - Edit, preview, and delete actions for each draft
  - Empty state message for users with no drafts
  - Bootstrap modal confirmations for destructive actions
  - Real-time UI updates with fade animations

- **API Endpoints**:
  - `GET /api/articles/drafts` - List authenticated user's draft articles (protected)
  - Returns only the user's own drafts with full metadata

- **Frontend Integration**:
  - Modular JavaScript (`drafts.js`) for drafts management
  - Publish draft functionality (updates draft status to false)
  - Delete draft functionality with confirmation
  - Interactive UI with real-time updates
  - "Draft Articles" card on admin dashboard for easy navigation

- **Access Control**:
  - Only article authors can see their own drafts
  - Drafts completely hidden from other users
  - Authentication required for all draft operations

### Test Coverage
- 8+ comprehensive integration tests covering:
  - Creating draft articles
  - Listing user drafts (API and page)
  - Publishing drafts (draft to published status change)
  - Draft visibility (author-only access)
  - Drafts excluded from public listings
  - Deleting drafts
  - Authentication requirements
  - Empty drafts state handling

### Technical Details
- **Workflow**: Streamlined draft-to-publish workflow for authors
- **Security**: Proper authorization and ownership validation
- **User Experience**: Clear separation between drafts and published content
- **Integration**: Seamless integration with existing article management system

## [2.2.0] - 2025-11-10

### Added - Draft Status Feature (TDD Implementation)
- **Complete Draft/Published Status System**:
  - Articles can be saved as drafts or published
  - Draft status field added to all article-related data structures
  - Default behavior: new articles are published (backward compatible)
  - Flexible status changes: can toggle between draft and published

- **Database Changes**:
  - Added `draft` column to articles table (INTEGER: 0=published, 1=draft)
  - Column defaults to 0 (published) for backward compatibility
  - Migration automatically applied to existing installations

- **Model Updates**:
  - Added `draft` field to Article, ArticleResponse, CreateArticle, UpdateArticle
  - Proper serialization with camelCase for API consistency
  - Boolean representation: false=published, true=draft

- **API Endpoints Updated**:
  - `POST /api/articles` - Accept draft status on creation
  - `PUT /api/articles/{slug}` - Allow changing draft status on update
  - `GET /api/articles/{slug}` - Added optional authentication to check draft visibility
  - `GET /api/articles` - Filter out all drafts from public listings
  - `GET /api/articles/feed` - Show user's own drafts, exclude others' drafts
  - `GET /api/search?q={query}` - Filter out all drafts from search results

- **Authorization & Visibility Rules**:
  - Only article author can view their own drafts
  - Drafts completely hidden from other users
  - Public endpoints (list, search, feed) automatically exclude drafts
  - Draft articles inaccessible via direct URL to non-authors

- **Frontend Integration**:
  - Draft checkbox in article editor
  - Draft status indicator on admin dashboard
  - Visual distinction between draft and published articles
  - "Save as Draft" functionality in editor

### Test Coverage
- 11 comprehensive integration tests:
  - Creating articles as drafts
  - Updating draft status
  - Draft visibility and authorization
  - Draft filtering in list/search/feed endpoints
  - Author-only access to own drafts
  - Non-author blocking from viewing drafts
  - Idempotency testing for draft operations

### Technical Details
- **TDD Approach**: Tests written first, then implementation
- **Backward Compatibility**: Existing articles default to published
- **Security**: Proper authorization prevents draft leakage
- **Data Integrity**: Database constraints ensure valid draft status

## [2.2.0-alpha] - 2025-10-25

### Added - Shortcode System for Internal Article Linking
- **Shortcode Syntax**: `[[article:slug]]` or `[[article:slug|Custom Text]]`
- **Processing Pipeline**:
  - Parse shortcodes from article body at submission time
  - Query database for referenced article titles
  - Replace shortcodes with markdown links
  - Store processed markdown in database

- **Markdown Link Generation**:
  - If article found: `[Article Title](/articles/slug)`
  - If article not found: `[slug](/articles/slug "Article not found")`
  - If custom text provided: `[Custom Text](/articles/slug)`

- **Admin Dashboard Enhancement**:
  - Copy-to-clipboard functionality for article slugs
  - Quick shortcode generation for internal linking
  - Interactive buttons for each article in dashboard

- **Implementation Details**:
  - New `src/lib/shortcodes.rs` module with regex-based parser
  - `parse_shortcodes()` function extracts all shortcodes from text
  - `process_shortcodes()` async function resolves shortcodes via database
  - Integration with article creation and update workflows

### Test Coverage
- 15+ comprehensive tests covering:
  - Shortcode parsing (basic, custom text, multiple shortcodes)
  - Database resolution (existing and non-existing articles)
  - Full text processing integration
  - Admin dashboard clipboard functionality
  - Edge cases (malformed shortcodes, empty slugs)

### Technical Details
- **Use Case**: Simplifies internal article cross-referencing
- **Author Workflow**: Copy slug from dashboard, paste as shortcode in editor
- **Processing Time**: Shortcodes resolved at article save time (not render time)
- **Performance**: Regex-based parsing with efficient database queries

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
