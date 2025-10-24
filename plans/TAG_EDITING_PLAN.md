# Tag Editing Implementation Plan

**Issue:** Article tags cannot be edited after article creation. The frontend sends tags during updates, but the backend ignores them.

**Date:** 2025-10-21
**Status:** Planning Phase

---

## Problem Analysis

### Current State

**Frontend (✅ Already Working):**
- The editor template (`templates/articles/editor.html`) includes a tags input field
- The editor.js JavaScript parses tags and sends them in both CREATE and UPDATE requests
- Tags are sent as `tagList: ["tag1", "tag2"]` in the request payload

**Backend (❌ Issue Found):**
- `CreateArticle` model has `tag_list: Option<Vec<String>>` field
- `UpdateArticle` model **DOES NOT** have a `tag_list` field
- `create_article()` function processes tags correctly (lines 95-138 in articles.rs)
- `update_article()` function **IGNORES** tags completely (lines 530-619 in articles.rs)

### Root Cause

The `UpdateArticle` struct in `src/lib/models/article.rs` (lines 52-57) lacks a `tag_list` field:

```rust
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateArticle {
    pub title: Option<String>,
    pub description: Option<String>,
    pub body: Option<String>,
    // ❌ Missing: tag_list field
}
```

---

## Implementation Plan

### Phase 1: Backend Data Model Updates

**File:** `src/lib/models/article.rs`

**Changes:**
1. Add `tag_list` field to `UpdateArticle` struct
2. Use proper serde rename for camelCase compatibility

**Code Changes:**
```rust
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]  // Add this if not present
pub struct UpdateArticle {
    pub title: Option<String>,
    pub description: Option<String>,
    pub body: Option<String>,
    #[serde(rename = "tagList")]
    pub tag_list: Option<Vec<String>>,  // ✅ Add this field
}
```

**Location:** `src/lib/models/article.rs:52-57`

---

### Phase 2: Backend Tag Update Logic

**File:** `src/lib/routes/articles.rs`

**Changes:** Modify `update_article()` function to handle tag updates

**Implementation Strategy:**

There are two approaches for handling tag updates:

#### **Option A: Replace All Tags (Recommended)**
- Simpler implementation
- Clear semantics: "these are the article's tags now"
- Matches how other fields work (title, description, body)

**Steps:**
1. If `tag_list` is provided in the update request
2. Delete all existing article-tag associations for this article
3. Insert new tag associations (same logic as `create_article`)

#### **Option B: Merge Tags**
- More complex
- Could lead to confusion about expected behavior
- Not recommended

**Recommended Implementation (Option A):**

Add this logic to `update_article()` function after the existing field updates (around line 615):

```rust
// Handle tag updates if provided
if let Some(tag_list) = &article_data.tag_list {
    // Get the article ID first
    let mut id_rows = conn
        .query(
            "SELECT id FROM articles WHERE slug = ?",
            libsql::params![slug.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let id_row = id_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Article not found".to_string()))?;

    let article_id: String = id_row
        .get(0)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Delete existing tag associations
    conn.execute(
        "DELETE FROM article_tags WHERE article_id = ?",
        libsql::params![article_id.clone()],
    )
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Add new tags
    for tag_name in tag_list {
        // Insert tag if it doesn't exist
        let tag_id = Uuid::new_v4();
        conn.execute(
            "INSERT OR IGNORE INTO tags (id, name) VALUES (?, ?)",
            libsql::params![tag_id.to_string(), tag_name.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        // Get the tag ID
        let mut tag_rows = conn
            .query(
                "SELECT id FROM tags WHERE name = ?",
                libsql::params![tag_name.clone()],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        if let Some(tag_row) = tag_rows
            .next()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
        {
            let existing_tag_id: String = tag_row
                .get(0)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;

            // Link article to tag
            conn.execute(
                "INSERT INTO article_tags (article_id, tag_id) VALUES (?, ?)",
                libsql::params![article_id.clone(), existing_tag_id],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        }
    }
}
```

**Location:** `src/lib/routes/articles.rs:615` (before the final `get_article` call)

---

### Phase 3: Code Quality Improvements

**Refactoring Opportunity:**

The tag handling logic is duplicated between `create_article()` and `update_article()`. Consider extracting it to a helper function:

```rust
async fn associate_article_tags(
    conn: &libsql::Connection,
    article_id: &str,
    tag_list: &[String],
) -> Result<(), AppError> {
    for tag_name in tag_list {
        // Insert tag if it doesn't exist
        let tag_id = Uuid::new_v4();
        conn.execute(
            "INSERT OR IGNORE INTO tags (id, name) VALUES (?, ?)",
            libsql::params![tag_id.to_string(), tag_name.clone()],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        // Get the tag ID
        let mut tag_rows = conn
            .query(
                "SELECT id FROM tags WHERE name = ?",
                libsql::params![tag_name.clone()],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        if let Some(tag_row) = tag_rows
            .next()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
        {
            let existing_tag_id: String = tag_row
                .get(0)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;

            // Link article to tag
            conn.execute(
                "INSERT INTO article_tags (article_id, tag_id) VALUES (?, ?)",
                libsql::params![article_id.to_string(), existing_tag_id],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        }
    }
    Ok(())
}
```

