# Theme System Implementation Plan for CrustyRustacean Dev Blog

This document provides a comprehensive plan for implementing a transparent, flexible theme system that integrates with the existing CrustyRustacean Dev Blog codebase.

## Current Codebase State

### Existing Architecture
- **Framework**: Axum web framework on Shuttle.rs platform
- **Database**: libSQL/Turso cloud database
- **Templating**: Tera engine with 23 templates in `templates/` directory
- **Static Assets**: Served from `static/` via `ServeDir` at `/static` route
- **Configuration**: Loaded from Shuttle `SecretStore` (JWT_SECRET, EXTERNAL_STYLESHEET, OVERRIDE_STYLESHEET)
- **State Management**: `AppState` struct in `src/lib/state.rs` containing Tera engine, DB, JWT keys

### Current Theme System (Limited)
**Location**: `src/lib/state.rs:22-49` and `src/lib/config.rs:1-39`

**Current Implementation**:
```rust
// src/lib/config.rs
pub struct AppConfig {
    pub jwt_secret: String,
    pub external_stylesheet: String,  // Bootstrap CDN URL
    pub override_stylesheet: String,  // /static/css/styles.css
}

// src/lib/state.rs - setup_templates()
tera.register_function("theme_stylesheet", move |args| {
    let which = args.get("which") // "external" or "override"
    // Returns stylesheet URL based on which parameter
});
```

**Used in templates** (`templates/base.html:21-29`):
```html
<link rel="stylesheet" href="{{ theme_stylesheet(which='external') }}" />
<link rel="stylesheet" href="{{ theme_stylesheet(which='override') }}" />
```

### Pain Points
1. Hardcoded template paths in `Tera::new("templates/**/*")`
2. Single static directory with no theme isolation
3. No support for theme inheritance or overrides
4. Configuration limited to two stylesheet URLs
5. No metadata or versioning for themes
6. No way to list or switch themes dynamically

---

## Proposed Theme System Design

### 1. Theme Structure

```rust
// src/lib/themes/mod.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct ThemeConfig {
    pub metadata: ThemeMetadata,
    #[serde(default)]
    pub settings: HashMap<String, toml::Value>,
    #[serde(default)]
    pub overrides: ThemeOverrides,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThemeMetadata {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ThemeOverrides {
    pub templates: Option<HashMap<String, String>>, // "base" -> "custom-base.html"
    pub parent: Option<String>, // Inherit from another theme (e.g., "default")
}

pub struct Theme {
    pub name: String,
    pub path: PathBuf,
    pub config: ThemeConfig,
    pub template_paths: Vec<PathBuf>, // Ordered: custom -> parent -> default
    pub static_paths: Vec<PathBuf>,   // Static asset search paths
}
```

### 2. Theme Loader

```rust
// src/lib/themes/loader.rs
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct ThemeLoader {
    themes_dir: PathBuf,
}

impl ThemeLoader {
    pub fn new(themes_dir: PathBuf) -> Self {
        Self { themes_dir }
    }

    /// Load a theme by name, resolving parent chain
    pub fn load(&self, theme_name: &str) -> Result<Theme> {
        let theme_path = self.themes_dir.join(theme_name);
        
        // Load theme.toml
        let config_path = theme_path.join("theme.toml");
        let config_str = fs::read_to_string(&config_path)
            .context(format!("Failed to read theme config: {}", theme_name))?;
        
        let config: ThemeConfig = toml::from_str(&config_str)
            .context("Failed to parse theme.toml")?;

        // Build template search paths (most specific to least specific)
        let mut template_paths = vec![theme_path.join("templates")];
        
        // If theme has a parent, load it recursively
        if let Some(parent_name) = &config.overrides.parent {
            let parent = self.load(parent_name)?;
            template_paths.extend(parent.template_paths);
        }

        Ok(Theme {
            name: theme_name.to_string(),
            path: theme_path,
            config,
            template_paths,
        })
    }

    /// List all available themes
    pub fn list_themes(&self) -> Result<Vec<String>> {
        let mut themes = Vec::new();
        
        for entry in fs::read_dir(&self.themes_dir)? {
            let entry = entry?;
            if entry.path().join("theme.toml").exists() {
                if let Some(name) = entry.file_name().to_str() {
                    themes.push(name.to_string());
                }
            }
        }
        
        Ok(themes)
    }
}
```

