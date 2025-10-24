# Internal Links Implementation Plan

**Issue:** Blog posts cannot link to each other using a convenient internal link syntax. Authors must manually construct full URLs or relative paths.

**Date:** 2025-10-22
**Status:** Planning Phase

---

## Problem Analysis

### Current State

**Frontend:**
- The article editor (`templates/articles/editor.html`) provides a standard markdown editor
- Authors can create links using standard markdown syntax: `[Link Text](https://full-url.com)`
- No special syntax for internal links between blog posts
- No auto-completion or validation for internal references

**Backend:**
- Markdown processing happens in `src/lib/markdown.rs` (113 lines)
- Uses `pulldown-cmark` crate for markdown-to-HTML conversion
- No awareness of article relationships or internal links
- No backlink tracking between articles

**Database:**
- Articles table contains `slug` (unique identifier) and `title`
- No table for tracking article relationships or backlinks
- No metadata about which articles reference which

### Root Cause

The system lacks:
1. **Special syntax** for internal links (e.g., `[[article-slug]]` or `[[Article Title]]`)
2. **Processing logic** to detect and convert internal link syntax
3. **Database queries** to validate referenced articles exist
4. **Backlink tracking** to show which articles reference each other
5. **Broken link handling** for references to non-existent articles

### Desired Behavior

Authors should be able to write:
```markdown
Check out my previous post on [[rust-web-frameworks]] for more details.

For beginners, see [[Getting Started with Rust]].
```

Which should render as:
```html
Check out my previous post on <a href="/articles/rust-web-frameworks">Rust Web Frameworks</a> for more details.

For beginners, see <a href="/articles/getting-started-with-rust">Getting Started with Rust</a>.
```

---

## Implementation Plan

### Phase 1: Choose Internal Link Syntax

**Decision Required:** Select the syntax for internal links

#### **Option A: Wiki-Style Double Brackets (Recommended)**
**Syntax:** `[[article-slug]]` or `[[Article Title]]`

**Pros:**
- Familiar to users of Obsidian, Notion, Roam Research
- Clear visual distinction from regular markdown links
- Can support both slug and title matching
- Easy to regex parse

**Cons:**
- Not standard markdown
- Requires custom parsing

**Examples:**
```markdown
[[rust-web-frameworks]]                    → Links by slug
[[Getting Started with Rust]]               → Links by title (fuzzy matched to slug)
[[rust-web-frameworks|Custom Link Text]]    → Custom display text
```

#### **Option B: Custom Markdown Extension**
**Syntax:** `[Article Title](//@article-slug)` or `[Text](article:slug)`

**Pros:**
- Closer to standard markdown syntax
- Easier to parse with existing markdown tools

**Cons:**
- Less intuitive
- Doesn't stand out as internal link in editor

**Recommendation:** Use **Option A (Wiki-Style)** for better UX and familiarity

---

### Phase 2: Database Schema Updates

**File:** `src/lib/database.rs`

**New Table:** `article_backlinks`

**Purpose:** Track which articles link to which, enabling:
- "Referenced by" sections on article pages
- Broken link detection
- Article relationship graphs (future)

**Schema:**
```sql
CREATE TABLE IF NOT EXISTS article_backlinks (
    id TEXT PRIMARY KEY,
    source_article_id TEXT NOT NULL,
    target_article_slug TEXT NOT NULL,
    target_article_id TEXT,  -- NULL if link is broken
    link_text TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (source_article_id) REFERENCES articles(id) ON DELETE CASCADE,
    FOREIGN KEY (target_article_id) REFERENCES articles(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_backlinks_source ON article_backlinks(source_article_id);
CREATE INDEX IF NOT EXISTS idx_backlinks_target_slug ON article_backlinks(target_article_slug);
CREATE INDEX IF NOT EXISTS idx_backlinks_target_id ON article_backlinks(target_article_id);
```

**Migration Code:**
Add to `src/lib/database.rs` in the `run_migrations()` function:

```rust
// Create article_backlinks table
conn.execute(
    "CREATE TABLE IF NOT EXISTS article_backlinks (
        id TEXT PRIMARY KEY,
        source_article_id TEXT NOT NULL,
        target_article_slug TEXT NOT NULL,
        target_article_id TEXT,
        link_text TEXT,
        created_at TEXT DEFAULT (datetime('now')),
        FOREIGN KEY (source_article_id) REFERENCES articles(id) ON DELETE CASCADE,
        FOREIGN KEY (target_article_id) REFERENCES articles(id) ON DELETE SET NULL
    )",
    (),
)
.await
.map_err(|e| format!("Failed to create article_backlinks table: {}", e))?;

conn.execute(
    "CREATE INDEX IF NOT EXISTS idx_backlinks_source ON article_backlinks(source_article_id)",
    (),
)
.await
.map_err(|e| format!("Failed to create index: {}", e))?;

conn.execute(
    "CREATE INDEX IF NOT EXISTS idx_backlinks_target_slug ON article_backlinks(target_article_slug)",
    (),
)
.await
.map_err(|e| format!("Failed to create index: {}", e))?;

conn.execute(
    "CREATE INDEX IF NOT EXISTS idx_backlinks_target_id ON article_backlinks(target_article_id)",
    (),
)
.await
.map_err(|e| format!("Failed to create index: {}", e))?;
```

---

### Phase 3: Internal Link Parsing

**File:** `src/lib/markdown.rs`

**Current Implementation:**
```rust
pub fn markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new_ext(markdown, Options::all());
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}
```

**New Implementation Strategy:**

#### Step 1: Extract Internal Links
Create regex to find `[[...]]` patterns before markdown processing:

```rust
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    // Matches [[slug]], [[slug|custom text]], or [[Article Title]]
    static ref INTERNAL_LINK_REGEX: Regex = Regex::new(
        r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]"
    ).unwrap();
}

#[derive(Debug, Clone)]
pub struct InternalLink {
    pub full_match: String,      // "[[rust-basics]]"
    pub target: String,           // "rust-basics" or "Article Title"
    pub custom_text: Option<String>, // Custom display text if provided
    pub position: (usize, usize), // Start and end position in text
}

pub fn extract_internal_links(markdown: &str) -> Vec<InternalLink> {
    INTERNAL_LINK_REGEX
        .captures_iter(markdown)
        .map(|cap| {
            let full_match = cap.get(0).unwrap();
            let target = cap.get(1).unwrap().as_str().trim().to_string();
            let custom_text = cap.get(2).map(|m| m.as_str().trim().to_string());

            InternalLink {
                full_match: full_match.as_str().to_string(),
                target,
                custom_text,
                position: (full_match.start(), full_match.end()),
            }
        })
        .collect()
}
```

#### Step 2: Resolve Internal Links
Query database to resolve slugs/titles to actual articles:

```rust
use crate::database::DatabaseConnection;
use crate::errors::AppError;

#[derive(Debug, Clone)]
pub struct ResolvedInternalLink {
    pub original: InternalLink,
    pub slug: Option<String>,      // Resolved slug if article exists
    pub title: Option<String>,     // Resolved title if article exists
    pub exists: bool,              // Whether target article exists
}

pub async fn resolve_internal_links(
    links: Vec<InternalLink>,
    db: &DatabaseConnection,
) -> Result<Vec<ResolvedInternalLink>, AppError> {
    let mut resolved_links = Vec::new();

    for link in links {
        // Try to find article by slug first
        let mut rows = db.conn
            .query(
                "SELECT slug, title FROM articles WHERE slug = ? OR title = ?",
                libsql::params![link.target.clone(), link.target.clone()],
            )
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        if let Some(row) = rows.next().await.map_err(|e| AppError::InternalServerError(e.to_string()))? {
            let slug: String = row.get(0).map_err(|e| AppError::InternalServerError(e.to_string()))?;
            let title: String = row.get(1).map_err(|e| AppError::InternalServerError(e.to_string()))?;

            resolved_links.push(ResolvedInternalLink {
                original: link,
                slug: Some(slug),
                title: Some(title),
                exists: true,
            });
        } else {
            // Article not found - broken link
            resolved_links.push(ResolvedInternalLink {
                original: link,
                slug: None,
                title: None,
                exists: false,
            });
        }
    }

    Ok(resolved_links)
}
```

#### Step 3: Replace Internal Links with Standard Markdown
Convert internal links to standard markdown links before processing:

```rust
pub fn replace_internal_links(
    markdown: &str,
    resolved_links: &[ResolvedInternalLink],
) -> String {
    let mut result = markdown.to_string();

    // Process in reverse order to maintain position offsets
    for link in resolved_links.iter().rev() {
        let replacement = if link.exists {
            let slug = link.slug.as_ref().unwrap();
            let display_text = link.original.custom_text
                .as_ref()
                .unwrap_or(link.title.as_ref().unwrap());

            // Replace with standard markdown link
            format!("[{}](/articles/{})", display_text, slug)
        } else {
            // Broken link - keep original text but mark it
            format!(
                "<span class=\"broken-link\" title=\"Article not found\">{}</span>",
                link.original.custom_text
                    .as_ref()
                    .unwrap_or(&link.original.target)
            )
        };

        let (start, end) = link.original.position;
        result.replace_range(start..end, &replacement);
    }

    result
}
```

#### Step 4: Update Main Markdown Function
Make the function async and integrate link resolution:

```rust
pub async fn markdown_to_html(
    markdown: &str,
    db: Option<&DatabaseConnection>,
) -> Result<String, AppError> {
    let processed_markdown = if let Some(database) = db {
        // Extract internal links
        let internal_links = extract_internal_links(markdown);

        if !internal_links.is_empty() {
            // Resolve links against database
            let resolved = resolve_internal_links(internal_links, database).await?;

            // Replace internal link syntax with standard markdown
            replace_internal_links(markdown, &resolved)
        } else {
            markdown.to_string()
        }
    } else {
        // No database connection - skip internal link processing
        markdown.to_string()
    };

    // Standard markdown processing
    let parser = Parser::new_ext(&processed_markdown, Options::all());
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    Ok(html_output)
}
```

**Location:** `src/lib/markdown.rs`

**Dependencies to Add to Cargo.toml:**
```toml
regex = "1.10"
lazy_static = "1.4"
```

---

### Phase 4: Update Article Routes

**File:** `src/lib/routes/articles.rs`

**Changes Required:**

#### Update `get_article()` Function
Modify to pass database connection to markdown processor:

```rust
// Around line 220-250 in get_article()
let html_body = markdown_to_html(&article.body, Some(&state.db))
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;
```

#### Update `create_article()` Function
Process internal links and create backlinks when creating articles:

```rust
// After creating the article (around line 140)
let internal_links = extract_internal_links(&article_body);
if !internal_links.is_empty() {
    let resolved = resolve_internal_links(internal_links, &state.db).await?;

    // Store backlinks in database
    for link in resolved {
        save_backlink(
            &state.db,
            &article_id,
            &link.original.target,
            link.slug.as_deref(),
            link.original.custom_text.as_deref(),
        ).await?;
    }
}
```

#### Update `update_article()` Function
Refresh backlinks when article is updated:

```rust
// After updating the article body (around line 615)
if let Some(new_body) = &article_data.body {
    // Delete old backlinks
    state.db.conn.execute(
        "DELETE FROM article_backlinks WHERE source_article_id = ?",
        libsql::params![article_id.clone()],
    )
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Extract and save new backlinks
    let internal_links = extract_internal_links(new_body);
    if !internal_links.is_empty() {
        let resolved = resolve_internal_links(internal_links, &state.db).await?;

        for link in resolved {
            save_backlink(
                &state.db,
                &article_id,
                &link.original.target,
                link.slug.as_deref(),
                link.original.custom_text.as_deref(),
            ).await?;
        }
    }
}
```

**Helper Function:**
```rust
async fn save_backlink(
    db: &DatabaseConnection,
    source_article_id: &str,
    target_slug: &str,
    resolved_target_id: Option<&str>,
    link_text: Option<&str>,
) -> Result<(), AppError> {
    use uuid::Uuid;

    let backlink_id = Uuid::new_v4().to_string();

    db.conn.execute(
        "INSERT INTO article_backlinks (id, source_article_id, target_article_slug, target_article_id, link_text)
         VALUES (?, ?, ?, ?, ?)",
        libsql::params![
            backlink_id,
            source_article_id,
            target_slug,
            resolved_target_id,
            link_text,
        ],
    )
    .await
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(())
}
```

---

### Phase 5: Frontend Display Enhancements

**File:** `templates/articles/article.html`

**Add "Referenced By" Section:**

Add after the article content (around line 150):

```html
{% if backlinks and backlinks | length > 0 %}
<div class="backlinks-section mt-5 pt-4 border-top">
    <h4 class="mb-3">
        <i class="bi bi-arrow-left-circle"></i> Referenced By
    </h4>
    <div class="list-group">
        {% for backlink in backlinks %}
        <a href="/articles/{{ backlink.slug }}" class="list-group-item list-group-item-action">
            <div class="d-flex w-100 justify-content-between">
                <h6 class="mb-1">{{ backlink.title }}</h6>
                <small class="text-muted">{{ backlink.created_at | date(format="%b %d, %Y") }}</small>
            </div>
            {% if backlink.link_text %}
            <small class="text-muted">Linked as: "{{ backlink.link_text }}"</small>
            {% endif %}
        </a>
        {% endfor %}
    </div>
</div>
{% endif %}
```

**File:** `static/css/styles.css`

**Add Styles for Broken Links:**

```css
/* Internal link styles */
.broken-link {
    color: #dc3545;
    text-decoration: line-through;
    cursor: help;
    border-bottom: 1px dashed #dc3545;
}

.broken-link:hover {
    background-color: #fff3cd;
}

/* Backlinks section */
.backlinks-section {
    background-color: #f8f9fa;
    padding: 1.5rem;
    border-radius: 0.5rem;
}

.backlinks-section .list-group-item {
    border: 1px solid #dee2e6;
    margin-bottom: 0.5rem;
    border-radius: 0.25rem;
}

.backlinks-section .list-group-item:hover {
    background-color: #e9ecef;
}
```

---

### Phase 6: API Endpoint for Backlinks

**File:** `src/lib/routes/articles.rs`

**New Function:**

```rust
pub async fn get_article_backlinks(
    Path(slug): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Get article ID from slug
    let mut article_rows = state.db.conn
        .query(
            "SELECT id FROM articles WHERE slug = ?",
            libsql::params![slug],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let article_row = article_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Article not found".to_string()))?;

    let article_id: String = article_row
        .get(0)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Get backlinks
    let mut backlink_rows = state.db.conn
        .query(
            "SELECT b.link_text, a.slug, a.title, a.created_at
             FROM article_backlinks b
             INNER JOIN articles a ON b.source_article_id = a.id
             WHERE b.target_article_id = ?
             ORDER BY a.created_at DESC",
            libsql::params![article_id],
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let mut backlinks = Vec::new();
    while let Some(row) = backlink_rows
        .next()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        let link_text: Option<String> = row.get(0).ok();
        let slug: String = row.get(1).map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let title: String = row.get(2).map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let created_at: String = row.get(3).map_err(|e| AppError::InternalServerError(e.to_string()))?;

        backlinks.push(serde_json::json!({
            "linkText": link_text,
            "slug": slug,
            "title": title,
            "createdAt": created_at,
        }));
    }

    Ok(Json(serde_json::json!({
        "backlinks": backlinks,
        "count": backlinks.len(),
    })))
}
```

**Add Route:**
In `src/lib/startup.rs`, add:

```rust
.route("/api/articles/:slug/backlinks", get(get_article_backlinks))
```

---

### Phase 7: Editor Auto-Complete (Optional Enhancement)

**File:** `static/js/editor.js`

**Add Auto-Complete for Internal Links:**