Then use it in both functions:
- In `create_article()`: Replace lines 96-138
- In `update_article()`: Use after deleting existing tags

---

### Phase 4: Testing

**Test Coverage Needed:**

1. **Update article with new tags**
   - Create article with tags ["rust", "tutorial"]
   - Update article with tags ["rust", "advanced"]
   - Verify: "tutorial" removed, "advanced" added

2. **Update article removing all tags**
   - Create article with tags ["rust", "web"]
   - Update article with tags []
   - Verify: All tags removed

3. **Update article without touching tags**
   - Create article with tags ["rust"]
   - Update article title only (no tag_list in payload)
   - Verify: Tags unchanged

4. **Update article adding tags to tagless article**
   - Create article with no tags
   - Update article with tags ["new", "tags"]
   - Verify: Tags added correctly

5. **Authorization test**
   - User A creates article with tags
   - User B attempts to update tags
   - Verify: 403 Forbidden (existing auth check should handle this)

6. **Idempotency test**
   - Update article with tags ["a", "b", "c"]
   - Update again with same tags
   - Verify: No errors, tags remain the same

**Test File:** `tests/api/articles.rs`

**Example Test:**
```rust
#[tokio::test]
async fn test_update_article_tags() {
    let app = spawn_app().await;
    let user = app.create_test_user("author", "author@example.com", "password123").await;

    // Create article with initial tags
    let create_payload = json!({
        "article": {
            "title": "Test Article",
            "description": "Test Description",
            "body": "Test Body",
            "tagList": ["rust", "tutorial"]
        }
    });

    let create_response = app.client
        .post(&format!("{}/api/articles", app.address))
        .header("Authorization", format!("Bearer {}", user.token))
        .json(&create_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_status!(create_response, 200);
    let created: serde_json::Value = create_response.json().await.unwrap();
    let slug = created["article"]["slug"].as_str().unwrap();

    // Update article with different tags
    let update_payload = json!({
        "article": {
            "tagList": ["rust", "advanced", "web"]
        }
    });

    let update_response = app.client
        .put(&format!("{}/api/articles/{}", app.address, slug))
        .header("Authorization", format!("Bearer {}", user.token))
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_status!(update_response, 200);
    let updated: serde_json::Value = update_response.json().await.unwrap();

    // Verify tags were updated
    let tags = updated["article"]["tagList"].as_array().unwrap();
    assert_eq!(tags.len(), 3);
    assert!(tags.iter().any(|t| t == "rust"));
    assert!(tags.iter().any(|t| t == "advanced"));
    assert!(tags.iter().any(|t| t == "web"));
    assert!(!tags.iter().any(|t| t == "tutorial")); // Old tag removed
}
```

---

### Phase 5: Documentation Updates

**Files to Update:**

1. **README.md**
   - Update API documentation for PUT /api/articles/{slug}
   - Note that tags can now be updated

2. **CODEBASE_INDEX.md**
   - Update UpdateArticle model documentation
   - Update update_article endpoint description

3. **CHANGELOG.md**
   - Add entry for this feature:
     ```
     ## [Unreleased]
     ### Added
     - Support for editing article tags via PUT /api/articles/{slug}

     ### Changed
     - UpdateArticle model now includes optional tag_list field
     ```

---

## Implementation Steps (Ordered)

1. ✅ **Analysis Complete** - Understand current implementation
2. ⬜ **Update Data Model** - Add tag_list to UpdateArticle struct
3. ⬜ **Implement Tag Update Logic** - Modify update_article() function
4. ⬜ **Add Helper Function (Optional)** - Refactor duplicate code
5. ⬜ **Write Tests** - Add comprehensive test coverage
6. ⬜ **Run Tests** - Verify all tests pass
7. ⬜ **Manual Testing** - Test in dev environment
8. ⬜ **Update Documentation** - README, CODEBASE_INDEX, CHANGELOG
9. ⬜ **Code Review** - Review changes for quality
10. ⬜ **Commit & Push** - Commit with descriptive message
11. ⬜ **Deploy** - Deploy to production (via Shuttle)

---

## Edge Cases to Consider

1. **Empty Tag List**
   - Behavior: Remove all tags from article
   - Implementation: DELETE all article_tags, don't insert any new ones

2. **Duplicate Tags in Request**
   - Example: `["rust", "rust", "web"]`
   - Behavior: Deduplicate before processing
   - Implementation: Use `HashSet` or `Vec::dedup()` after sorting

3. **Tag Case Sensitivity**
   - Current: Tags are case-sensitive ("Rust" != "rust")
   - Decision: Keep current behavior or normalize to lowercase?
   - Recommendation: Normalize to lowercase for consistency

4. **Tag Whitespace**
   - Example: `["  rust  ", "web"]`
   - Behavior: Trim whitespace
   - Implementation: Already handled in frontend (line 21 of editor.js)