### 3. Integration with Existing AppState

**Current**: `src/lib/state.rs:14-64`
**Updated**:

```rust
// src/lib/state.rs (UPDATED)
use crate::auth::Keys;
use crate::themes::{Theme, ThemeLoader};
use crate::{AppConfig, AppError, DatabaseConnection};
use axum_template::engine::Engine;
use std::collections::HashMap;
use tera::{Tera, Value};

type AppEngine = Engine<Tera>;

#[derive(Clone)]
pub struct AppState {
    pub engine: AppEngine,
    pub db: DatabaseConnection,
    pub jwt_keys: Keys,
    pub theme: Theme,  // NEW: Current active theme
}

fn setup_templates(config: &AppConfig, theme: &Theme) -> Result<Engine<Tera>, tera::Error> {
    let mut tera = Tera::default();

    // Load templates from theme hierarchy (custom -> parent -> default)
    for template_path in &theme.template_paths {
        let glob_pattern = format!("{}/**/*.html", template_path.display());
        if let Err(e) = tera.add_template_files(vec![(glob_pattern.as_str(), None)]) {
            tracing::warn!("Failed to load templates from {}: {}", template_path.display(), e);
        }
    }

    // Register theme-specific functions
    register_theme_functions(&mut tera, config, theme);

    Ok(Engine::from(tera))
}

fn register_theme_functions(tera: &mut Tera, config: &AppConfig, theme: &Theme) {
    // EXISTING: theme_stylesheet function (backward compatibility)
    let external = config.external_stylesheet.clone();
    let override_css = config.override_stylesheet.clone();

    tera.register_function(
        "theme_stylesheet",
        move |args: &HashMap<String, Value>| -> tera::Result<Value> {
            use tera::from_value;
            let which = args
                .get("which")
                .and_then(|v| from_value::<String>(v.clone()).ok())
                .unwrap_or_else(|| "external".to_string());

            let url = match which.as_str() {
                "external" => &external,
                "override" => &override_css,
                _ => "",
            };
            Ok(Value::String(url.to_string()))
        },
    );

    // NEW: theme_setting function
    let settings = theme.config.settings.clone();
    tera.register_function(
        "theme_setting",
        move |args: &HashMap<String, Value>| -> tera::Result<Value> {
            use tera::from_value;
            let key = args
                .get("key")
                .and_then(|v| from_value::<String>(v.clone()).ok())
                .ok_or_else(|| "key argument required")?;

            settings.get(&key)
                .and_then(|v| tera::to_value(v).ok())
                .ok_or_else(|| format!("Setting '{}' not found", key).into())
        }
    );

    // NEW: theme_asset function for theme-specific assets
    let theme_name = theme.name.clone();
    tera.register_function(
        "theme_asset",
        move |args: &HashMap<String, Value>| -> tera::Result<Value> {
            use tera::from_value;
            let path = args
                .get("path")
                .and_then(|v| from_value::<String>(v.clone()).ok())
                .ok_or_else(|| "path argument required")?;

            Ok(Value::String(format!("/themes/{}/static/{}", theme_name, path)))
        }
    );
}

impl AppState {
    pub fn new(db: DatabaseConnection, config: &AppConfig) -> Result<Self, AppError> {
        // Load theme from config (default to "default")
        let theme_name = std::env::var("THEME_NAME")
            .unwrap_or_else(|_| "default".to_string());

        let theme_loader = ThemeLoader::new(std::path::PathBuf::from("themes"));
        let theme = theme_loader.load(&theme_name)
            .map_err(|e| AppError::InternalServerError(format!("Failed to load theme: {}", e)))?;

        let engine = setup_templates(config, &theme)?;
        let jwt_keys = Keys::from_config(config);

        Ok(Self {
            engine,
            db,
            jwt_keys,
            theme,
        })
    }
}
```

### 4. Static Assets Routing (Axum Integration)

**Current**: `src/lib/startup.rs:120` serves static from single directory
**Updated**: Support multi-theme static asset serving

