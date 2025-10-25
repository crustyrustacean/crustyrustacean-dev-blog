# JavaScript Unused Code Review

## Summary

After reviewing the JavaScript codebase, I've identified several categories of unused or redundant code that can be safely removed or refactored.

## 🔴 Completely Unused Files (Should be Deleted)

### 1. **media-picker.js** - Empty file, never loaded
- **Path**: `static/js/media-picker.js`
- **Status**: File is completely empty (1 byte)
- **Referenced**: No template references
- **Action**: ✅ **DELETE**

### 2. **media-uploader.js** - Empty file, never loaded
- **Path**: `static/js/media-uploader.js`
- **Status**: File is completely empty (1 byte)
- **Referenced**: No template references
- **Action**: ✅ **DELETE**

## 🟡 Potentially Unused Templates & Associated Scripts

### 3. **articles/article-old.html** + Associated JS
- **Status**: Not referenced in any Rust route handler
- **Currently Used**: `articles/article.html` (line 1024 in articles.rs)
- **Impact**: If removed, would eliminate associated article-page.js dependencies
- **Action**: ⚠️ **VERIFY** then delete if confirmed unused

### 4. **articles/simple.html** + article-init.js
- **Status**: Not referenced in any Rust route handler
- **Script Loaded**: `article-init.js` (only loaded by this template)
- **Impact**: article-init.js can be removed if this template is unused
- **Action**: ⚠️ **VERIFY** then delete if confirmed unused

### 5. **articles/modern.html**
- **Status**: Not referenced in any Rust route handler
- **Scripts**: Uses utils.js, article.js, article-page.js, comments.js
- **Action**: ⚠️ **VERIFY** then delete if confirmed unused

## 🟠 Code Duplication Issues

### 6. **Utility Function Duplication**

Multiple files redefine utility functions instead of using `utils.js`:

#### `getAuthToken()` duplicated in:
- ✅ `utils.js:6` (original)
- ❌ `api-keys.js:217` (duplicate definition)
- ❌ `article-page.js:4` (duplicate definition)
- ❌ `editor.js:85` (variant: `getAuthTokenFromCookies`)

**Impact**: 3 duplicate implementations of the same function

#### `formatDate()` duplicated in:
- ✅ `utils.js:33` (original)
- ❌ `api-keys.js:232` (duplicate definition)
- ❌ `article-page.js:20` (duplicate definition)

**Impact**: 2 duplicate implementations

#### `showError()` duplicated in:
- ✅ `utils.js:66` (original)
- ❌ `api-keys.js:279` (duplicate definition)
- ❌ `auth.js:171` (duplicate definition)
- ❌ `editor.js:97` (duplicate definition)

**Impact**: 3 duplicate implementations

#### `showSuccess()` duplicated in:
- ✅ `utils.js:74` (original)
- ❌ `api-keys.js:273` (duplicate definition)
- ❌ `auth.js:200` (duplicate definition)
- ❌ `editor.js:101` (duplicate definition)

**Impact**: 3 duplicate implementations

### Recommendation for Duplication
**Action**: ⚠️ **REFACTOR** - Ensure utils.js is loaded first and remove all duplicate definitions

## 🟢 Unused Utility Functions

### 7. **getCookie() in utils.js**
- **Defined**: `utils.js:24`
- **Usage**: Never called anywhere in the codebase
- **Note**: `getAuthToken()` has its own cookie parsing logic inline
- **Action**: ⚠️ **REVIEW** - Consider removing or consolidating with getAuthToken

## 📊 Usage Statistics

### JavaScript Files by Usage Status

| File | Status | Templates Using It |
|------|--------|-------------------|
| base.js | ✅ Used | All pages (base.html) |
| utils.js | ✅ Used | index.html, list.html, simple.html, modern.html |
| admin.js | ✅ Used | admin/dashboard.html |
| api-keys.js | ✅ Used | admin/api-keys.html |
| article.js | ✅ Used | articles/simple.html, articles/modern.html |
| article-list.js | ✅ Used | articles/list.html |
| article-page.js | ✅ Used | articles/article.html, articles/modern.html |
| **article-init.js** | ⚠️ Used | articles/simple.html (unused template?) |
| auth.js | ✅ Used | auth/login.html, auth/register.html |
| authors.js | ✅ Used | profile/authors.html |
| comments.js | ✅ Used | articles/article.html, articles/simple.html, modern.html |
| editor.js | ✅ Used | articles/editor.html |
| error404.js | ✅ Used | errors/404.html |
| favorites.js | ✅ Used | profile/favorites.html |
| feed.js | ✅ Used | articles/feed.html |
| homepage.js | ✅ Used | index.html |
| media-library.js | ✅ Used | media/library.html |
| **media-picker.js** | ❌ Never used | None |
| **media-uploader.js** | ❌ Never used | None |
| profile.js | ✅ Used | profile/profile.html |
| tags-admin.js | ✅ Used | admin/tags.html |

## 🎯 Recommended Actions

### Immediate Deletions (Safe)
```bash
# Delete empty, unreferenced files
rm static/js/media-picker.js
rm static/js/media-uploader.js
```

### Verification Required
1. Check if `articles/article-old.html` is used anywhere
2. Check if `articles/simple.html` is used anywhere  
3. Check if `articles/modern.html` is used anywhere
4. If templates are unused, delete them along with `article-init.js`

### Refactoring Needed
1. **Consolidate utility functions**:
   - Ensure `utils.js` is loaded on all pages that need utilities
   - Remove duplicate definitions from:
     - `api-keys.js` (lines 217, 232, 273, 279)
     - `article-page.js` (lines 4, 20)
     - `auth.js` (lines 171, 200)
     - `editor.js` (lines 85, 97, 101)

2. **Review getCookie() function**:
   - Either use it in `getAuthToken()` or remove it

## 💾 Estimated Cleanup Impact

- **Files to delete**: 2-5 files (depending on template verification)
- **Lines of duplicate code**: ~100+ lines across multiple files
- **Maintenance benefit**: Reduced confusion, single source of truth for utilities
- **Performance**: Minimal (files are small, but reduces HTTP requests)

## 🔍 Notes

### Template Usage Verification
The following templates were NOT found in route handlers:
- `templates/articles/article-old.html`
- `templates/articles/simple.html`
- `templates/articles/modern.html`
- `templates/admin/debug.html`
- `templates/admin/simple.html`

Currently used template: `articles/article.html` (confirmed in `routes/articles.rs:1024`)

### Debug/Development Files
- `templates/admin/debug.html` - May be intentionally kept for debugging
- `templates/admin/simple.html` - Status unknown

## ✅ Verification Commands

To confirm unused templates:
```bash
# Search for template references in Rust code
grep -r "article-old\|simple.html\|modern.html" src/

# Search for template references in templates (extends/includes)
grep -r "article-old\|simple.html\|modern.html" templates/

# Check which templates actually load article-init.js
grep -r "article-init.js" templates/
```

## 🚀 Next Steps

1. **Phase 1**: Delete empty files (media-picker.js, media-uploader.js)
2. **Phase 2**: Verify template usage and delete unused templates
3. **Phase 3**: Refactor utility function duplications
4. **Phase 4**: Remove unused getCookie() or integrate it properly
