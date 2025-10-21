---
title: "[Phase 1] Settings/Configuration UI"
labels: enhancement, phase-1, priority-high
assignees:
milestone: Phase 1 - Content Management Foundations
---

## Summary

Create database-backed settings system for site configuration with admin UI for management.

## Priority

**HIGH** - Foundation for many other features

## Estimated Time

1-2 weeks

## Overview

Implement centralized settings management to allow admins to configure:
- Site information (title, tagline, logo)
- Reading settings (posts per page)
- Discussion settings (comments on/off)
- User settings (registration, default role)
- Permalink structure

## Implementation Tasks

### Database

- [ ] Create `settings` table with key-value storage
- [ ] Add type field for proper value parsing
- [ ] Seed default settings
- [ ] Create database migration

### Backend Models

- [ ] Create `Setting` struct
- [ ] Add helper methods (as_bool, as_i64, as_string)
- [ ] Create settings cache system (optional optimization)

### Backend Routes

- [ ] Create `src/lib/routes/settings.rs`
- [ ] Implement GET /api/settings (admin only)
- [ ] Implement PUT /api/settings (admin only)
- [ ] Implement GET /api/settings/public (public settings only)

### Settings Integration

- [ ] Use settings in article listing (posts_per_page)
- [ ] Use settings in registration (allow_registration)
- [ ] Use settings in templates (site_title, site_tagline)
- [ ] Add settings validation

### Frontend

- [ ] Create settings page (`templates/admin/settings.html`)
- [ ] Create settings JavaScript (`static/js/settings-admin.js`)
- [ ] Organize settings into sections:
  - General (site title, tagline)
  - Reading (posts per page)
  - Discussion (comments)
  - Users (registration, default role)
- [ ] Add setting descriptions and help text
- [ ] Add validation on frontend

### Testing

- [ ] Test get all settings (admin)
- [ ] Test update settings (admin)
- [ ] Test non-admin cannot access settings
- [ ] Test settings applied in article listing
- [ ] Test public settings accessible without auth
- [ ] Test invalid values rejected

## Database Schema

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
    ('site_logo', '', 'string', 'URL to site logo', CURRENT_TIMESTAMP),
    ('posts_per_page', '20', 'number', 'Number of posts per page', CURRENT_TIMESTAMP),
    ('allow_comments', 'true', 'boolean', 'Enable/disable comments globally', CURRENT_TIMESTAMP),
    ('allow_registration', 'true', 'boolean', 'Allow new user registration', CURRENT_TIMESTAMP),
    ('default_role', 'author', 'string', 'Default role for new users', CURRENT_TIMESTAMP),
    ('permalink_structure', '/articles/{slug}', 'string', 'URL structure for articles', CURRENT_TIMESTAMP);
```

## Files to Create

- `src/lib/models/settings.rs` - Setting model
- `src/lib/routes/settings.rs` - Settings routes
- `templates/admin/settings.html` - Settings UI
- `static/js/settings-admin.js` - Settings frontend
- `tests/api/settings.rs` - Settings tests

## Files to Modify

- `src/lib/database.rs` - Add migration
- `src/lib/startup.rs` - Register settings routes
- `src/lib/routes/articles.rs` - Use posts_per_page setting
- `src/lib/routes/users.rs` - Use allow_registration setting
- `templates/base.html` - Use site_title and site_tagline

## API Examples

### Get All Settings (GET /api/settings) - Admin only
```json
{
  "settings": [
    {
      "key": "site_title",
      "value": "CrustyRustacean Dev Blog",
      "type": "string",
      "description": "Site title shown in header"
    },
    {
      "key": "posts_per_page",
      "value": "20",
      "type": "number",
      "description": "Number of posts per page"
    }
  ]
}
```

### Update Settings (PUT /api/settings) - Admin only
```json
{
  "settings": {
    "site_title": "My Awesome Blog",
    "posts_per_page": "25",
    "allow_comments": "false"
  }
}
```

### Get Public Settings (GET /api/settings/public) - No auth
```json
{
  "settings": {
    "site_title": "CrustyRustacean Dev Blog",
    "site_tagline": "A developer blog built with Rust",
    "allow_registration": "true"
  }
}
```

## Setting Categories

### General Settings
- site_title
- site_tagline
- site_logo
- site_description
- site_keywords

### Reading Settings
- posts_per_page
- feed_items
- show_excerpt

### Discussion Settings
- allow_comments
- require_approval
- comment_max_length

### User Settings
- allow_registration
- default_role
- require_email_verification

### Permalink Settings
- permalink_structure
- category_base
- tag_base

## Success Criteria

- ✅ Settings stored in database
- ✅ Admin UI to manage settings
- ✅ Settings applied across application
- ✅ Settings validated before save
- ✅ Public settings accessible without auth
- ✅ All tests pass (6+ new tests)
- ✅ Site title/tagline shown in templates

## Related Documents

- `PROJECT_PLAN.md` - Phase 1, Issue #5

## Dependencies

- Requires user roles & permissions (#4) for admin-only access