```javascript
// Auto-complete for internal links
async function setupInternalLinkAutocomplete() {
    const bodyTextarea = document.getElementById('article-body');
    if (!bodyTextarea) return;

    let autocompleteTimeout;
    let autocompleteList = null;

    bodyTextarea.addEventListener('input', async (e) => {
        clearTimeout(autocompleteTimeout);

        const cursorPos = e.target.selectionStart;
        const textBeforeCursor = e.target.value.substring(0, cursorPos);

        // Check if user is typing [[
        const match = textBeforeCursor.match(/\[\[([^\]]*)$/);

        if (match) {
            const query = match[1];

            // Debounce API call
            autocompleteTimeout = setTimeout(async () => {
                if (query.length >= 2) {
                    const suggestions = await fetchArticleSuggestions(query);
                    showAutocompleteSuggestions(suggestions, e.target, cursorPos);
                }
            }, 300);
        } else {
            hideAutocompleteSuggestions();
        }
    });
}

async function fetchArticleSuggestions(query) {
    try {
        const response = await fetch(`/api/articles?limit=5&search=${encodeURIComponent(query)}`);
        const data = await response.json();
        return data.articles || [];
    } catch (error) {
        console.error('Failed to fetch suggestions:', error);
        return [];
    }
}

function showAutocompleteSuggestions(suggestions, textarea, cursorPos) {
    hideAutocompleteSuggestions();

    if (suggestions.length === 0) return;

    const list = document.createElement('div');
    list.className = 'internal-link-autocomplete';
    list.style.position = 'absolute';

    // Position below cursor (simplified - would need proper positioning)
    const rect = textarea.getBoundingClientRect();
    list.style.top = `${rect.top + 20}px`;
    list.style.left = `${rect.left}px`;

    suggestions.forEach(article => {
        const item = document.createElement('div');
        item.className = 'autocomplete-item';
        item.textContent = article.title;
        item.dataset.slug = article.slug;

        item.addEventListener('click', () => {
            insertInternalLink(textarea, article.slug);
            hideAutocompleteSuggestions();
        });

        list.appendChild(item);
    });

    document.body.appendChild(list);
    autocompleteList = list;
}

function hideAutocompleteSuggestions() {
    if (autocompleteList) {
        autocompleteList.remove();
        autocompleteList = null;
    }
}

function insertInternalLink(textarea, slug) {
    const cursorPos = textarea.selectionStart;
    const textBeforeCursor = textarea.value.substring(0, cursorPos);
    const textAfterCursor = textarea.value.substring(cursorPos);

    // Find the [[ that started the autocomplete
    const match = textBeforeCursor.match(/\[\[([^\]]*)$/);
    if (match) {
        const startPos = cursorPos - match[0].length;
        const newText = `[[${slug}]]`;

        textarea.value =
            textarea.value.substring(0, startPos) +
            newText +
            textAfterCursor;

        // Position cursor after the inserted link
        textarea.selectionStart = textarea.selectionEnd = startPos + newText.length;
        textarea.focus();
    }
}

// Initialize on page load
document.addEventListener('DOMContentLoaded', () => {
    setupInternalLinkAutocomplete();
});
```

**CSS for Auto-Complete:**

```css
/* Internal link autocomplete */
.internal-link-autocomplete {
    position: absolute;
    background: white;
    border: 1px solid #ddd;
    border-radius: 4px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.15);
    max-height: 200px;
    overflow-y: auto;
    z-index: 1000;
    min-width: 300px;
}

.autocomplete-item {
    padding: 8px 12px;
    cursor: pointer;
    border-bottom: 1px solid #f0f0f0;
}

.autocomplete-item:last-child {
    border-bottom: none;
}

.autocomplete-item:hover {
    background-color: #f5f5f5;
}
```

---

### Phase 8: Testing Strategy

**Test File:** `tests/api/internal_links.rs` (new file)

**Test Coverage:**

#### 1. **Internal Link Extraction Tests**
```rust
#[test]
fn test_extract_internal_links() {
    let markdown = "Check out [[rust-basics]] and [[advanced-patterns|Advanced Stuff]].";
    let links = extract_internal_links(markdown);

    assert_eq!(links.len(), 2);
    assert_eq!(links[0].target, "rust-basics");
    assert_eq!(links[0].custom_text, None);
    assert_eq!(links[1].target, "advanced-patterns");
    assert_eq!(links[1].custom_text, Some("Advanced Stuff".to_string()));
}

#[test]
fn test_extract_no_internal_links() {
    let markdown = "Regular markdown with [external link](https://example.com).";
    let links = extract_internal_links(markdown);

    assert_eq!(links.len(), 0);
}
```

#### 2. **Link Resolution Tests**
```rust
#[tokio::test]
async fn test_resolve_internal_links_existing_article() {
    let app = spawn_app().await;
    let user = app.create_test_user("author", "author@example.com", "password").await;

    // Create target article
    let target_article = app.create_test_article(
        &user.token,
        "Rust Basics",
        "rust-basics",
        "Introduction to Rust",
        "Content here"
    ).await;

    // Test link resolution
    let links = vec![InternalLink {
        full_match: "[[rust-basics]]".to_string(),
        target: "rust-basics".to_string(),
        custom_text: None,
        position: (0, 16),
    }];

    let resolved = resolve_internal_links(links, &app.db).await.unwrap();

    assert_eq!(resolved.len(), 1);
    assert!(resolved[0].exists);
    assert_eq!(resolved[0].slug.as_ref().unwrap(), "rust-basics");
    assert_eq!(resolved[0].title.as_ref().unwrap(), "Rust Basics");
}

#[tokio::test]
async fn test_resolve_internal_links_broken_link() {
    let app = spawn_app().await;

    let links = vec![InternalLink {
        full_match: "[[non-existent]]".to_string(),
        target: "non-existent".to_string(),
        custom_text: None,
        position: (0, 17),
    }];

    let resolved = resolve_internal_links(links, &app.db).await.unwrap();

    assert_eq!(resolved.len(), 1);
    assert!(!resolved[0].exists);
    assert_eq!(resolved[0].slug, None);
}
```

#### 3. **Markdown Processing Tests**
```rust
#[tokio::test]
async fn test_markdown_with_internal_links() {
    let app = spawn_app().await;
    let user = app.create_test_user("author", "author@example.com", "password").await;

    // Create target article
    app.create_test_article(
        &user.token,
        "Rust Basics",
        "rust-basics",
        "Introduction",
        "Content"
    ).await;

    // Process markdown with internal link
    let markdown = "See [[rust-basics]] for details.";
    let html = markdown_to_html(markdown, Some(&app.db)).await.unwrap();

    assert!(html.contains(r#"<a href="/articles/rust-basics">"#));
    assert!(html.contains("Rust Basics"));
}

#[tokio::test]
async fn test_markdown_with_broken_internal_link() {
    let app = spawn_app().await;

    let markdown = "See [[non-existent]] for details.";
    let html = markdown_to_html(markdown, Some(&app.db)).await.unwrap();

    assert!(html.contains("broken-link"));
    assert!(html.contains("non-existent"));
}
```

