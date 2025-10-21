---
title: "[Phase 0.5] Theme System Foundation"
labels: enhancement, phase-0.5, priority-critical, architecture
assignees:
milestone: Phase 0.5 - Theme Foundation
---

## Summary

Establish theme system foundation to decouple styling from functionality before building Phase 1 features. This prevents hardcoding Bootstrap throughout new features and enables theme flexibility.

## Priority

**CRITICAL** - Architectural foundation that prevents technical debt

## Estimated Time

1-2 weeks

## Problem

Currently, Bootstrap classes are hardcoded throughout all 22+ templates:
- `templates/base.html` - Bootstrap navbar, grid
- `templates/articles/*.html` - Bootstrap cards, containers
- `templates/admin/*.html` - Bootstrap forms
- All new features will inherit this tight coupling

**Risk:** If we build Phase 1 features (media library, settings, admin UIs) with hardcoded Bootstrap, we'll need to refactor everything when implementing the theme system.

**Solution:** Build lightweight theme foundation NOW, then build all new features theme-agnostic from day one.

## Goals

1. ✅ Move templates into theme directory structure
2. ✅ Create theme metadata system
3. ✅ Abstract CSS framework with semantic classes
4. ✅ Build reusable component system
5. ✅ Implement basic theme loader in backend
6. ✅ Maintain backward compatibility (everything still works)

## Implementation Tasks

### Week 1: Structure & Migration

#### 1. Create Theme Directory Structure

- [ ] Create `themes/` directory at project root
- [ ] Create `themes/default/` subdirectory
- [ ] Create `themes/default/templates/` for HTML templates
- [ ] Create `themes/default/static/css/` for theme CSS
- [ ] Create `themes/default/static/js/` for theme JavaScript
- [ ] Create `themes/default/components/` for reusable parts

**Structure:**
```
themes/
└── default/
    ├── theme.toml              # Theme metadata
    ├── templates/              # All Tera templates
    │   ├── base.html
    │   ├── index.html
    │   ├── articles/
    │   │   ├── list.html
    │   │   ├── article.html
    │   │   └── editor.html
    │   ├── auth/
    │   ├── profile/
    │   ├── admin/
    │   ├── components/         # Reusable components
    │   │   ├── navbar.html
    │   │   ├── footer.html
    │   │   ├── card.html
    │   │   ├── button.html
    │   │   └── form.html
    │   └── errors/
    └── static/
        ├── css/
        │   ├── theme.css       # Theme-specific styles
        │   └── variables.css   # CSS custom properties
        └── js/
            └── theme.js        # Theme-specific JS
```

#### 2. Create Theme Metadata

- [ ] Create `theme.toml` configuration file
- [ ] Define theme metadata (name, version, author)
- [ ] Specify CSS framework used
- [ ] Document template parts/components
- [ ] Add theme screenshot (optional)

**File:** `themes/default/theme.toml`
```toml
[theme]
name = "Default Bootstrap Theme"
version = "1.0.0"
author = "CrustyRustacean"
description = "Bootstrap 5 based responsive theme"
screenshot = "screenshot.png"

[css_framework]
name = "bootstrap"
version = "5.3"
cdn = "https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css"

[template_parts]
navbar = "components/navbar.html"
footer = "components/footer.html"
article_card = "components/card.html"
button = "components/button.html"

[widget_areas]
sidebar = "Sidebar widget area"
footer_1 = "Footer column 1"
footer_2 = "Footer column 2"
```

#### 3. Migrate Existing Templates

- [ ] Move all files from `templates/` → `themes/default/templates/`
- [ ] Preserve directory structure
- [ ] Keep original `templates/` as symlink for backward compatibility (optional)
- [ ] Update any hardcoded template paths in tests

#### 4. Backend Theme Loader

- [ ] Create `src/lib/theme.rs` module
- [ ] Implement `Theme` struct
- [ ] Implement theme loading from `theme.toml`
- [ ] Update `AppState` to include active theme
- [ ] Modify Tera initialization to load from theme directory
- [ ] Add theme validation (required files exist)

