---
title: "[Phase 1] User Roles & Permissions"
labels: enhancement, phase-1, priority-high
assignees:
milestone: Phase 1 - Content Management Foundations
---

## Summary

Implement role-based access control (RBAC) to support multi-user CMS workflows with different permission levels.

## Priority

**HIGH** - Required for multi-user platform

## Estimated Time

2-3 weeks

## Overview

Add user roles and permissions system to enable:
- Different access levels for users
- Fine-grained permission controls
- Admin user management
- Content ownership and editing rights

## Role Hierarchy

1. **Subscriber** (Level 1) - View published content, comment, manage own profile
2. **Author** (Level 2) - Create/edit/delete own articles, upload media
3. **Editor** (Level 3) - Edit/publish any article, moderate comments, manage tags
4. **Admin** (Level 4) - Full system access, manage users, settings, themes

## Implementation Tasks

### Database

- [ ] Add `role` field to users table
- [ ] Create index on role field
- [ ] Create `role_permissions` table (optional, for fine-grained control)
- [ ] Seed default permissions
- [ ] Create migration

### Backend Models

- [ ] Create `UserRole` enum (Subscriber, Author, Editor, Admin)
- [ ] Add `role` field to `User` struct
- [ ] Add role level comparison methods
- [ ] Update JWT claims to include role

### Permission System

- [ ] Create `Permission` enum
- [ ] Create `src/lib/permissions.rs` module
- [ ] Implement `check_permission()` function
- [ ] Create `RequireRole` extractor for routes
- [ ] Add permission checks to all protected routes

### Routes & Authorization

- [ ] Update article routes with permission checks
- [ ] Update comment routes with permission checks
- [ ] Update media routes with permission checks
- [ ] Create user management routes (admin only)
  - [ ] GET /api/users (list users)
  - [ ] PUT /api/users/{id}/role (change role)
  - [ ] DELETE /api/users/{id} (deactivate user)

### Frontend

- [ ] Create user management UI (`templates/admin/users.html`)
- [ ] Create user management JavaScript (`static/js/users-admin.js`)
- [ ] Add role selector to user edit form
- [ ] Show role badges in UI
- [ ] Hide admin features from non-admins

### Testing

- [ ] Test subscriber cannot create article
- [ ] Test author can create article
- [ ] Test author can edit own article
- [ ] Test author cannot edit others' article
- [ ] Test editor can edit any article
- [ ] Test editor can delete any article
- [ ] Test admin can manage users
- [ ] Test non-admin cannot access user management
- [ ] Test role changes persist
- [ ] Test permission checks work for media upload

## Database Schema

```sql
-- Add role to users
ALTER TABLE users ADD COLUMN role TEXT NOT NULL DEFAULT 'author';
CREATE INDEX idx_users_role ON users(role);

-- Optional: Fine-grained permissions table
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
    ('admin', '*');
```

## Files to Create

- `src/lib/permissions.rs` - Permission system
- `src/lib/routes/admin.rs` - User management routes
- `templates/admin/users.html` - User management UI
- `static/js/users-admin.js` - User management frontend
- `tests/api/permissions.rs` - Permission tests

## Files to Modify

- `src/lib/models/user.rs` - Add UserRole enum and role field
- `src/lib/auth.rs` - Add RequireRole extractor, update JWT claims
- `src/lib/routes/articles.rs` - Add permission checks
- `src/lib/routes/comments.rs` - Add permission checks
- `src/lib/routes/media.rs` - Add permission checks
- `src/lib/database.rs` - Add migration
- `src/lib/startup.rs` - Register admin routes

## API Examples

### Check User Role (GET /api/user)
```json
{
  "user": {
    "id": 1,
    "username": "john",
    "email": "john@example.com",
    "role": "author"
  }
}
```

### List Users (GET /api/users) - Admin only
```json
{
  "users": [
    {
      "id": 1,
      "username": "john",
      "email": "john@example.com",
      "role": "author"
    }
  ],
  "total": 1
}
```

### Update User Role (PUT /api/users/{id}/role) - Admin only
```json
{
  "role": "editor"
}
```

## Success Criteria

- ✅ Users have roles (subscriber, author, editor, admin)
- ✅ Permissions enforced on all protected routes
- ✅ Admins can manage user roles
- ✅ Authors can only edit own content
- ✅ Editors can edit any content
- ✅ All tests pass (10+ new tests)
- ✅ UI shows/hides features based on role
- ✅ JWT includes role information

## Related Documents

- `PROJECT_PLAN.md` - Phase 1, Issue #4

## Dependencies

- Can be implemented in parallel with other Phase 1 features
- Should be completed before Phase 2 features that rely on permissions