#### 4. **Backlink Creation Tests**
```rust
#[tokio::test]
async fn test_create_article_with_internal_links_creates_backlinks() {
    let app = spawn_app().await;
    let user = app.create_test_user("author", "author@example.com", "password").await;

    // Create target article
    let target = app.create_test_article(
        &user.token,
        "Target Article",
        "target-article",
        "Description",
        "Content"
    ).await;

    // Create source article with internal link
    let source_body = "Check out [[target-article]] for more info.";
    let source = app.create_test_article(
        &user.token,
        "Source Article",
        "source-article",
        "Description",
        source_body
    ).await;

    // Verify backlink was created
    let response = app.client
        .get(&format!("{}/api/articles/{}/backlinks", app.address, "target-article"))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    let backlinks = body["backlinks"].as_array().unwrap();

    assert_eq!(backlinks.len(), 1);
    assert_eq!(backlinks[0]["slug"], "source-article");
}
```

#### 5. **Backlink Update Tests**
```rust
#[tokio::test]
async fn test_update_article_refreshes_backlinks() {
    let app = spawn_app().await;
    let user = app.create_test_user("author", "author@example.com", "password").await;

    // Create two target articles
    let target1 = app.create_test_article(&user.token, "Target 1", "target-1", "Desc", "Content").await;
    let target2 = app.create_test_article(&user.token, "Target 2", "target-2", "Desc", "Content").await;

    // Create source with link to target1
    let source = app.create_test_article(
        &user.token,
        "Source",
        "source",
        "Desc",
        "See [[target-1]]"
    ).await;

    // Update source to link to target2 instead
    let update_payload = json!({
        "article": {
            "body": "See [[target-2]] instead"
        }
    });

    app.client
        .put(&format!("{}/api/articles/source", app.address))
        .header("Authorization", format!("Bearer {}", user.token))
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to execute request");

    // Verify target1 has no backlinks
    let response1 = app.client
        .get(&format!("{}/api/articles/target-1/backlinks", app.address))
        .send()
        .await
        .unwrap();
    let body1: serde_json::Value = response1.json().await.unwrap();
    assert_eq!(body1["count"], 0);

    // Verify target2 has one backlink
    let response2 = app.client
        .get(&format!("{}/api/articles/target-2/backlinks", app.address))
        .send()
        .await
        .unwrap();
    let body2: serde_json::Value = response2.json().await.unwrap();
    assert_eq!(body2["count"], 1);
}
```

#### 6. **Edge Case Tests**
```rust
#[tokio::test]
async fn test_internal_link_with_custom_text() {
    let app = spawn_app().await;
    let user = app.create_test_user("author", "author@example.com", "password").await;

    app.create_test_article(&user.token, "Rust Guide", "rust-guide", "Desc", "Content").await;

    let markdown = "Read the [[rust-guide|comprehensive guide]] here.";
    let html = markdown_to_html(markdown, Some(&app.db)).await.unwrap();

    assert!(html.contains("comprehensive guide"));
    assert!(html.contains(r#"href="/articles/rust-guide""#));
}

#[tokio::test]
async fn test_multiple_links_to_same_article() {
    let app = spawn_app().await;
    let user = app.create_test_user("author", "author@example.com", "password").await;

    app.create_test_article(&user.token, "Target", "target", "Desc", "Content").await;

    let markdown = "See [[target]] and also [[target|this article]].";
    let html = markdown_to_html(markdown, Some(&app.db)).await.unwrap();

    // Should have two links in the output
    assert_eq!(html.matches(r#"href="/articles/target""#).count(), 2);
}

#[test]
fn test_internal_link_regex_edge_cases() {
    // Empty brackets
    let links = extract_internal_links("[[]]");
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].target, "");

    // Nested brackets
    let links = extract_internal_links("[[outer[[inner]]]]");
    // Should handle gracefully (exact behavior TBD)

    // Special characters
    let links = extract_internal_links("[[rust-web_framework.v2]]");
    assert_eq!(links.len(), 1);
}
```

---

## Implementation Steps (Ordered)