5. **Very Long Tag Names**
   - Should we add validation for max tag length?
   - Recommendation: Add validation (e.g., max 50 characters per tag)

6. **Special Characters in Tags**
   - Should tags allow spaces, punctuation, etc.?
   - Current: No validation
   - Recommendation: Add regex validation for alphanumeric + hyphens only

7. **Maximum Number of Tags**
   - Should we limit how many tags per article?
   - Recommendation: Add validation (e.g., max 10 tags per article)

---

## Validation Enhancements (Optional)

Add to `UpdateArticle` struct:

```rust
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateArticle {
    pub title: Option<String>,
    pub description: Option<String>,
    pub body: Option<String>,
    #[serde(rename = "tagList")]
    #[validate(length(max = 10))]  // Max 10 tags per article
    pub tag_list: Option<Vec<String>>,
}
```

Add tag validation function:

```rust
fn validate_tag(tag: &str) -> bool {
    // Allow alphanumeric, hyphens, and underscores
    // Max 50 characters
    if tag.len() > 50 || tag.is_empty() {
        return false;
    }
    tag.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}
```

---

## Database Considerations

### Query Optimization

Current implementation queries the database multiple times per tag. For articles with many tags, this could be slow.

**Optimization (Future Enhancement):**

Use a single query to get all tag IDs:

```rust
// Build comma-separated placeholders
let placeholders = tag_list.iter()
    .map(|_| "?")
    .collect::<Vec<_>>()
    .join(",");

let query = format!("SELECT id, name FROM tags WHERE name IN ({})", placeholders);

// Execute query with all tag names
let mut params = Vec::new();
for tag_name in tag_list {
    params.push(libsql::Value::from(tag_name.clone()));
}

let mut tag_rows = conn.query(&query, params).await?;

// Process results...
```

### Orphaned Tags

After removing tags from articles, some tags might not be associated with any articles.

**Consideration:**
- Should we delete orphaned tags?
- Or keep them for reuse?

**Recommendation:** Keep orphaned tags (current behavior). They can be cleaned up later via admin interface or cron job.

---

## Performance Impact

**Expected Impact:** Minimal

- Tag updates are infrequent compared to reads
- Database operations are simple (INSERT, DELETE)
- SQLite handles small tag sets efficiently
- No impact on article retrieval performance

**Benchmarking (Optional):**
- Measure time to update article with 0, 5, 10 tags
- Ensure update completes in < 100ms for typical cases

---

## API Contract

### Before (Current)

**PUT /api/articles/{slug}**

Request:
```json
{
  "article": {
    "title": "Updated Title",
    "description": "Updated Description",
    "body": "Updated Body"
  }
}
```

Response includes tags from creation (unchanged).

### After (With Tag Editing)

**PUT /api/articles/{slug}**

Request:
```json
{
  "article": {
    "title": "Updated Title",
    "description": "Updated Description",
    "body": "Updated Body",
    "tagList": ["rust", "web", "tutorial"]
  }
}
```

Response includes updated tags.

**Backward Compatibility:** ✅ Yes
- `tagList` is optional
- If not provided, tags remain unchanged
- Existing API clients continue to work

---

## Security Considerations

1. **Authorization:** Already handled (must be article author)
2. **SQL Injection:** Protected (parameterized queries)
3. **XSS:** Tags are text-only, rendered as text (not HTML)
4. **Tag Bombing:** Add max tags validation (10 tags recommended)
5. **Tag Length:** Add max length validation (50 chars recommended)

---

## Rollback Plan

If issues are discovered after deployment:

1. **Quick Fix:** Revert the commit
2. **Database:** No schema changes, so no migration rollback needed
3. **Data:** Tag associations remain in database, no data loss
4. **Frontend:** Already handles case where backend ignores tags

---

## Success Criteria

✅ Feature is complete when:

1. Users can edit article tags via the editor UI
2. Tag updates are reflected immediately in article views
3. All tests pass (existing + new tag update tests)
4. No regression in existing article functionality
5. Documentation is updated
6. Code is reviewed and approved

---

## Timeline Estimate

- **Phase 1 (Model Update):** 5 minutes
- **Phase 2 (Backend Logic):** 30 minutes
- **Phase 3 (Refactoring):** 30 minutes
- **Phase 4 (Testing):** 45 minutes
- **Phase 5 (Documentation):** 15 minutes

**Total Estimated Time:** ~2 hours

---

## Open Questions

1. **Tag Normalization:** Should we normalize tags to lowercase?
   - Recommendation: Yes, for consistency

2. **Tag Validation:** What characters should be allowed?
   - Recommendation: Alphanumeric + hyphens + underscores

3. **Max Tags:** What's a reasonable limit?
   - Recommendation: 10 tags per article

4. **Partial Updates:** Should `tagList: []` mean "no change" or "remove all tags"?
   - Recommendation: `[]` = remove all tags, `null` or absent = no change

---

## Related Issues

- None currently

## Related Pull Requests

- None currently

---

**Next Steps:** Review this plan, confirm approach, then proceed with implementation.
