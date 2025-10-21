---
title: "[Phase 1] Tag Editing for Articles"
labels: enhancement, phase-1, quick-win, priority-critical
assignees:
milestone: Phase 1 - Content Management Foundations
---

## Summary

Users cannot edit article tags after creation. The frontend sends tags during updates, but the backend ignores them.

## Priority

**CRITICAL** - Quick win, should be completed first

## Estimated Time

2 hours

## Problem

- Frontend editor includes tags input and sends `tagList` on both CREATE and UPDATE
- `CreateArticle` model has `tag_list` field and processes tags correctly
- `UpdateArticle` model **lacks** `tag_list` field
- `update_article()` function ignores tags completely

## Detailed Plan

See `TAG_EDITING_PLAN.md` for comprehensive implementation plan.

## Implementation Tasks

- [ ] Add `tag_list: Option<Vec<String>>` to `UpdateArticle` struct
- [ ] Implement tag update logic in `update_article()` function (replace all tags)
- [ ] Extract shared tag association code to helper function
- [ ] Add comprehensive test coverage (6+ test cases)
- [ ] Update documentation (CODEBASE_INDEX.md, README.md)

## Files to Modify

- `src/lib/models/article.rs:52-57` - Add tag_list field
- `src/lib/routes/articles.rs:615` - Add tag update logic
- `tests/api/articles.rs` - Add tests

## Test Cases

1. Update article with new tags
2. Update article removing all tags (empty array)
3. Update article without touching tags (field not provided)
4. Update article adding tags to tagless article
5. Authorization test (non-author cannot update tags)
6. Idempotency test (update with same tags twice)

## Success Criteria

- ✅ Tags can be edited via PUT /api/articles/{slug}
- ✅ All existing tests pass
- ✅ 6+ new tests for tag editing scenarios
- ✅ Documentation updated
- ✅ Backward compatible (tagList optional)

## Related Documents

- `TAG_EDITING_PLAN.md` - Detailed implementation plan
- `PROJECT_PLAN.md` - Phase 0, Issue #1

## Dependencies

None - can be implemented immediately
