---
title: "[Phase 1] Media Library & Upload System"
labels: enhancement, phase-1, priority-high
assignees:
milestone: Phase 1 - Content Management Foundations
---

## Summary

Implement media library with image upload, management, and integration with existing Apache OpenDAL + Turso storage API.

## Priority

**HIGH** - Foundation for rich content creation

## Estimated Time

3-4 weeks

## Current State

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

## Architecture

```
Blog App → Storage API Client → External Storage API (OpenDAL + Turso)
         ↓
    Local Metadata DB
```

## Implementation Tasks

### Backend

- [ ] Create storage API client wrapper (`src/lib/storage/client.rs`)
- [ ] Create media data models (`src/lib/models/media.rs`)
- [ ] Add media database table with migration
- [ ] Implement media API routes (`src/lib/routes/media.rs`)
  - [ ] POST /api/media/upload (multipart/form-data)
  - [ ] GET /api/media (list with pagination/filtering)
  - [ ] GET /api/media/{id} (get single)
  - [ ] PUT /api/media/{id} (update metadata)
  - [ ] DELETE /api/media/{id} (delete)
- [ ] Add file validation (type, size limits)
- [ ] Update AppState with StorageClient
- [ ] Add configuration for storage API (URL, API key)

### Frontend

- [ ] Create media library UI (`templates/admin/media-library.html`)
- [ ] Build media library JavaScript (`static/js/media-library.js`)
  - [ ] Drag-and-drop upload
  - [ ] Upload progress indicator
  - [ ] Media grid/list view
  - [ ] Search and filtering
- [ ] Integrate media picker into article editor
- [ ] Add "Insert Media" button to editor toolbar

### Testing

- [ ] Write comprehensive tests (`tests/api/media.rs`)
  - [ ] Upload image successfully
  - [ ] Upload rejected for invalid file type
  - [ ] Upload rejected for oversized file
  - [ ] List media with pagination
  - [ ] Filter media by mime type
  - [ ] Get single media file
  - [ ] Update media metadata (alt text, caption)
  - [ ] Delete media file (authorized)
  - [ ] Delete media file rejected (unauthorized)
  - [ ] Unauthenticated request rejected

### Configuration & Documentation

- [ ] Set up storage API credentials in Shuttle secrets
- [ ] Add dependencies (multer, reqwest)
- [ ] Update CODEBASE_INDEX.md
- [ ] Update README.md with media API documentation

## Files to Create

- `src/lib/storage/mod.rs`
- `src/lib/storage/client.rs`
- `src/lib/models/media.rs`
- `src/lib/routes/media.rs`
- `templates/admin/media-library.html`
- `static/js/media-library.js`
- `tests/api/media.rs`

## Files to Modify

- `src/lib/state.rs` - Add StorageClient to AppState
- `src/lib/config.rs` - Add storage API configuration
- `src/lib/startup.rs` - Register media routes
- `static/js/editor.js` - Add media picker integration
- `Cargo.toml` - Add dependencies (multer, reqwest)
- `Shuttle.toml` - Add storage API secrets

## Dependencies (Cargo.toml)

```toml
multer = "3.0"  # Multipart form handling
reqwest = { version = "0.11", features = ["json", "multipart"] }
```

## Configuration (Shuttle.toml)

```toml
[secrets]
STORAGE_API_URL = "https://storage.example.com"
STORAGE_API_KEY = "your-api-key"
```

## Database Schema

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

## Success Criteria

- ✅ Users can upload images via drag-and-drop
- ✅ Uploaded files stored in external storage API
- ✅ Media library UI shows uploaded files
- ✅ Files can be inserted into articles from editor
- ✅ Media can be deleted by uploader
- ✅ File validation prevents invalid uploads
- ✅ All tests pass (15-20 new tests)
- ✅ Documentation updated

## Related Documents

- `PROJECT_PLAN.md` - Phase 1, Issue #2

## Dependencies

None - can be started after tag editing is complete