```rust
// src/lib/startup.rs (ADDITION to build_router)

// Add theme-specific static file serving
// This serves files from themes/{theme_name}/static/*
Router::new()
    // ... existing routes ...
    .nest_service("/static", ServeDir::new("static"))  // EXISTING: Global static assets
    .nest_service("/themes", ServeDir::new("themes"))   // NEW: Theme-specific assets
    // ... rest of router config
```

**No changes needed to route handlers** - they already use `AppState` which will now include theme info.

### 5. Directory Structure and Migration Plan

**Current Structure**:
```
crustyrustacean-dev-blog/
├── templates/          # 23 existing templates
│   ├── base.html
│   ├── index.html
│   ├── about.html
│   ├── privacy.html
│   ├── terms.html
│   ├── admin/
│   ├── articles/
│   ├── auth/
│   ├── profile/
│   └── errors/
└── static/            # Global static assets
    ├── css/styles.css
    ├── js/*.js (18 files)
    ├── hero.jpg
    └── favicon.png
```

**New Structure** (after theme system implementation):
```
crustyrustacean-dev-blog/
├── themes/
│   └── default/                    # Default theme (migrated from templates/)
│       ├── theme.toml
│       ├── templates/              # All 23 existing templates moved here
│       │   ├── base.html
│       │   ├── index.html
│       │   ├── about.html
│       │   ├── privacy.html
│       │   ├── terms.html
│       │   ├── admin/
│       │   │   ├── dashboard.html
│       │   │   ├── tags.html
│       │   │   └── api-keys.html
│       │   ├── articles/
│       │   │   ├── article.html
│       │   │   ├── list.html
│       │   │   ├── editor.html
│       │   │   └── feed.html
│       │   ├── auth/
│       │   │   ├── login.html
│       │   │   └── register.html
│       │   ├── profile/
│       │   │   ├── profile.html
│       │   │   ├── authors.html
│       │   │   └── favorites.html
│       │   └── errors/
│       │       └── 404.html
│       └── static/                 # Theme-specific assets
│           ├── css/
│           │   └── theme.css       # Theme-specific styling
│           ├── js/
│           │   └── theme.js        # Theme-specific scripts
│           └── images/
│               └── logo.png
│
├── static/                         # REMAINS: Global shared assets
│   ├── css/styles.css              # Global CSS (Bootstrap overrides)
│   ├── js/*.js                     # All 18 JS modules (unchanged)
│   ├── hero.jpg
│   └── favicon.png
│
└── templates/                      # DEPRECATED: Keep for backward compat initially
    └── (empty or fallback templates)
```

**Example Custom Theme**:
```
themes/
  custom-dark/
    theme.toml                      # Inherits from "default"
    templates/
      base.html                     # Override base template only
      articles/
        article.html                # Custom article layout
    static/
      css/
        dark-theme.css              # Dark mode styles
```

**Example theme.toml files**:

```toml
# themes/default/theme.toml
[metadata]
name = "CrustyRustacean Default Theme"
version = "1.0.0"
author = "CrustyRustacean Team"
description = "The default Bootstrap-based theme for CrustyRustacean Dev Blog"

[settings]
brand_name = "CrustyRustacean"
footer_text = "Learning about solving the things, mostly in Rust...mostly."
show_analytics = true
bootstrap_version = "5.1.3"
font_awesome_version = "6.0.0"

# No parent - this is the base theme
```

```toml
# themes/custom-dark/theme.toml
[metadata]
name = "Dark Mode Theme"
version = "1.0.0"
author = "Jeff"
description = "A dark mode variant of the default theme"

[settings]
brand_name = "CrustyRustacean"
footer_text = "Built with Rust 🦀"
color_scheme = "dark"
accent_color = "#bb86fc"

[overrides]
parent = "default"  # Inherit from default theme
```

### 6. Template Usage Examples

**Updated base.html** (using new theme functions):