**File:** `src/lib/theme.rs` (new)
```rust
use serde::Deserialize;
use std::path::{Path, PathBuf};
use crate::errors::AppError;

#[derive(Debug, Clone, Deserialize)]
pub struct ThemeConfig {
    pub theme: ThemeMetadata,
    pub css_framework: Option<CssFramework>,
    pub template_parts: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThemeMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CssFramework {
    pub name: String,
    pub version: String,
    pub cdn: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub path: PathBuf,
    pub config: ThemeConfig,
}

impl Theme {
    /// Load a theme by name from the themes directory
    pub fn load(theme_name: &str) -> Result<Self, AppError> {
        let theme_path = PathBuf::from("themes").join(theme_name);

        if !theme_path.exists() {
            return Err(AppError::NotFound(
                format!("Theme '{}' not found", theme_name)
            ));
        }

        // Load theme.toml
        let config_path = theme_path.join("theme.toml");
        let config_str = std::fs::read_to_string(&config_path)
            .map_err(|e| AppError::InternalServerError(
                format!("Failed to read theme config: {}", e)
            ))?;

        let config: ThemeConfig = toml::from_str(&config_str)
            .map_err(|e| AppError::InternalServerError(
                format!("Failed to parse theme config: {}", e)
            ))?;

        // Validate required directories exist
        let templates_path = theme_path.join("templates");
        if !templates_path.exists() {
            return Err(AppError::InternalServerError(
                format!("Theme '{}' missing templates directory", theme_name)
            ));
        }

        Ok(Self {
            name: theme_name.to_string(),
            path: theme_path,
            config,
        })
    }

    /// Get the path to the templates directory
    pub fn templates_path(&self) -> PathBuf {
        self.path.join("templates")
    }

    /// Get the path to the static assets directory
    pub fn static_path(&self) -> PathBuf {
        self.path.join("static")
    }

    /// Get the path to a specific template
    pub fn template_path(&self, name: &str) -> PathBuf {
        self.templates_path().join(name)
    }
}
```

#### 5. Update AppState and Tera Loading

- [ ] Add `theme: Theme` to `AppState`
- [ ] Update Tera initialization in `main.rs` to load from theme directory
- [ ] Add theme static file serving route
- [ ] Test that all existing pages still render

**File:** `src/lib/state.rs`
```rust
use crate::theme::Theme;

#[derive(Clone)]
pub struct AppState {
    pub tera: Arc<Tera>,
    pub db: DatabaseConnection,
    pub keys: Keys,
    pub theme: Theme,  // NEW
}
```

**File:** `src/bin/main.rs` (modify Tera initialization)
```rust
// Load active theme (from settings or default)
let theme = Theme::load("default")?;

// Initialize Tera with theme templates
let template_pattern = format!("{}/**/*.html", theme.templates_path().display());
let mut tera = match Tera::new(&template_pattern) {
    Ok(t) => t,
    Err(e) => {
        eprintln!("Tera parsing error(s): {}", e);
        std::process::exit(1);
    }
};

tera.autoescape_on(vec![".html"]);
```

---

### Week 2: Components & Abstraction

#### 6. Extract Common Components

- [ ] Create `themes/default/templates/components/navbar.html`
- [ ] Create `themes/default/templates/components/footer.html`
- [ ] Create `themes/default/templates/components/card.html`
- [ ] Create `themes/default/templates/components/button.html`
- [ ] Create `themes/default/templates/components/form.html`

**Example:** `themes/default/templates/components/button.html`
```html
{% macro primary(text, url="#", icon="") %}
  <a href="{{ url }}" class="theme-btn theme-btn-primary">
    {% if icon %}<i class="{{ icon }}"></i>{% endif %}
    {{ text }}
  </a>
{% endmacro %}

{% macro secondary(text, url="#") %}
  <a href="{{ url }}" class="theme-btn theme-btn-secondary">
    {{ text }}
  </a>
{% endmacro %}

{% macro danger(text, url="#") %}
  <a href="{{ url }}" class="theme-btn theme-btn-danger">
    {{ text }}
  </a>
{% endmacro %}
```

**Example:** `themes/default/templates/components/card.html`
```html
{% macro article_card(article, show_author=true) %}
  <article class="theme-card">
    <div class="theme-card-body">
      <h2 class="theme-card-title">
        <a href="/articles/{{ article.slug }}">{{ article.title }}</a>
      </h2>
      <p class="theme-card-description">{{ article.description }}</p>

      {% if show_author %}
      <div class="theme-card-meta">
        <span class="theme-author">{{ article.author.username }}</span>
        <span class="theme-date">{{ article.created_at }}</span>
      </div>
      {% endif %}

      {% if article.tags %}
      <div class="theme-tags">
        {% for tag in article.tags %}
          <a href="/articles?tag={{ tag }}" class="theme-tag">{{ tag }}</a>
        {% endfor %}
      </div>
      {% endif %}
    </div>
  </article>
{% endmacro %}
```

#### 7. Create Semantic CSS Classes

