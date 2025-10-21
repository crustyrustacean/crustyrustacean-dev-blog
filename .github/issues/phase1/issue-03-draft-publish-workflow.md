---
title: "[Phase 1] Draft/Publish Workflow"
labels: enhancement, phase-1, priority-high
assignees:
milestone: Phase 1 - Content Management Foundations
---

## Summary

Add article status management to enable drafts, publishing, and scheduled posts.

## Priority

**HIGH** - Essential CMS feature

## Estimated Time

2 weeks

## Overview

Implement article workflow states to allow authors to:
- Save articles as drafts (not publicly visible)
- Publish articles immediately
- Schedule articles for future publication

## Implementation Tasks

### Database

- [ ] Add `status` field to articles table (draft, published, scheduled)
- [ ] Add `published_at` field to articles table
- [ ] Create database migration
- [ ] Add indexes for efficient queries
- [ ] Update existing articles to published status

### Backend Models

- [ ] Create `ArticleStatus` enum (Draft, Published, Scheduled)
- [ ] Add `status` and `published_at` fields to `Article` struct
- [ ] Add `status` and `published_at` to `CreateArticle` struct
- [ ] Add `status` and `published_at` to `UpdateArticle` struct

### Backend Routes

- [ ] Update `list_articles()` to filter by published status
- [ ] Create `list_my_articles()` endpoint (author sees all their articles)
- [ ] Create `preview_article()` endpoint (author can preview drafts)
- [ ] Update queries to respect publish dates
- [ ] Implement scheduled publishing (database trigger or background job)

### Frontend

- [ ] Add status selector to editor UI
- [ ] Add publish date picker for scheduled posts
- [ ] Show article status in article list
- [ ] Add draft/published filter in admin dashboard
- [ ] Show scheduled posts count in dashboard

### Testing

- [ ] Test create article as draft
- [ ] Test publish draft article
- [ ] Test schedule article for future
- [ ] Test scheduled article not visible before publish date
- [ ] Test scheduled article auto-published after date
- [ ] Test draft not visible in public listing
- [ ] Test author can see their drafts
- [ ] Test non-author cannot see others' drafts
- [ ] Test update draft to published
- [ ] Test update published to draft

## Database Schema Changes

```sql
-- Migration
ALTER TABLE articles ADD COLUMN status TEXT NOT NULL DEFAULT 'published';
ALTER TABLE articles ADD COLUMN published_at TEXT;

-- Update existing articles
UPDATE articles SET published_at = created_at WHERE status = 'published';

-- Indexes
CREATE INDEX idx_articles_status ON articles(status);
CREATE INDEX idx_articles_published_at ON articles(published_at);

-- Auto-publish trigger (Option A)
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

## Files to Modify

- `src/lib/models/article.rs` - Add status enum and fields
- `src/lib/routes/articles.rs` - Update queries and add preview endpoint
- `src/lib/database.rs` - Add migration
- `templates/articles/editor.html` - Add status controls
- `static/js/editor.js` - Handle status selection
- `tests/api/articles.rs` - Add workflow tests

## Files to Create (Optional)

- `src/lib/jobs/publisher.rs` - Background job for scheduled publishing (Option B)

## API Changes

### Create Article (POST /api/articles)
```json
{
  "article": {
    "title": "My Article",
    "description": "Description",
    "body": "Content",
    "status": "draft",  // NEW: optional, defaults to draft
    "publishedAt": "2025-10-25T10:00:00Z"  // NEW: optional, for scheduled
  }
}
```

### Update Article (PUT /api/articles/{slug})
```json
{
  "article": {
    "status": "published",  // NEW: change status
    "publishedAt": null     // NEW: update publish date
  }
}
```

## Success Criteria

- ✅ Articles can be saved as drafts
- ✅ Drafts not visible in public listings
- ✅ Authors can view/edit their drafts
- ✅ Articles can be scheduled for future publication
- ✅ Scheduled articles auto-publish at specified time
- ✅ Status visible in admin dashboard
- ✅ All tests pass (10+ new tests)
- ✅ Backward compatible (existing articles become published)

## Related Documents

- `PROJECT_PLAN.md` - Phase 1, Issue #3

## Dependencies

- Should be implemented after tag editing (#1)
- Can be done in parallel with media library (#2)