1. ✅ **Analysis Complete** - Understand requirements and plan approach
2. ⬜ **Choose Syntax** - Finalize internal link syntax (wiki-style recommended)
3. ⬜ **Database Migration** - Create article_backlinks table
4. ⬜ **Add Dependencies** - Add regex and lazy_static to Cargo.toml
5. ⬜ **Implement Link Extraction** - Add regex parsing in markdown.rs
6. ⬜ **Implement Link Resolution** - Add database queries to resolve links
7. ⬜ **Update Markdown Processing** - Make markdown_to_html async and link-aware
8. ⬜ **Update Article Routes** - Modify create/update/get functions
9. ⬜ **Add Backlinks API** - Create GET /api/articles/:slug/backlinks endpoint
10. ⬜ **Update Frontend Templates** - Add "Referenced By" section
11. ⬜ **Add CSS Styles** - Style broken links and backlinks section
12. ⬜ **Write Unit Tests** - Test link extraction and resolution
13. ⬜ **Write Integration Tests** - Test end-to-end functionality
14. ⬜ **Run Tests** - Verify all tests pass
15. ⬜ **Manual Testing** - Test in dev environment
16. ⬜ **Optional: Editor Auto-Complete** - Implement [[... suggestions
17. ⬜ **Update Documentation** - README, CHANGELOG, CODEBASE_INDEX
18. ⬜ **Code Review** - Review all changes
19. ⬜ **Commit & Push** - Commit with descriptive message
20. ⬜ **Deploy** - Deploy to production via Shuttle

---

## Edge Cases to Consider

### 1. **Circular Links**
- **Scenario:** Article A links to Article B, which links back to Article A
- **Behavior:** Allowed - not a problem for rendering
- **Implementation:** No special handling needed

### 2. **Self-Links**
- **Scenario:** Article links to itself `[[same-article]]`
- **Behavior:** Allowed but potentially confusing
- **Implementation:** Detect and optionally warn or style differently

### 3. **Case Sensitivity**
- **Scenario:** `[[Rust-Basics]]` vs `[[rust-basics]]`
- **Decision:** Slugs are case-sensitive in database
- **Recommendation:** Use case-insensitive matching with LOWER() in SQL

### 4. **Link to Draft/Unpublished Article**
- **Scenario:** Link to article that exists but isn't published
- **Behavior:** Treat as broken link for non-authors
- **Implementation:** Check article status in resolution query

### 5. **Link with Special Characters**
- **Scenario:** `[[article-with-[brackets]]]`
- **Behavior:** Regex needs to handle nested brackets
- **Implementation:** Use non-greedy matching or escape sequences

### 6. **Very Long Link Text**
- **Scenario:** `[[very-long-slug-name|Custom text that goes on for many characters]]`
- **Behavior:** Should truncate display in backlinks section
- **Implementation:** CSS text-overflow or string truncation

### 7. **Links in Code Blocks**
- **Scenario:**
  ```markdown
  Example syntax: [[article-name]]
  ```
- **Behavior:** Should NOT be processed inside code blocks
- **Implementation:** Process internal links before markdown parsing, or parse markdown first and skip code sections

### 8. **Deleted Target Article**
- **Scenario:** Article B is deleted after Article A links to it
- **Behavior:** Link becomes broken, backlink removed
- **Implementation:** Foreign key ON DELETE SET NULL handles this

### 9. **Slug Changes**
- **Scenario:** Target article's slug is changed
- **Behavior:** Links break unless we track slug history
- **Implementation:** Document that slug changes break links (or implement slug redirect table)

### 10. **Performance with Many Links**
- **Scenario:** Article with 50+ internal links
- **Behavior:** Could slow down rendering
- **Implementation:** Batch database queries or cache article slug/title map

---

## Validation Enhancements

### Markdown Validation
Add validation to warn about potential issues:

```rust
pub struct InternalLinkValidation {
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

pub fn validate_internal_links(
    markdown: &str,
    resolved_links: &[ResolvedInternalLink],
) -> InternalLinkValidation {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    for link in resolved_links {
        // Broken links
        if !link.exists {
            errors.push(format!(
                "Broken link to '{}' - article not found",
                link.original.target
            ));
        }

        // Empty target
        if link.original.target.trim().is_empty() {
            warnings.push("Empty internal link target found".to_string());
        }

        // Very long custom text
        if let Some(text) = &link.original.custom_text {
            if text.len() > 100 {
                warnings.push(format!(
                    "Very long custom link text ({}  chars): '{}'",
                    text.len(),
                    &text[..50]
                ));
            }
        }
    }

    InternalLinkValidation { warnings, errors }
}
```

### API Response with Validation
Include validation results in article create/update responses:

```rust
// In create_article or update_article response
if !validation.warnings.is_empty() || !validation.errors.is_empty() {
    response["linkValidation"] = json!({
        "warnings": validation.warnings,
        "errors": validation.errors,
    });
}
```

---

## Database Considerations

### Query Optimization

#### Problem: N+1 Query Issue
Current implementation queries database once per internal link. For articles with many links, this is inefficient.

**Solution: Batch Query**
```rust
pub async fn resolve_internal_links_batch(
    links: Vec<InternalLink>,
    db: &DatabaseConnection,
) -> Result<Vec<ResolvedInternalLink>, AppError> {
    if links.is_empty() {
        return Ok(Vec::new());
    }

    // Collect all unique targets
    let targets: Vec<String> = links.iter()
        .map(|l| l.target.clone())
        .collect();

    // Build IN clause
    let placeholders = targets.iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");

    let query = format!(
        "SELECT slug, title FROM articles WHERE slug IN ({}) OR title IN ({})",
        placeholders, placeholders
    );

    // Build params - both slug and title matches
    let mut params = Vec::new();
    for target in &targets {
        params.push(libsql::Value::from(target.clone()));
    }
    for target in &targets {
        params.push(libsql::Value::from(target.clone()));
    }

    // Execute single query
    let mut rows = db.conn
        .query(&query, params)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Build lookup map
    let mut article_map = std::collections::HashMap::new();
    while let Some(row) = rows.next().await.map_err(|e| AppError::InternalServerError(e.to_string()))? {
        let slug: String = row.get(0).map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let title: String = row.get(1).map_err(|e| AppError::InternalServerError(e.to_string()))?;
        article_map.insert(slug.clone(), (slug, title));
    }

    // Resolve each link using the map
    let resolved: Vec<ResolvedInternalLink> = links.into_iter()
        .map(|link| {
            if let Some((slug, title)) = article_map.get(&link.target) {
                ResolvedInternalLink {
                    original: link,
                    slug: Some(slug.clone()),
                    title: Some(title.clone()),
                    exists: true,
                }
            } else {
                ResolvedInternalLink {
                    original: link,
                    slug: None,
                    title: None,
                    exists: false,
                }
            }
        })
        .collect();

    Ok(resolved)
}
```

### Indexing Strategy

Already included in schema:
- `idx_backlinks_source` - Fast lookup of outgoing links from an article
- `idx_backlinks_target_slug` - Fast lookup by slug (for broken link detection)
- `idx_backlinks_target_id` - Fast lookup of incoming links (backlinks)

### Backlink Cleanup

**Orphaned Backlinks:** When articles are deleted, backlinks are handled by CASCADE.

**Broken Links:** Backlinks where target article no longer exists (target_article_id = NULL).

**Cleanup Query (Admin Tool):**
```sql
-- Find all broken backlinks
SELECT b.id, b.source_article_id, b.target_article_slug
FROM article_backlinks b
WHERE b.target_article_id IS NULL;

-- Delete all broken backlinks (optional)
DELETE FROM article_backlinks WHERE target_article_id IS NULL;
```

---

## Performance Impact

### Expected Impact
- **Article Creation:** +50-100ms (for link extraction and backlink creation)
- **Article Updates:** +50-100ms (for backlink refresh)
- **Article Rendering:** +10-30ms (for link resolution)
- **Backlinks Query:** <10ms (indexed lookup)

### Optimization Strategies

#### 1. **Caching**
Cache resolved article slug/title map in memory:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ArticleCache {
    slug_to_title: Arc<RwLock<HashMap<String, String>>>,
}

impl ArticleCache {
    pub async fn get_or_fetch(&self, slug: &str, db: &DatabaseConnection) -> Option<String> {
        // Check cache first
        {
            let cache = self.slug_to_title.read().await;
            if let Some(title) = cache.get(slug) {
                return Some(title.clone());
            }
        }

        // Fetch from database
        // ... query logic ...

        // Update cache
        {
            let mut cache = self.slug_to_title.write().await;
            cache.insert(slug.to_string(), title.clone());
        }

        Some(title)
    }
}
```

#### 2. **Lazy Backlink Loading**
Don't load backlinks on every article view - load them via separate API call or only when section is expanded:

```javascript
// In article-page.js
async function loadBacklinks(slug) {
    const response = await fetch(`/api/articles/${slug}/backlinks`);
    const data = await response.json();

    if (data.count > 0) {
        renderBacklinks(data.backlinks);
    }
}

// Load after page render
document.addEventListener('DOMContentLoaded', () => {
    const slug = getArticleSlug();
    loadBacklinks(slug);
});
```

#### 3. **Background Processing**
For very large articles, process backlinks asynchronously:

```rust
tokio::spawn(async move {
    // Process backlinks in background
    process_article_backlinks(article_id, body, db).await;
});
```

---

## API Contract

### New Endpoints

#### **GET /api/articles/:slug/backlinks**

**Description:** Get all articles that link to this article

**Response:**
```json
{
  "backlinks": [
    {
      "slug": "source-article",
      "title": "Source Article Title",
      "linkText": "custom link text",
      "createdAt": "2025-10-22T10:00:00Z"
    }
  ],
  "count": 1
}
```

### Modified Endpoints

#### **POST /api/articles** (Create Article)

**Changes:**
- Now processes internal links in body
- Creates backlink entries
- Response includes link validation warnings/errors (if any)

**Response Addition:**
```json
{
  "article": { /* ... */ },
  "linkValidation": {
    "warnings": ["Warning: Very long custom link text..."],
    "errors": ["Broken link to 'non-existent' - article not found"]
  }
}
```

#### **PUT /api/articles/:slug** (Update Article)

**Changes:**
- Refreshes backlinks when body is updated
- Deletes old backlinks, creates new ones
- Response includes link validation

**Backward Compatibility:** ✅ Yes
- No breaking changes to request/response format
- Existing clients continue to work
- Link processing is transparent

---

## Security Considerations

### 1. **SQL Injection**
- **Risk:** User-controlled link targets in SQL queries
- **Mitigation:** All queries use parameterized statements ✅

### 2. **XSS Attacks**
- **Risk:** Malicious HTML in link text
- **Mitigation:**
  - Markdown processor escapes HTML
  - Link text stored as plain text
  - Rendered through safe templating ✅

### 3. **Link Enumeration**
- **Risk:** Users can discover unpublished article slugs via broken links
- **Mitigation:** Only show backlinks for published articles
- **Implementation:**
  ```sql
  -- Add WHERE clause to backlinks query
  WHERE b.target_article_id = ?
    AND a.published = 1  -- Only published articles
  ```

### 4. **Denial of Service**
- **Risk:** Article with thousands of internal links could overload system
- **Mitigation:** Add validation limit (max 100 internal links per article)
- **Implementation:**
  ```rust
  if internal_links.len() > 100 {
      return Err(AppError::BadRequest(
          "Article contains too many internal links (max 100)".to_string()
      ));
  }
  ```

### 5. **Resource Exhaustion**
- **Risk:** Recursive link checking could consume excessive resources
- **Mitigation:**
  - Process links sequentially (or with bounded parallelism)
  - Set timeout on database queries
  - Use batch queries to minimize round trips

### 6. **Authorization**
- **Risk:** Unauthorized backlink manipulation
- **Mitigation:**
  - Backlinks created/updated only through article endpoints
  - No direct backlink API for creation/deletion
  - Article authorization applies ✅

---

## Rollback Plan

### If Issues Discovered Post-Deployment

#### **Quick Fix: Disable Link Processing**
Add feature flag to disable internal link processing:

```rust
// In config.rs
pub struct Config {
    // ... existing fields ...
    pub enable_internal_links: bool,
}

// In markdown.rs
pub async fn markdown_to_html(
    markdown: &str,
    db: Option<&DatabaseConnection>,
    config: &Config,
) -> Result<String, AppError> {
    let processed_markdown = if config.enable_internal_links {
        // Process internal links
        // ...
    } else {
        markdown.to_string()
    };
    // ...
}
```

Set `ENABLE_INTERNAL_LINKS=false` in Secrets.toml to disable.

#### **Database Rollback**
If backlinks table causes issues:

```sql
-- Drop backlinks table
DROP TABLE IF EXISTS article_backlinks;
DROP INDEX IF EXISTS idx_backlinks_source;
DROP INDEX IF EXISTS idx_backlinks_target_slug;
DROP INDEX IF EXISTS idx_backlinks_target_id;
```

**Note:** No data loss for articles - backlinks are metadata only.

#### **Code Revert**
1. Git revert the feature commit
2. Redeploy via `shuttle deploy`
3. Internal link syntax remains in articles but renders as plain text

---

## Success Criteria

✅ Feature is complete when:

1. Authors can use `[[article-slug]]` syntax in markdown
2. Internal links render as proper HTML `<a>` tags
3. Broken links are visually indicated
4. "Referenced By" section shows on article pages
5. Backlinks API returns correct data
6. All tests pass (unit + integration)
7. No regression in existing markdown functionality
8. Performance impact is minimal (<100ms overhead)
9. Documentation is updated
10. Code is reviewed and approved
11. Feature can be disabled via config flag

---

## Timeline Estimate

- **Phase 1 (Syntax Decision):** 15 minutes
- **Phase 2 (Database Migration):** 30 minutes
- **Phase 3 (Link Parsing & Resolution):** 2-3 hours
- **Phase 4 (Article Routes Update):** 1-2 hours
- **Phase 5 (Frontend Display):** 1 hour
- **Phase 6 (Backlinks API):** 1 hour
- **Phase 7 (Auto-Complete - Optional):** 2-3 hours
- **Phase 8 (Testing):** 3-4 hours
- **Documentation & Review:** 1 hour

**Total Estimated Time (Core Features):** ~10-12 hours
**With Optional Auto-Complete:** ~13-15 hours

**Recommended Approach:** Implement in phases
- **Phase 1-6:** Core functionality (~8-10 hours)
- **Phase 7:** Optional enhancement (add later if needed)

---

## Open Questions

### 1. **Link Syntax Preference**
- **Question:** Wiki-style `[[...]]` or custom markdown extension?
- **Recommendation:** Wiki-style for familiarity and ease of use
- **Decision:** TBD - User preference

### 2. **Link Resolution Strategy**
- **Question:** Match by slug only, title only, or both?
- **Recommendation:** Try slug first (exact match), then title (case-insensitive)
- **Rationale:** Slugs are unique and stable; titles can change

### 3. **Broken Link Display**
- **Question:** How to display broken links?
- **Options:**
  - Strike-through with tooltip (current plan)
  - Red text with warning icon
  - Remove link entirely and show as plain text
- **Recommendation:** Strike-through with tooltip (most informative)

### 4. **Backlink Visibility**
- **Question:** Should backlinks be shown by default or on-demand?
- **Options:**
  - Always visible at bottom of article
  - Collapsed accordion section
  - Separate tab or modal
  - Loaded via AJAX after page load
- **Recommendation:** Always visible if count > 0, AJAX loaded for performance

### 5. **Max Internal Links**
- **Question:** What's a reasonable limit per article?
- **Recommendation:** 100 internal links max
- **Rationale:** Prevents abuse while allowing comprehensive cross-referencing

### 6. **Slug Changes**
- **Question:** Should we handle slug changes gracefully?
- **Options:**
  - Document that slug changes break links (simplest)
  - Implement slug redirect table (complex)
  - Scan all articles and update links (very expensive)
- **Recommendation:** Document the limitation for now; consider redirects later

### 7. **Draft Article Links**
- **Question:** Can published articles link to draft articles?
- **Recommendation:**
  - Allow the link in markdown
  - Resolve link if viewer is author
  - Show as broken link to other users

---

## Future Enhancements

### Phase 2 Features (Post-MVP)

1. **Link Graph Visualization**
   - Visual graph of article relationships
   - Interactive navigation
   - Cluster detection

2. **Orphaned Article Detection**
   - Find articles with no incoming links
   - Suggest linking opportunities

3. **Link Suggestions**
   - Auto-suggest internal links based on content
   - NLP-based similarity matching

4. **Alias System**
   - `[[alias|article-slug]]` syntax
   - Allow multiple names for same article

5. **Slug History & Redirects**
   - Track slug changes
   - Auto-update internal links
   - Create redirects for old slugs

6. **Backlink Context**
   - Show surrounding text where link appears
   - Preview on hover

7. **Bi-directional Link Preview**
   - Show linked article preview in sidebar
   - Hover to preview content

8. **Link Analytics**
   - Track most-linked articles
   - Identify broken links across all articles
   - Link health dashboard

---

## Related Issues

- None currently

## Related Pull Requests

- None currently

---

## Dependencies

### New Cargo Dependencies
```toml
[dependencies]
regex = "1.10"
lazy_static = "1.4"
```

### Existing Dependencies Used
- `libsql` - Database queries
- `pulldown-cmark` - Markdown processing
- `uuid` - Backlink ID generation
- `serde_json` - API responses

---

## Migration Path

### For Existing Articles

**Scenario:** What about articles written before this feature?

**Answer:** No migration needed
- Old articles work as-is
- Authors can add internal links when editing
- No automatic link detection/conversion

### For Future Content Migration

If moving from another platform with internal links:

1. **Identify link syntax** in source content
2. **Write conversion script** to transform to `[[...]]` syntax
3. **Import articles** with new syntax
4. **Validate links** using the validation API
5. **Fix broken links** manually or via script

---

**Next Steps:** Review this plan, confirm syntax choice and priorities, then proceed with phased implementation.