- [ ] Create `themes/default/static/css/theme.css`
- [ ] Create `themes/default/static/css/variables.css`
- [ ] Define semantic class mappings to Bootstrap
- [ ] Use CSS custom properties for theming

**File:** `themes/default/static/css/variables.css`
```css
:root {
  /* Colors */
  --theme-primary: #0d6efd;
  --theme-secondary: #6c757d;
  --theme-success: #198754;
  --theme-danger: #dc3545;
  --theme-warning: #ffc107;
  --theme-info: #0dcaf0;

  /* Typography */
  --theme-font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  --theme-font-size-base: 1rem;
  --theme-line-height-base: 1.5;

  /* Spacing */
  --theme-spacing-xs: 0.25rem;
  --theme-spacing-sm: 0.5rem;
  --theme-spacing-md: 1rem;
  --theme-spacing-lg: 1.5rem;
  --theme-spacing-xl: 3rem;

  /* Layout */
  --theme-container-max-width: 1200px;
  --theme-border-radius: 0.375rem;
}
```

**File:** `themes/default/static/css/theme.css`
```css
/* Semantic class mappings to Bootstrap */

/* Layout */
.theme-container { @extend .container; }
.theme-grid { @extend .row; }
.theme-col-main { @extend .col-md-8; }
.theme-col-sidebar { @extend .col-md-4; }

/* Buttons */
.theme-btn { @extend .btn; }
.theme-btn-primary { @extend .btn-primary; }
.theme-btn-secondary { @extend .btn-secondary; }
.theme-btn-danger { @extend .btn-danger; }

/* Cards */
.theme-card { @extend .card; }
.theme-card-body { @extend .card-body; }
.theme-card-title { @extend .card-title; }

/* Forms */
.theme-form { @extend .form; }
.theme-form-group { @extend .mb-3; }
.theme-form-label { @extend .form-label; }
.theme-form-input { @extend .form-control; }
.theme-form-textarea { @extend .form-control; }

/* Tags */
.theme-tag { @extend .badge; @extend .bg-secondary; }

/* Or use CSS custom properties (no build step needed): */
.theme-btn {
  display: inline-block;
  padding: 0.375rem 0.75rem;
  border-radius: var(--theme-border-radius);
  font-family: var(--theme-font-family);
  text-decoration: none;
  cursor: pointer;
}

.theme-btn-primary {
  background-color: var(--theme-primary);
  color: white;
}
```

#### 8. Update Base Template

- [ ] Update `themes/default/templates/base.html` to use components
- [ ] Replace hardcoded Bootstrap navbar with component
- [ ] Replace hardcoded footer with component
- [ ] Add theme CSS/JS loading
- [ ] Add CSS custom properties

**File:** `themes/default/templates/base.html`
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}CrustyRustacean Dev Blog{% endblock %}</title>

    <!-- CSS Framework (Bootstrap for default theme) -->
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css" rel="stylesheet">

    <!-- Theme CSS Variables -->
    <link rel="stylesheet" href="/theme/static/css/variables.css">

    <!-- Theme Custom CSS -->
    <link rel="stylesheet" href="/theme/static/css/theme.css">

    <!-- Site CSS -->
    <link rel="stylesheet" href="/static/css/styles.css">

    {% block extra_css %}{% endblock %}
</head>
<body>
    <!-- Use navbar component instead of hardcoded HTML -->
    {% include "components/navbar.html" %}

    <main class="theme-container">
        {% block content %}{% endblock %}
    </main>

    <!-- Use footer component -->
    {% include "components/footer.html" %}

    <!-- JS Framework -->
    <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/js/bootstrap.bundle.min.js"></script>

    <!-- Theme JS -->
    <script src="/theme/static/js/theme.js"></script>

    <!-- Site JS -->
    <script src="/static/js/base.js"></script>

    {% block extra_js %}{% endblock %}
</body>
</html>
```

#### 9. Update One Template as Example

- [ ] Refactor `themes/default/templates/articles/list.html`
- [ ] Replace hardcoded Bootstrap with theme components
- [ ] Use semantic CSS classes
- [ ] Import and use card component macro

**Example:** `themes/default/templates/articles/list.html`
```html
{% extends "base.html" %}
{% import "components/card.html" as card %}
{% import "components/button.html" as btn %}

{% block content %}
<div class="theme-container">
    <div class="theme-grid">
        <div class="theme-col-main">
            <h1>Articles</h1>

            {% for article in articles %}
                {{ card::article_card(article=article, show_author=true) }}
            {% endfor %}

            <div class="theme-pagination">
                <!-- Pagination component -->
            </div>
        </div>

        <aside class="theme-col-sidebar">
            <!-- Sidebar widgets -->
        </aside>
    </div>