```html
<!-- themes/default/templates/base.html -->
<!doctype html>
<html lang="en">
<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{% block title %}{{ theme_setting(key='brand_name') }}{% endblock %}</title>

    <!-- EXISTING: Backward compatible stylesheet loading -->
    <link rel="stylesheet" href="{{ theme_stylesheet(which='external') }}" />

    <!-- NEW: Theme-specific assets -->
    <link rel="stylesheet" href="{{ theme_asset(path='css/theme.css') }}" />

    <!-- EXISTING: Global override stylesheet -->
    <link rel="stylesheet" href="{{ theme_stylesheet(which='override') }}" />

    <link rel="icon" href="/static/favicon.png" />
    <link href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/{{ theme_setting(key='font_awesome_version') }}/css/all.min.css" rel="stylesheet" />
</head>
<body class="d-flex flex-column min-vh-100">
    <!-- Navbar with theme settings -->
    <nav class="navbar navbar-expand-lg navbar-dark bg-dark">
        <div class="container">
            <a class="navbar-brand" href="/">
                <i class="fas fa-laptop-code me-2"></i>
                {{ theme_setting(key='brand_name') }}
            </a>
            <!-- ... rest of navbar ... -->
        </div>
    </nav>

    <main class="container mt-4 flex-grow-1">
        {% block content %}{% endblock %}
    </main>

    <footer class="bg-dark text-light mt-auto py-4">
        <div class="container">
            <div class="row">
                <div class="col-md-6">
                    <h5>{{ theme_setting(key='brand_name') }}</h5>
                    <p class="mb-0">{{ theme_setting(key='footer_text') }}</p>
                </div>
                <!-- ... rest of footer ... -->
            </div>
        </div>
    </footer>

    <script src="https://cdn.jsdelivr.net/npm/bootstrap@{{ theme_setting(key='bootstrap_version') }}/dist/js/bootstrap.bundle.min.js"></script>
    <script src="/static/js/base.js"></script>
    {% block scripts %}{% endblock %}
</body>
</html>
```

---

## Implementation Phases

### Phase 1: Core Theme Infrastructure
**Files to create**:
- `src/lib/themes/mod.rs` - Theme structs and module exports
- `src/lib/themes/loader.rs` - ThemeLoader implementation

**Files to modify**:
- `src/lib/lib.rs` - Add `pub mod themes;` export
- `Cargo.toml` - Add `toml` dependency if not present

**Goal**: Basic theme loading functionality without template integration

**Tests**:
- Load theme from directory
- Parse theme.toml correctly
- Handle parent theme resolution
- List available themes

---

### Phase 2: AppState Integration
**Files to modify**:
- `src/lib/state.rs` - Update AppState, setup_templates, register_theme_functions
- `src/lib/config.rs` - Optional: Add theme configuration fields

**Goal**: Integrate theme system into existing AppState and Tera setup

**Tests**:
- Theme loaded during AppState::new()
- Template functions registered correctly
- Backward compatibility with existing theme_stylesheet()

---

### Phase 3: Static Asset Routing
**Files to modify**:
- `src/lib/startup.rs` - Add `/themes` route for ServeDir
- `Shuttle.toml` - Update assets array to include themes/

**Goal**: Enable serving static assets from theme directories

**Tests**:
- Theme static assets accessible via /themes/{theme}/static/
- Global /static/ assets still work

---

### Phase 4: Default Theme Migration
**Tasks**:
1. Create `themes/default/` directory structure
2. Move all templates from `templates/` to `themes/default/templates/`
3. Create `themes/default/theme.toml` with metadata
4. Create `themes/default/static/` for theme-specific assets
5. Update templates to use new theme functions (optional)

**Files created**:
- `themes/default/theme.toml`
- `themes/default/templates/**/*` (23 templates)
- `themes/default/static/css/theme.css`
- `themes/default/static/js/theme.js`

**Goal**: Migrate existing templates to default theme without breaking functionality

**Tests**:
- All existing tests pass unchanged
- All pages render correctly
- Static assets load properly

---

### Phase 5: Shuttle Configuration
**Files to modify**:
- `Secrets.dev.toml` - Add THEME_NAME = "default"
- `Secrets.toml` (production) - Add THEME_NAME secret
- `Shuttle.toml` - Ensure themes/ included in assets

**Goal**: Configure theme selection via Shuttle secrets

---

### Phase 6: Testing & Documentation
**Tasks**:
1. Add comprehensive unit tests for theme system
2. Add integration tests for theme loading
3. Update CLAUDE.md with theme system documentation
4. Update README.md with theme usage instructions
5. Create example custom theme

