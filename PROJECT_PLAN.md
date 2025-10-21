# CrustyRustacean Dev Blog - Project Plan

**Goal:** Transform the current developer blog into a WordPress-like content management platform.

**Created:** 2025-10-21
**Version:** 1.0.0
**Repository:** https://github.com/crustyrustacean/crustyrustacean-dev-blog

---

## Table of Contents

1. [Vision & Goals](#vision--goals)
2. [Current State](#current-state)
3. [Implementation Roadmap](#implementation-roadmap)
4. [Phase 0: Quick Wins](#phase-0-quick-wins)
5. [Phase 1: Content Management Foundations](#phase-1-content-management-foundations)
6. [Phase 2: Core CMS Features](#phase-2-core-cms-features)
7. [Phase 3: Customization & Extensibility](#phase-3-customization--extensibility)
8. [Phase 4: Advanced Features](#phase-4-advanced-features)
9. [Low Priority Features](#low-priority-features)
10. [Implementation Guidelines](#implementation-guidelines)

---

## Vision & Goals

Transform CrustyRustacean Dev Blog from a developer blog into a **full-featured, WordPress-like CMS** while maintaining:

- **Type Safety:** Rust's compile-time guarantees
- **Performance:** Blazing-fast response times
- **Modern Stack:** Async runtime, edge database, modern frontend
- **Production Quality:** Comprehensive testing, security, observability

### Target Feature Parity

- ✅ User management with roles/permissions
- ✅ Rich content creation (WYSIWYG, media library)
- ✅ Flexible page/post system
- ✅ Theme customization
- ✅ Plugin architecture
- ✅ SEO optimization
- ✅ Multi-author workflows

---

## Current State

### What We Have (v1.4.0)

Based on `CODEBASE_INDEX.md`:

✅ **Authentication & Users**
- JWT-based authentication (RS256)
- User registration/login
- Profile management
- Following system

✅ **Content Management**
- Article CRUD with markdown
- Comments system
- Tag-based categorization
- Favorites system
- RSS feed

✅ **Admin Features**
- Admin dashboard
- Tag management (`/admin/tags`)
- Author discovery

✅ **Frontend**
- Responsive Bootstrap UI
- 17 JavaScript modules
- Tera templating
- SEO basics (sitemap, robots.txt)

✅ **Testing**
- 81+ integration tests
- Test-driven development
- ~4,777 lines of test code

### What We're Missing (WordPress Gaps)

❌ **Media Management**
- No image upload
- No media library
- No file management

❌ **Content Workflow**
- No draft/publish states
- No scheduled publishing
- No revisions/history

❌ **CMS Structure**
- No static pages (separate from posts)
- No hierarchical content
- No custom post types

❌ **Customization**
- No theme system
- No visual customization
- No plugin architecture

❌ **User Management**
- No role-based permissions
- No user management UI
- Single role (author)

---

## Implementation Roadmap

### Timeline Overview

| Phase | Duration | Focus Area |
|-------|----------|------------|
| **Phase 0** | 1 week | Quick wins & bug fixes |
| **Phase 1** | 2 months | Content management foundations |
| **Phase 2** | 2 months | Core CMS features |
| **Phase 3** | 2 months | Customization & extensibility |
| **Phase 4** | Ongoing | Advanced features & polish |

**Total to WordPress-like Platform:** ~6-7 months

---

## Phase 0: Quick Wins

**Timeline:** 1 week
**Goal:** Fix immediate UX issues and set foundation

### Issue #1: Tag Editing for Articles ⚡

**Status:** Detailed plan exists in `TAG_EDITING_PLAN.md`

**Problem:** Users cannot edit article tags after creation. Frontend sends tags on update, but backend ignores them.

**Estimated Time:** 2 hours

**Implementation:**
1. Add `tag_list: Option<Vec<String>>` to `UpdateArticle` struct
2. Implement tag update logic in `update_article()` function
3. Extract shared tag association code to helper function
4. Add comprehensive test coverage (6+ test cases)
5. Update documentation

**Files to Modify:**
- `src/lib/models/article.rs:52-57` - Add tag_list field
- `src/lib/routes/articles.rs:615` - Add tag update logic
- `tests/api/articles.rs` - Add tests

**Success Criteria:**
- ✅ Tags can be edited via PUT /api/articles/{slug}
- ✅ All existing tests pass
- ✅ 6+ new tests for tag editing scenarios
- ✅ Documentation updated

**Priority:** **CRITICAL** - Should be completed first

---

## Phase 1: Content Management Foundations

**Timeline:** Months 1-2
**Goal:** Enable rich content creation and proper multi-user workflows

---

### Issue #2: Media Library & Upload System 🔥

**Priority:** HIGH
**Estimated Time:** 3-4 weeks
**Dependencies:** Existing Storage API (Apache OpenDAL + Turso)

#### Current State

**Existing Infrastructure:**
- ✅ Storage API built with Apache OpenDAL
- ✅ Turso database for metadata
- ✅ Backend ready for integration

**Missing Components:**
- ❌ Integration with blog application
- ❌ Frontend upload UI
- ❌ Media browser interface
- ❌ Image optimization/resizing
- ❌ Media management endpoints

#### Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                   Blog Application                       │
│  ┌──────────────────────────────────────────────────┐  │
│  │         Media Routes (/api/media)                 │  │
│  │  - POST /upload (multipart/form-data)             │  │
│  │  - GET /list (pagination, filtering)              │  │
│  │  - GET /{id} (metadata)                           │  │
│  │  - DELETE /{id} (soft delete)                     │  │
│  └────────────────┬─────────────────────────────────┘  │
│                   │                                      │
│                   ▼                                      │
│  ┌──────────────────────────────────────────────────┐  │
│  │         Storage API Client                        │  │
│  │  - Apache OpenDAL integration                     │  │
│  │  - Turso metadata storage                         │  │
│  │  - Upload handling                                │  │
│  │  - File validation                                │  │
│  └────────────────┬─────────────────────────────────┘  │
└────────────────────┼──────────────────────────────────┘
                     │
                     ▼
       ┌─────────────────────────────┐
       │  External Storage API        │
       │  (Separate Project)          │
       │                              │
       │  - OpenDAL backend           │
       │  - Turso metadata DB         │
       │  - S3/Local storage          │
       └──────────────────────────────┘
```

#### Implementation Plan

##### 1. Storage API Integration Layer

**File:** `src/lib/storage/client.rs` (new)

**Purpose:** Client wrapper for external storage API

```rust
use serde::{Deserialize, Serialize};
use reqwest::Client;

#[derive(Clone)]
pub struct StorageClient {
    base_url: String,
    api_key: String,
    client: Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaFile {
    pub id: String,
    pub filename: String,
    pub mime_type: String,
    pub size: u64,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub uploaded_by: i64,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct MediaListResponse {
    pub files: Vec<MediaFile>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
}

impl StorageClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            base_url,
            api_key,
            client: Client::new(),
        }
    }

    pub async fn upload(
        &self,
        file_data: Vec<u8>,
        filename: String,
        mime_type: String,
        user_id: i64,
    ) -> Result<MediaFile, AppError> {
        // Implementation
    }

    pub async fn list(
        &self,
        page: usize,
        per_page: usize,
        mime_type_filter: Option<String>,
    ) -> Result<MediaListResponse, AppError> {
        // Implementation
    }

    pub async fn get(&self, id: &str) -> Result<MediaFile, AppError> {
        // Implementation
    }

    pub async fn delete(&self, id: &str, user_id: i64) -> Result<(), AppError> {
        // Implementation
    }
}
```

**Location:** Create `src/lib/storage/` directory

---

##### 2. Media Data Models

**File:** `src/lib/models/media.rs` (new)

**Purpose:** Local media metadata and request/response types

```rust
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct Media {
    pub id: String,
    pub filename: String,
    pub original_filename: String,
    pub mime_type: String,
    pub size: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub storage_url: String,
    pub thumbnail_url: Option<String>,
    pub uploaded_by: i64,
    pub created_at: String,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMediaMetadata {
    #[validate(length(max = 200))]
    pub alt_text: Option<String>,
    #[validate(length(max = 500))]
    pub caption: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MediaListResponse {
    pub media: Vec<Media>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
}
```

**Database Schema:**

```sql
CREATE TABLE media (
    id TEXT PRIMARY KEY,
    filename TEXT NOT NULL,
    original_filename TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    size INTEGER NOT NULL,
    width INTEGER,
    height INTEGER,
    storage_url TEXT NOT NULL,
    thumbnail_url TEXT,
    uploaded_by INTEGER NOT NULL,
    alt_text TEXT,
    caption TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (uploaded_by) REFERENCES users(id)
);

CREATE INDEX idx_media_uploaded_by ON media(uploaded_by);
CREATE INDEX idx_media_mime_type ON media(mime_type);
```

---

##### 3. Media API Routes

**File:** `src/lib/routes/media.rs` (new)

**Endpoints:**

1. **POST /api/media/upload** - Upload file
   - Protected: requires JWT
   - Multipart form data
   - File validation (type, size)
   - Image optimization
   - Returns media metadata

2. **GET /api/media** - List media files
   - Protected: requires JWT
   - Query params: page, per_page, mime_type
   - Returns paginated list

3. **GET /api/media/{id}** - Get single media file
   - Protected: requires JWT
   - Returns full metadata

4. **PUT /api/media/{id}** - Update metadata
   - Protected: requires JWT
   - Authorization: must be uploader
   - Update alt_text, caption

5. **DELETE /api/media/{id}** - Delete file
   - Protected: requires JWT
   - Authorization: must be uploader or admin
   - Soft delete in database
   - Delete from storage API

**Implementation:**

```rust
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    Json,
};
use crate::{
    auth::AuthenticatedUser,
    errors::AppError,
    models::media::{Media, MediaListResponse, UpdateMediaMetadata},
    state::AppState,
};

// Upload handler
pub async fn upload_media(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    mut multipart: Multipart,
) -> Result<Json<Media>, AppError> {
    // Extract file from multipart
    // Validate file type and size
    // Call storage API client
    // Store metadata in local DB
    // Return media object
}

// List handler
pub async fn list_media(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(params): Query<ListParams>,
) -> Result<Json<MediaListResponse>, AppError> {
    // Query local database
    // Apply filters
    // Return paginated results
}

// Get single
pub async fn get_media(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<Media>, AppError> {
    // Query database by ID
    // Return media object
}

// Update metadata
pub async fn update_media_metadata(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<String>,
    Json(data): Json<UpdateMediaMetadata>,
) -> Result<Json<Media>, AppError> {
    // Check authorization
    // Update alt_text, caption
    // Return updated media
}

// Delete
pub async fn delete_media(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    // Check authorization
    // Call storage API delete
    // Delete from local DB
    // Return 204 No Content
}
```

---

##### 4. Frontend Integration

**File:** `static/js/media-library.js` (new)

**Features:**
- Drag-and-drop file upload
- Upload progress indicator
- Media grid/list view
- Search and filter
- Media selection for editor
- Inline metadata editing

**File:** `templates/admin/media-library.html` (new)

**UI Components:**
- Upload area (drag-and-drop or click)
- Media grid with thumbnails
- Filter controls (images, videos, documents)
- Search bar
- Pagination
- Detail sidebar (metadata, usage)

**Editor Integration:**

Modify `static/js/editor.js` to add media insert functionality:

```javascript
// Add media button to editor toolbar
const mediaButton = document.createElement('button');
mediaButton.textContent = 'Insert Media';
mediaButton.onclick = () => openMediaLibrary();

function openMediaLibrary() {
    // Open modal with media library
    // Allow selection
    // Insert markdown image syntax: ![alt text](url)
}
```

---

##### 5. Image Processing

**Options:**

**Option A: Client-side processing (recommended for MVP)**
- Use browser APIs for resizing before upload
- Simpler backend
- Reduces upload size

**Option B: Server-side processing**
- Add `image` crate to dependencies
- Generate thumbnails on upload
- Multiple size variants

**Recommended:** Start with Option A, add Option B later

---

##### 6. File Validation

**Security Measures:**

```rust
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB
const ALLOWED_MIME_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/gif",
    "image/webp",
    "application/pdf",
];

fn validate_upload(
    mime_type: &str,
    size: u64,
) -> Result<(), AppError> {
    if size > MAX_FILE_SIZE {
        return Err(AppError::BadRequest(
            format!("File too large. Max: {}MB", MAX_FILE_SIZE / 1024 / 1024)
        ));
    }

    if !ALLOWED_MIME_TYPES.contains(&mime_type) {
        return Err(AppError::BadRequest(
            "File type not allowed".to_string()
        ));
    }

    Ok(())
}
```

---

##### 7. Configuration

Add to `src/lib/config.rs`:

```rust
pub struct AppConfig {
    pub jwt_secret: String,
    pub storage_api_url: String,      // NEW
    pub storage_api_key: String,      // NEW
    pub max_upload_size: u64,         // NEW
}
```

Add to `Shuttle.toml`:

```toml
[secrets]
STORAGE_API_URL = "https://storage.example.com"
STORAGE_API_KEY = "your-api-key"
```

---

##### 8. Update AppState

**File:** `src/lib/state.rs`

```rust
use crate::storage::StorageClient;

#[derive(Clone)]
pub struct AppState {
    pub tera: Arc<Tera>,
    pub db: DatabaseConnection,
    pub keys: Keys,
    pub storage: StorageClient,  // NEW
}
```

---

#### Testing Plan

**Test File:** `tests/api/media.rs` (new)

**Test Cases:**

1. ✅ Upload image successfully
2. ✅ Upload rejected for invalid file type
3. ✅ Upload rejected for oversized file
4. ✅ List media with pagination
5. ✅ Filter media by mime type
6. ✅ Get single media file
7. ✅ Update media metadata (alt text, caption)
8. ✅ Delete media file (authorized)
9. ✅ Delete media file rejected (unauthorized)
10. ✅ Unauthenticated request rejected
11. ✅ Media library page loads
12. ✅ Upload from editor interface

**Estimated Tests:** 15-20 tests

---

#### File Structure

```
src/lib/
├── storage/
│   ├── mod.rs
│   └── client.rs          # Storage API client
├── models/
│   └── media.rs           # Media models (NEW)
├── routes/
│   └── media.rs           # Media endpoints (NEW)

templates/
└── admin/
    └── media-library.html # Media library UI (NEW)

static/
└── js/
    └── media-library.js   # Media library frontend (NEW)

tests/api/
└── media.rs               # Media tests (NEW)
```

---

#### Dependencies to Add

Add to `Cargo.toml`:

```toml
[dependencies]
# For multipart form handling
multer = "3.0"

# For image processing (optional, Phase 2)
# image = "0.25"

# HTTP client for storage API
reqwest = { version = "0.11", features = ["json", "multipart"] }
```

---

#### Success Criteria

- ✅ Users can upload images via drag-and-drop
- ✅ Uploaded files stored in external storage API
- ✅ Media library UI shows uploaded files
- ✅ Files can be inserted into articles from editor
- ✅ Media can be deleted by uploader
- ✅ File validation prevents invalid uploads
- ✅ All tests pass (15-20 new tests)
- ✅ Documentation updated

---

#### Integration Checklist

- [ ] Set up storage API credentials in Shuttle secrets
- [ ] Implement StorageClient wrapper
- [ ] Create media database table migration
- [ ] Implement media routes (upload, list, get, update, delete)
- [ ] Build media library UI
- [ ] Add multipart form handling
- [ ] Implement file validation
- [ ] Integrate media picker into article editor
- [ ] Write comprehensive tests
- [ ] Update documentation
- [ ] Deploy and verify

**Estimated Time:** 3-4 weeks

---

### Issue #3: Draft/Publish Workflow 🔥

**Priority:** HIGH
**Estimated Time:** 2 weeks

#### Overview

Add article status management to enable drafts, publishing, and scheduled posts.

#### Database Schema Changes

Add `status` and `published_at` fields to articles table:

```sql
-- Migration
ALTER TABLE articles ADD COLUMN status TEXT NOT NULL DEFAULT 'published';
ALTER TABLE articles ADD COLUMN published_at TEXT;

-- Update existing articles
UPDATE articles SET published_at = created_at WHERE status = 'published';

-- Index for efficient queries
CREATE INDEX idx_articles_status ON articles(status);
CREATE INDEX idx_articles_published_at ON articles(published_at);
```

**Status Values:**
- `draft` - Not visible to public
- `published` - Live and visible
- `scheduled` - Will be published at future date

#### Implementation

**1. Update Article Model**

**File:** `src/lib/models/article.rs`

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Article {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub author_id: i64,
    pub status: ArticleStatus,        // NEW
    pub published_at: Option<String>, // NEW
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ArticleStatus {
    Draft,
    Published,
    Scheduled,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateArticle {
    pub title: String,
    pub description: String,
    pub body: String,
    pub tag_list: Option<Vec<String>>,
    pub status: Option<ArticleStatus>,    // NEW (defaults to draft)
    pub published_at: Option<String>,     // NEW (for scheduled)
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateArticle {
    pub title: Option<String>,
    pub description: Option<String>,
    pub body: Option<String>,
    pub tag_list: Option<Vec<String>>,
    pub status: Option<ArticleStatus>,    // NEW
    pub published_at: Option<String>,     // NEW
}
```

**2. Update Article Routes**

**File:** `src/lib/routes/articles.rs`

Modify queries to filter by status:

```rust
// Public listing - only published articles
pub async fn list_articles(...) -> Result<...> {
    let query = "
        SELECT * FROM articles
        WHERE status = 'published'
        AND (published_at IS NULL OR published_at <= datetime('now'))
        ORDER BY published_at DESC
    ";
    // ...
}

// Author's drafts - show all their articles
pub async fn list_my_articles(...) -> Result<...> {
    let query = "
        SELECT * FROM articles
        WHERE author_id = ?
        ORDER BY updated_at DESC
    ";
    // ...
}

// Preview endpoint - allow viewing drafts by author
pub async fn preview_article(...) -> Result<...> {
    // Check if user is author OR article is published
    // Return article
}
```

**3. Scheduled Publishing**

**Option A: Database trigger (simple)**

```sql
-- Auto-publish scheduled articles
CREATE TRIGGER auto_publish_scheduled
AFTER UPDATE ON articles
FOR EACH ROW
WHEN NEW.status = 'scheduled' AND NEW.published_at <= datetime('now')
BEGIN
    UPDATE articles
    SET status = 'published'
    WHERE id = NEW.id;
END;
```

**Option B: Background job (more control)**

Create `src/lib/jobs/publisher.rs`:

```rust
// Run every minute, check for scheduled articles
pub async fn publish_scheduled_articles(db: &DatabaseConnection) -> Result<()> {
    let query = "
        UPDATE articles
        SET status = 'published'
        WHERE status = 'scheduled'
        AND published_at <= datetime('now')
    ";
    db.execute(query, params![]).await?;
    Ok(())
}
```

**Recommended:** Start with Option A, migrate to Option B if needed

**4. Frontend Changes**

**File:** `static/js/editor.js`

Add status selector and publish date picker:

```javascript
// Status dropdown
<select id="article-status">
    <option value="draft">Draft</option>
    <option value="published">Publish Now</option>
    <option value="scheduled">Schedule</option>
</select>

// Date picker (shown when status = scheduled)
<input type="datetime-local" id="publish-date" />
```

**File:** `templates/articles/editor.html`

Update template to include status controls.

**5. Admin Dashboard Updates**

Show article status in admin dashboard:

- Draft count
- Scheduled posts count
- Recently published

#### Testing

**Test File:** `tests/api/articles.rs`

**New Tests:**

1. Create article as draft
2. Publish draft article
3. Schedule article for future
4. Scheduled article not visible before publish date
5. Scheduled article auto-published after date
6. Draft not visible in public listing
7. Author can see their drafts
8. Non-author cannot see others' drafts
9. Update draft to published
10. Update published to draft

#### Success Criteria

- ✅ Articles can be saved as drafts
- ✅ Drafts not visible in public listings
- ✅ Authors can view/edit their drafts
- ✅ Articles can be scheduled for future publication
- ✅ Scheduled articles auto-publish at specified time
- ✅ Status visible in admin dashboard
- ✅ All tests pass

**Estimated Time:** 2 weeks

---

### Issue #4: User Roles & Permissions 🔥

**Priority:** HIGH
**Estimated Time:** 2-3 weeks

#### Overview

Implement role-based access control (RBAC) to support multi-user CMS workflows.

#### Roles

**Role Hierarchy:**

1. **Subscriber** (Level 1)
   - Can view published content
   - Can comment
   - Can manage own profile

2. **Author** (Level 2)
   - All Subscriber permissions
   - Can create/edit/delete own articles
   - Can upload media
   - Can manage own drafts

3. **Editor** (Level 3)
   - All Author permissions
   - Can edit/publish any article
   - Can moderate comments
   - Can manage tags/categories

4. **Admin** (Level 4)
   - All Editor permissions
   - Can manage users
   - Can change site settings
   - Can manage themes/plugins

#### Database Schema

```sql
-- Add role to users table
ALTER TABLE users ADD COLUMN role TEXT NOT NULL DEFAULT 'author';

-- Index for role-based queries
CREATE INDEX idx_users_role ON users(role);

-- Permissions table (optional, for fine-grained control)
CREATE TABLE role_permissions (
    role TEXT NOT NULL,
    permission TEXT NOT NULL,
    PRIMARY KEY (role, permission)
);

-- Seed default permissions
INSERT INTO role_permissions VALUES
    ('subscriber', 'view_published'),
    ('subscriber', 'comment'),
    ('author', 'view_published'),
    ('author', 'comment'),
    ('author', 'create_article'),
    ('author', 'edit_own_article'),
    ('author', 'delete_own_article'),
    ('author', 'upload_media'),
    ('editor', 'view_published'),
    ('editor', 'comment'),
    ('editor', 'create_article'),
    ('editor', 'edit_any_article'),
    ('editor', 'delete_any_article'),
    ('editor', 'upload_media'),
    ('editor', 'moderate_comments'),
    ('editor', 'manage_tags'),
    ('admin', '*');  -- Admin has all permissions
```

#### Implementation

**1. Role Model**

**File:** `src/lib/models/user.rs`

```rust
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Subscriber,
    Author,
    Editor,
    Admin,
}

impl UserRole {
    pub fn level(&self) -> u8 {
        match self {
            UserRole::Subscriber => 1,
            UserRole::Author => 2,
            UserRole::Editor => 3,
            UserRole::Admin => 4,
        }
    }

    pub fn can(&self, permission: &str) -> bool {
        // Check role_permissions table
        // Or hardcode common permissions
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub role: UserRole,  // NEW
    // ... other fields
}
```

**2. Permission System**

**File:** `src/lib/permissions.rs` (new)

```rust
use crate::models::user::{User, UserRole};
use crate::errors::AppError;

pub enum Permission {
    ViewPublished,
    Comment,
    CreateArticle,
    EditOwnArticle,
    DeleteOwnArticle,
    EditAnyArticle,
    DeleteAnyArticle,
    UploadMedia,
    ModerateComments,
    ManageTags,
    ManageUsers,
    ManageSettings,
}

pub fn check_permission(
    user: &User,
    permission: Permission,
    resource_owner_id: Option<i64>,
) -> Result<(), AppError> {
    match permission {
        Permission::EditOwnArticle | Permission::DeleteOwnArticle => {
            if let Some(owner_id) = resource_owner_id {
                if user.id == owner_id || user.role.level() >= 3 {
                    return Ok(());
                }
            }
            Err(AppError::Forbidden("Insufficient permissions".to_string()))
        }
        Permission::EditAnyArticle | Permission::DeleteAnyArticle => {
            if user.role.level() >= 3 {
                return Ok(());
            }
            Err(AppError::Forbidden("Insufficient permissions".to_string()))
        }
        Permission::ManageUsers | Permission::ManageSettings => {
            if user.role == UserRole::Admin {
                return Ok(());
            }
            Err(AppError::Forbidden("Admin only".to_string()))
        }
        // ... other permissions
        _ => Ok(()),
    }
}
```

**3. Auth Middleware**

**File:** `src/lib/auth.rs`

Add role-based extractors:

```rust
// Require specific role
pub struct RequireRole(pub UserRole);

#[async_trait]
impl<S> FromRequestParts<S> for RequireRole
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthenticatedUser::from_request_parts(parts, state).await?;

        if user.role.level() >= self.0.level() {
            Ok(RequireRole(user.role))
        } else {
            Err(AppError::Forbidden("Insufficient role".to_string()))
        }
    }
}

// Usage in routes:
pub async fn admin_only_route(
    _role: RequireRole(UserRole::Admin),
) -> Result<...> {
    // Only admins can access
}
```

**4. Update Routes**

**File:** `src/lib/routes/articles.rs`

Add permission checks:

```rust
pub async fn update_article(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(slug): Path<String>,
    Json(data): Json<UpdateArticle>,
) -> Result<...> {
    // Get article author
    let article = get_article_by_slug(&state.db, &slug).await?;

    // Check permission
    check_permission(
        &user,
        Permission::EditOwnArticle,
        Some(article.author_id),
    )?;

    // Proceed with update
}
```

**5. User Management UI**

**File:** `templates/admin/users.html` (new)

Admin interface to:
- List all users
- Change user roles
- Activate/deactivate users
- View user statistics

**Route:** `GET /admin/users` (admin only)

**File:** `src/lib/routes/admin.rs`

```rust
pub async fn list_users(
    State(state): State<AppState>,
    _role: RequireRole(UserRole::Admin),
    Query(params): Query<PaginationParams>,
) -> Result<...> {
    // List users with pagination
}

pub async fn update_user_role(
    State(state): State<AppState>,
    _role: RequireRole(UserRole::Admin),
    Path(user_id): Path<i64>,
    Json(data): Json<UpdateUserRole>,
) -> Result<...> {
    // Update user's role
}
```

#### Testing

**Test File:** `tests/api/permissions.rs` (new)

**Test Cases:**

1. Subscriber cannot create article
2. Author can create article
3. Author can edit own article
4. Author cannot edit others' article
5. Editor can edit any article
6. Editor can delete any article
7. Admin can manage users
8. Non-admin cannot access user management
9. Role changes persist
10. Permission checks work for media upload

#### Success Criteria

- ✅ Users have roles (subscriber, author, editor, admin)
- ✅ Permissions enforced on all protected routes
- ✅ Admins can manage user roles
- ✅ Authors can only edit own content
- ✅ Editors can edit any content
- ✅ All tests pass

**Estimated Time:** 2-3 weeks

---

### Issue #5: Settings/Configuration UI 🔥

**Priority:** HIGH
**Estimated Time:** 1-2 weeks

#### Overview

Create database-backed settings system for site configuration.

#### Database Schema

```sql
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    type TEXT NOT NULL,  -- 'string', 'number', 'boolean', 'json'
    description TEXT,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Default settings
INSERT INTO settings VALUES
    ('site_title', 'CrustyRustacean Dev Blog', 'string', 'Site title shown in header', CURRENT_TIMESTAMP),
    ('site_tagline', 'A developer blog built with Rust', 'string', 'Site description', CURRENT_TIMESTAMP),
    ('posts_per_page', '20', 'number', 'Number of posts per page', CURRENT_TIMESTAMP),
    ('allow_comments', 'true', 'boolean', 'Enable/disable comments globally', CURRENT_TIMESTAMP),
    ('allow_registration', 'true', 'boolean', 'Allow new user registration', CURRENT_TIMESTAMP),
    ('default_role', 'author', 'string', 'Default role for new users', CURRENT_TIMESTAMP);
```

#### Implementation

**File:** `src/lib/models/settings.rs` (new)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub setting_type: String,
    pub description: Option<String>,
}

impl Setting {
    pub fn as_bool(&self) -> bool {
        self.value.to_lowercase() == "true"
    }

    pub fn as_i64(&self) -> i64 {
        self.value.parse().unwrap_or(0)
    }

    pub fn as_string(&self) -> String {
        self.value.clone()
    }
}
```

**File:** `src/lib/routes/settings.rs` (new)

```rust
// GET /api/settings (admin only)
pub async fn get_settings(
    State(state): State<AppState>,
    _role: RequireRole(UserRole::Admin),
) -> Result<Json<Vec<Setting>>, AppError> {
    // Return all settings
}

// PUT /api/settings (admin only)
pub async fn update_settings(
    State(state): State<AppState>,
    _role: RequireRole(UserRole::Admin),
    Json(data): Json<HashMap<String, String>>,
) -> Result<StatusCode, AppError> {
    // Update multiple settings
}
```

**File:** `templates/admin/settings.html` (new)

Settings page with sections:
- General (site title, tagline)
- Reading (posts per page)
- Discussion (comments)
- Users (registration, default role)

#### Success Criteria

- ✅ Settings stored in database
- ✅ Admin UI to manage settings
- ✅ Settings applied across application
- ✅ Settings validated before save

**Estimated Time:** 1-2 weeks

---

## Phase 2: Core CMS Features

**Timeline:** Months 3-4
**Goal:** Build WordPress-equivalent content structure

---

### Issue #6: Pages System 🔥

**Priority:** HIGH
**Estimated Time:** 3 weeks

Separate "Pages" from "Posts" (articles). Pages are static content like About, Contact, Terms.

**Features:**
- Hierarchical pages (parent/child)
- Custom templates
- Different permalink structure
- Page tree UI

**Database:**
```sql
CREATE TABLE pages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    parent_id INTEGER,
    template TEXT DEFAULT 'default',
    status TEXT NOT NULL DEFAULT 'published',
    author_id INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_id) REFERENCES pages(id),
    FOREIGN KEY (author_id) REFERENCES users(id)
);
```

**Routes:**
- `GET /pages/{slug}` - View page
- `GET /api/pages` - List pages (hierarchical)
- `POST /api/pages` - Create page
- `PUT /api/pages/{slug}` - Update page
- `DELETE /api/pages/{slug}` - Delete page

---

### Issue #7: Categories System

**Priority:** MEDIUM
**Estimated Time:** 2 weeks

Hierarchical categories to complement tags.

**Database:**
```sql
CREATE TABLE categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    parent_id INTEGER,
    FOREIGN KEY (parent_id) REFERENCES categories(id)
);

CREATE TABLE article_categories (
    article_id INTEGER NOT NULL,
    category_id INTEGER NOT NULL,
    PRIMARY KEY (article_id, category_id),
    FOREIGN KEY (article_id) REFERENCES articles(id),
    FOREIGN KEY (category_id) REFERENCES categories(id)
);
```

**Features:**
- Category CRUD
- Hierarchical structure
- Category archives
- Category widgets

---

### Issue #8: Tag Management Enhancements

**Priority:** MEDIUM
**Estimated Time:** 1 week
**Dependencies:** Tag Editing (#1)

Build on the tag editing feature with advanced management:

**Features:**
- Tag merging (combine duplicates)
- Tag analytics (usage counts, trends)
- Tag descriptions/metadata
- Orphaned tag cleanup
- Tag cloud weight calculation

**Database:**
```sql
ALTER TABLE tags ADD COLUMN description TEXT;
ALTER TABLE tags ADD COLUMN slug TEXT UNIQUE;

-- View for tag usage
CREATE VIEW tag_usage AS
SELECT
    t.id,
    t.name,
    t.description,
    COUNT(at.article_id) as usage_count
FROM tags t
LEFT JOIN article_tags at ON t.id = at.id
GROUP BY t.id;
```

---

### Issue #9: Article Revisions/History

**Priority:** MEDIUM
**Estimated Time:** 2 weeks

Track article changes over time.

**Database:**
```sql
CREATE TABLE article_revisions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    article_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    body TEXT NOT NULL,
    author_id INTEGER NOT NULL,
    revision_number INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (article_id) REFERENCES articles(id),
    FOREIGN KEY (author_id) REFERENCES users(id)
);
```

**Features:**
- Auto-save revisions on edit
- View revision history
- Compare revisions (diff)
- Restore previous version

---

### Issue #10: SEO Enhancements

**Priority:** MEDIUM
**Estimated Time:** 1 week

Improve search engine optimization.

**Features:**
- Custom meta descriptions per article/page
- Open Graph tags
- Twitter Cards
- Structured data (JSON-LD)
- Canonical URLs
- SEO score indicators

**Implementation:**
- Add fields to articles/pages tables
- Update templates with meta tags
- Add JSON-LD generation

---

### Issue #11: Advanced Pagination

**Priority:** LOW
**Estimated Time:** 3 days

Better pagination UX.

**Features:**
- Configurable page size (from settings)
- Infinite scroll option
- Previous/Next article navigation
- Jump to page

---

## Phase 3: Customization & Extensibility

**Timeline:** Months 5-6
**Goal:** Enable visual customization and extensibility

---

### Issue #12: Theme System 🔥

**Priority:** HIGH
**Estimated Time:** 4 weeks

Enable theme switching and customization.

**Structure:**
```
themes/
├── default/
│   ├── theme.toml          # Metadata
│   ├── templates/
│   │   ├── index.html
│   │   ├── article.html
│   │   ├── page.html
│   │   └── ...
│   ├── static/
│   │   ├── css/
│   │   └── js/
│   └── screenshot.png
└── custom-theme/
    └── ...
```

**Features:**
- Theme metadata (name, author, version)
- Template inheritance from default
- Theme-specific assets
- Theme options/customizer
- Live preview

---

### Issue #13: Menu Management System 🔥

**Priority:** HIGH
**Estimated Time:** 2 weeks

Custom navigation menus.

**Database:**
```sql
CREATE TABLE menus (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    location TEXT NOT NULL  -- 'header', 'footer', 'sidebar'
);

CREATE TABLE menu_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    menu_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    url TEXT NOT NULL,
    target TEXT DEFAULT '_self',
    parent_id INTEGER,
    position INTEGER NOT NULL,
    FOREIGN KEY (menu_id) REFERENCES menus(id),
    FOREIGN KEY (parent_id) REFERENCES menu_items(id)
);
```

**Features:**
- Multiple menus
- Drag-and-drop builder
- Link to pages/articles/external
- Nested menu items

---

### Issue #14: Widget System 🔥

**Priority:** HIGH
**Estimated Time:** 3 weeks

Dynamic sidebar content.

**Widget Types:**
- Recent posts
- Tag cloud
- Categories list
- Custom HTML
- Search box

**Database:**
```sql
CREATE TABLE widget_areas (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT
);

CREATE TABLE widgets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    widget_area_id INTEGER NOT NULL,
    type TEXT NOT NULL,
    title TEXT,
    config TEXT,  -- JSON
    position INTEGER NOT NULL,
    FOREIGN KEY (widget_area_id) REFERENCES widget_areas(id)
);
```

---

### Issue #15: WYSIWYG Editor 🔥

**Priority:** MEDIUM
**Estimated Time:** 2 weeks

Replace markdown editor with rich text editor.

**Options:**
- TinyMCE (feature-rich)
- Quill (lightweight)
- Editor.js (block-based)

**Features:**
- Rich text formatting
- Media insertion from library
- Code blocks
- Tables
- Markdown export option

---

### Issue #16: Plugin/Extension Architecture 🔥

**Priority:** HIGH
**Estimated Time:** 4-5 weeks

Extensibility system for custom functionality.

**Hook System:**
```rust
// Hook registry
pub struct HookRegistry {
    hooks: HashMap<String, Vec<Box<dyn Fn(Value) -> Value>>>,
}

// Register hook
hooks.register("before_article_save", |article| {
    // Modify article before save
    article
});

// Trigger hook
let article = hooks.trigger("before_article_save", article);
```

**Plugin Structure:**
```
plugins/
└── my-plugin/
    ├── plugin.toml        # Metadata
    ├── src/
    │   └── lib.rs         # Plugin code
    └── README.md
```

---

## Phase 4: Advanced Features

**Timeline:** Ongoing (Months 7+)
**Goal:** Polish and advanced functionality

---

### Issue #17: Enhanced Admin Dashboard

**Priority:** MEDIUM
**Estimated Time:** 2 weeks

Better admin experience.

**Features:**
- Quick stats (posts, comments, users)
- Recent activity feed
- Quick draft widget
- System health indicators
- Analytics charts

---

### Issue #18: User Profile Enhancements

**Priority:** LOW
**Estimated Time:** 1 week

Richer user profiles.

**Features:**
- Avatar upload (via media library)
- Social media links
- Rich bio (WYSIWYG)
- Author archive pages
- User statistics

---

## Low Priority Features

These can be implemented as needed:

### Email Notifications
- Comment notifications
- Weekly digest
- Password reset
- Welcome emails

### Enhanced Search
- Full-text search
- Filters (date, author, tags)
- Autocomplete
- Search analytics

### Comment Moderation
- Approval workflow
- Spam detection
- Comment editing
- Threaded replies

### Social Sharing
- Share buttons
- Open Graph optimization
- Twitter Cards
- Share counts

### Import/Export
- WordPress XML import
- JSON export
- Markdown export
- Database backup

### Analytics Dashboard
- Page views
- Popular content
- Traffic sources
- User engagement

---

## Implementation Guidelines

### General Principles

1. **Test-Driven Development**
   - Write tests first
   - Maintain >80% coverage
   - Integration tests for features

2. **Incremental Changes**
   - Small, focused PRs
   - Complete one feature before starting next
   - Keep main branch deployable

3. **Documentation**
   - Update CODEBASE_INDEX.md
   - API documentation
   - User guides for new features

4. **Security**
   - Input validation
   - Authorization checks
   - SQL injection prevention
   - XSS protection

5. **Performance**
   - Database indexing
   - Query optimization
   - Asset optimization
   - Caching strategy

### Code Quality Standards

- **Rust:** `cargo clippy` with no warnings
- **Formatting:** `cargo fmt`
- **Tests:** All tests must pass
- **Documentation:** Public APIs documented
- **Error Handling:** Use `AppError` enum consistently

### Git Workflow

1. Create feature branch from main
2. Implement feature with tests
3. Update documentation
4. Create pull request
5. Code review
6. Merge to main
7. Deploy to production

### Dependencies Policy

- Prefer mature, well-maintained crates
- Minimize dependency count
- Review security advisories
- Keep dependencies updated

---

## Success Metrics

### Phase 0
- ✅ Tag editing works
- ✅ No regressions

### Phase 1
- ✅ Media library functional
- ✅ Draft/publish workflow
- ✅ Role-based permissions
- ✅ Settings management

### Phase 2
- ✅ Pages separate from posts
- ✅ Categories implemented
- ✅ Article revisions working

### Phase 3
- ✅ Theme system active
- ✅ Menus customizable
- ✅ Widgets functional

### Phase 4
- ✅ Plugin system working
- ✅ Admin dashboard enhanced

### Overall Goal
- ✅ Feature parity with basic WordPress installation
- ✅ Performance better than WordPress
- ✅ Modern, responsive UI
- ✅ Comprehensive test coverage
- ✅ Production-ready deployment

---

## Resources

### Documentation
- Current: `CODEBASE_INDEX.md`
- Planning: `TAG_EDITING_PLAN.md`
- API: `README.md`

### Related Projects
- Storage API: Apache OpenDAL + Turso backend
- Theme Repository: (Future)
- Plugin Repository: (Future)

### External Dependencies
- Axum documentation: https://docs.rs/axum
- Tera templates: https://keats.github.io/tera/
- Turso database: https://docs.turso.tech/
- OpenDAL: https://opendal.apache.org/

---

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-10-21 | Initial project plan created |

---

**Next Action:** Implement tag editing feature as outlined in `TAG_EDITING_PLAN.md`