</div>
{% endblock %}
```

#### 10. Add Theme Static File Serving

- [ ] Add route for `/theme/static/*` to serve theme assets
- [ ] Update startup.rs with theme static route
- [ ] Test CSS and JS loading

**File:** `src/lib/startup.rs`
```rust
// Add theme static file serving
let theme_static_path = state.theme.static_path();
let theme_static_service = ServeDir::new(theme_static_path);

let app = Router::new()
    // ... existing routes
    .nest_service("/theme/static", theme_static_service)
    .nest_service("/static", static_service)
    // ...
```

---

### Testing

- [ ] All existing pages render correctly
- [ ] Templates load from theme directory
- [ ] Theme static files (CSS/JS) are accessible
- [ ] Components render properly
- [ ] No broken styles or layouts
- [ ] Test with different theme directory
- [ ] Verify backward compatibility

**Test Cases:**
1. ✅ Homepage loads with theme
2. ✅ Article list page uses card component
3. ✅ Navigation bar component renders
4. ✅ Footer component renders
5. ✅ Theme CSS loaded correctly
6. ✅ Theme JS loaded correctly
7. ✅ All existing tests pass
8. ✅ Can switch theme name in config

---

### Documentation

- [ ] Document theme structure in README
- [ ] Create THEME_DEVELOPMENT.md guide
- [ ] Update CODEBASE_INDEX.md with theme system
- [ ] Add comments to theme.toml explaining options
- [ ] Document component macro usage

---

## Files to Create

```
themes/default/
├── theme.toml
├── templates/
│   └── components/
│       ├── navbar.html
│       ├── footer.html
│       ├── card.html
│       ├── button.html
│       └── form.html
└── static/
    ├── css/
    │   ├── variables.css
    │   └── theme.css
    └── js/
        └── theme.js

src/lib/theme.rs
.github/issues/phase0/issue-02-theme-foundation.md
THEME_DEVELOPMENT.md
```

## Files to Modify

- `src/lib/state.rs` - Add theme to AppState
- `src/bin/main.rs` - Update Tera initialization
- `src/lib/startup.rs` - Add theme static route
- `src/lib/lib.rs` - Export theme module
- `themes/default/templates/base.html` - Use components
- `themes/default/templates/articles/list.html` - Example refactor
- `Cargo.toml` - Add `toml` crate dependency
- `CODEBASE_INDEX.md` - Document theme system
- `PROJECT_PLAN.md` - Add Phase 0.5

## Dependencies to Add

```toml
[dependencies]
toml = "0.8"  # For parsing theme.toml
```

## Migration Strategy

**Step 1: Structure (no breaking changes)**
- Create theme directory
- Copy templates to theme
- Load from new location
- Everything still works

**Step 2: Components (gradual refactor)**
- Extract one component at a time
- Update one template at a time
- Test after each change
- Old templates still work during migration

**Step 3: Abstraction (future-proofing)**
- Add semantic CSS classes
- Create CSS custom properties
- Update templates to use semantic classes
- Now can swap CSS frameworks easily

## Success Criteria

- ✅ All templates moved to `themes/default/templates/`
- ✅ Theme metadata loaded from `theme.toml`
- ✅ Tera loads templates from theme directory
- ✅ Theme static files served correctly
- ✅ At least 5 reusable components created
- ✅ At least 1 template refactored to use components
- ✅ Semantic CSS classes defined
- ✅ All existing functionality works (no regressions)
- ✅ All existing tests pass
- ✅ Documentation updated

## Benefits

✅ **Future-proof** - Can swap CSS frameworks without touching templates
✅ **Component reuse** - Build features faster with shared components
✅ **Theme flexibility** - Easy to add new themes later
✅ **Better organization** - Clear separation of concerns
✅ **Phase 1 ready** - New features built theme-agnostic from day one
✅ **No rework** - One-time refactor instead of continuous technical debt

## Related Documents

- `PROJECT_PLAN.md` - Phase 0.5
- WordPress Theme Developer Handbook (inspiration)

## Dependencies

- Should be completed after tag editing (#1)
- Must be completed before Phase 1 features (media library, settings, etc.)

## Timeline

- **Week 1:** Structure, migration, backend loader
- **Week 2:** Components, CSS abstraction, documentation
- **Total:** 1-2 weeks depending on thoroughness

## Next Steps After Completion

Once theme foundation is established:
- All Phase 1 features (media library, settings, drafts) will be built using theme components
- Settings UI can include theme selector
- Future themes can be added without touching core code
- Community themes become possible