**Files to create**:
- `tests/api/themes.rs` - Integration tests
- `src/lib/themes/mod.rs` - Unit tests
- `themes/example-custom/` - Example custom theme
- `docs/THEME_DEVELOPMENT.md` - Theme development guide

---

## Configuration & Deployment

### Shuttle Secrets (Secrets.dev.toml / Secrets.toml)

**Current**:
```toml
JWT_SECRET = "..."
TURSO_DATABASE_URL = "..."
TURSO_AUTH_TOKEN = "..."
EXTERNAL_STYLESHEET = "https://cdn.jsdelivr.net/npm/bootstrap@5.1.3/dist/css/bootstrap.min.css"
OVERRIDE_STYLESHEET = "/static/css/styles.css"
```

**Updated** (add):
```toml
# Theme configuration (NEW)
THEME_NAME = "default"  # or "custom-dark", etc.
```

### Shuttle.toml Updates

**Current**:
```toml
[build]
assets = [
  "templates/*",
  "static/*",
  "assets/*"
]
```

**Updated**:
```toml
[build]
assets = [
  "themes/*",      # NEW: Include all themes
  "static/*",      # Keep global static assets
  "assets/*"
]
# Remove "templates/*" once fully migrated
```

### Environment Variable

Alternatively, set theme via environment variable:
```bash
# Local development
export THEME_NAME=custom-dark
shuttle run

# Or in code startup
std::env::var("THEME_NAME").unwrap_or_else(|_| "default".to_string())
```

---

## Key Benefits

1. **Transparent**: Every file is visible in `themes/` directory, no magic
2. **Overridable**: Child themes only need to provide what they change
3. **Discoverable**: Want to change something? Look at default theme files
4. **Type-safe**: Rust ensures theme loading errors are caught at startup
5. **Fast**: Templates compiled once at startup, no runtime overhead
6. **Backward Compatible**: Existing templates/routes work unchanged
7. **Flexible**: Themes can inherit from each other
8. **Configurable**: Theme settings via TOML, accessible in templates
9. **Production Ready**: Works with Shuttle deployment and asset bundling

---

## Migration Strategy

### Minimal Breaking Changes
1. Keep `templates/` directory initially for backward compatibility
2. Load themes from `themes/` first, fall back to `templates/`
3. Gradually migrate templates to `themes/default/`
4. Once stable, deprecate `templates/` directory

### Rollback Plan
If issues arise:
1. Set `THEME_NAME=""` to disable theme system
2. Fall back to loading from `templates/` directory
3. Keep existing `setup_templates()` logic as fallback

---

## Testing Strategy

### Unit Tests
- Theme config parsing (valid/invalid TOML)
- Parent theme resolution (circular detection)
- Template path ordering
- Static path resolution

### Integration Tests
- Full theme loading in AppState
- Template rendering with theme functions
- Static asset serving from themes
- Theme switching (different THEME_NAME values)
- Backward compatibility with existing templates

### Manual Testing Checklist
- [ ] All 23 templates render correctly
- [ ] Static CSS/JS loads properly
- [ ] Theme settings accessible in templates
- [ ] Parent theme inheritance works
- [ ] Custom theme can override specific templates
- [ ] Shuttle deployment includes themes
- [ ] All 122 existing tests still pass

---

## Success Criteria

✅ **Phase 1 Complete**: Theme loader can parse theme.toml and resolve parents
✅ **Phase 2 Complete**: AppState loads theme, Tera functions registered
✅ **Phase 3 Complete**: Static assets served from /themes/{name}/static/
✅ **Phase 4 Complete**: All templates migrated to themes/default/, site works unchanged
✅ **Phase 5 Complete**: Theme selectable via THEME_NAME secret/env var
✅ **Phase 6 Complete**: Tests pass, documentation updated, example theme exists

---

## Future Enhancements

- **Admin UI**: Theme switcher in admin dashboard
- **Theme Marketplace**: Download/install themes from registry
- **Live Preview**: Preview theme changes without restart
- **Per-User Themes**: Allow users to select their preferred theme
- **Theme Validation**: CLI tool to validate theme structure
- **Hot Reload**: Auto-reload templates in development mode