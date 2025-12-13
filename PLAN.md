# Modular Theming System - Implementation Plan

## Executive Summary

This plan outlines the implementation of a modular theming system for the CrustyRustacean Dev Blog. The system will support pluggable themes via `theme.toml` configuration files and leverage modern CSS features for a maintainable, performant theming architecture.

---

## Current State Analysis

### What Exists
- **CSS Custom Properties**: Already in use in `static/css/styles.css` (`:root` variables for colors, shadows, radii)
- **Tera Template Functions**: `theme_stylesheet()` and `versioned_asset()` already registered
- **Basic Theme Config**: `external_stylesheet` and `override_stylesheet` in `AppConfig`
- **Bootstrap 5**: External CSS framework loaded via CDN

### What's Missing
- No theme switching mechanism
- No `theme.toml` configuration format
- No user preference persistence
- No dark mode support
- No theme discovery/loading system

---

## Proposed Architecture

### 1. Theme Directory Structure

```
themes/
├── default/
│   ├── theme.toml           # Theme metadata and configuration
│   ├── variables.css        # CSS custom properties
│   └── overrides.css        # Component-specific overrides (optional)
├── dark/
│   ├── theme.toml
│   ├── variables.css
│   └── overrides.css
├── high-contrast/
│   ├── theme.toml
│   └── variables.css
└── _base.css                # Shared base styles (current styles.css refactored)
```

### 2. Theme Configuration Format (`theme.toml`)

```toml
[theme]
id = "default"
name = "Crusty Light"
description = "The default light theme for CrustyRustacean"
version = "1.0.0"
author = "CrustyRustacean"

[theme.supports]
color_scheme = "light"          # "light", "dark", or "auto"
high_contrast = false

[theme.colors]
# Primary palette
primary = "#ff6b35"
primary-hover = "#e55a2b"
secondary = "#2c3e50"
accent = "#e74c3c"

# Backgrounds
bg-body = "#ffffff"
bg-surface = "#f8f9fa"
bg-elevated = "#ffffff"
bg-gradient-start = "#667eea"
bg-gradient-end = "#764ba2"

# Text
text-primary = "#000000"
text-secondary = "#2d3748"
text-muted = "#6c757d"
text-on-primary = "#ffffff"

# Semantic colors
success = "#28a745"
warning = "#ffc107"
danger = "#dc3545"
info = "#17a2b8"

# Borders and shadows
border-color = "#dee2e6"
border-radius = "12px"
shadow = "0 2px 15px rgba(0, 0, 0, 0.1)"
shadow-hover = "0 5px 25px rgba(0, 0, 0, 0.15)"

[theme.fonts]
body = "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
heading = "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
monospace = "'Monaco', 'Menlo', 'Ubuntu Mono', monospace"

[theme.external]
# Optional external dependencies
stylesheets = [
    "https://cdn.jsdelivr.net/npm/bootstrap@5.1.3/dist/css/bootstrap.min.css"
]

[theme.meta]
preview_colors = ["#ff6b35", "#2c3e50", "#667eea"]  # For theme picker UI
```

### 3. Modern CSS Architecture

#### 3.1 CSS Layers (`@layer`)
Organize CSS specificity for predictable overrides:

```css
/* Base layer structure */
@layer reset, base, components, utilities, theme;

@layer base {
    /* Typography, layout defaults */
}

@layer components {
    /* Card, button, form styles */
}

@layer theme {
    /* Theme-specific overrides - highest priority */
}
```

#### 3.2 CSS Custom Properties with Fallbacks
```css
:root {
    /* Theme variables with sensible fallbacks */
    --color-primary: var(--theme-primary, #ff6b35);
    --color-bg-body: var(--theme-bg-body, #ffffff);

    /* Computed properties using modern CSS */
    --color-primary-rgb: var(--theme-primary-rgb, 255, 107, 53);
    --color-primary-alpha-10: rgb(var(--color-primary-rgb) / 0.1);
}
```

#### 3.3 Light/Dark Mode with `color-scheme` and `light-dark()`
```css
:root {
    color-scheme: light dark;
}

body {
    /* Modern CSS: light-dark() function (Chrome 123+, Firefox 120+, Safari 17.5+) */
    background-color: light-dark(var(--bg-light), var(--bg-dark));
    color: light-dark(var(--text-light), var(--text-dark));
}

/* Fallback for older browsers */
@supports not (background: light-dark(white, black)) {
    :root {
        --color-bg-body: #ffffff;
        --color-text: #000000;
    }

    @media (prefers-color-scheme: dark) {
        :root {
            --color-bg-body: #1a1a1a;
            --color-text: #f0f0f0;
        }
    }
}
```

#### 3.4 Container Queries for Component Theming
```css
.card {
    container-type: inline-size;
    container-name: card;
}

@container card (min-width: 400px) {
    .card-body {
        padding: 2rem;
        font-size: 1.1rem;
    }
}
```

#### 3.5 Color Mixing for Dynamic Variations
```css
:root {
    --color-primary: #ff6b35;

    /* Generate hover states dynamically */
    --color-primary-hover: color-mix(in oklch, var(--color-primary), black 15%);
    --color-primary-light: color-mix(in oklch, var(--color-primary), white 30%);
    --color-primary-alpha: color-mix(in srgb, var(--color-primary) 20%, transparent);
}
```

---

## Implementation Steps

### Phase 1: Refactor Current CSS (Foundation)

**Files to modify:**
- `static/css/styles.css` → Split into base + theme variables

**Tasks:**
1. Create `static/css/base.css` with theme-agnostic styles
2. Create `static/css/variables.css` with all CSS custom properties
3. Implement CSS `@layer` structure for specificity management
4. Add `color-scheme: light dark` support

**Deliverables:**
- Refactored CSS with clear separation of concerns
- No visual changes (backwards compatible)

### Phase 2: Theme Configuration System

**New files:**
- `themes/default/theme.toml`
- `themes/dark/theme.toml`
- `src/lib/theme.rs`

**Tasks:**
1. Create `Theme` struct and TOML deserialization
2. Implement theme discovery (scan `themes/` directory)
3. Create theme validation logic
4. Generate CSS from `theme.toml` at startup or build time

**Rust Implementation:**

```rust
// src/lib/theme.rs
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct ThemeConfig {
    pub theme: ThemeMetadata,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThemeMetadata {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub author: Option<String>,
    pub supports: ThemeSupports,
    pub colors: HashMap<String, String>,
    pub fonts: Option<ThemeFonts>,
    pub external: Option<ThemeExternal>,
    pub meta: Option<ThemeMeta>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThemeSupports {
    pub color_scheme: String,  // "light", "dark", "auto"
    pub high_contrast: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThemeFonts {
    pub body: Option<String>,
    pub heading: Option<String>,
    pub monospace: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThemeExternal {
    pub stylesheets: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThemeMeta {
    pub preview_colors: Option<Vec<String>>,
}

impl ThemeConfig {
    pub fn load(path: &str) -> Result<Self, toml::de::Error> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| toml::de::Error::custom(e.to_string()))?;
        toml::from_str(&content)
    }

    /// Generate CSS custom properties from theme colors
    pub fn to_css(&self) -> String {
        let mut css = String::from(":root {\n");

        for (key, value) in &self.theme.colors {
            css.push_str(&format!("    --theme-{}: {};\n", key, value));
        }

        if let Some(fonts) = &self.theme.fonts {
            if let Some(body) = &fonts.body {
                css.push_str(&format!("    --theme-font-body: {};\n", body));
            }
            if let Some(heading) = &fonts.heading {
                css.push_str(&format!("    --theme-font-heading: {};\n", heading));
            }
            if let Some(mono) = &fonts.monospace {
                css.push_str(&format!("    --theme-font-mono: {};\n", mono));
            }
        }

        css.push_str("}\n");
        css
    }
}
```

### Phase 3: Theme Loading & Switching

**Files to modify:**
- `src/lib/state.rs` - Add theme to AppState
- `src/lib/config.rs` - Add active theme config
- `templates/base.html` - Dynamic theme CSS injection

**Tasks:**
1. Load available themes at startup
2. Store active theme in `AppState`
3. Create Tera function `theme_css()` to inject theme variables
4. Add `<meta name="color-scheme">` based on theme

**Template Changes:**
```html
<head>
    <meta name="color-scheme" content="{{ theme.color_scheme }}">

    <!-- Base styles (framework + base) -->
    <link rel="stylesheet" href="{{ theme_external_stylesheet() }}">
    <link rel="stylesheet" href="{{ versioned_asset(path='/static/css/base.css') }}">

    <!-- Theme variables (generated or static) -->
    <style id="theme-variables">
        {{ theme_css() | safe }}
    </style>

    <!-- Theme-specific overrides -->
    {% if theme.has_overrides %}
    <link rel="stylesheet" href="{{ versioned_asset(path='/static/themes/' ~ theme.id ~ '/overrides.css') }}">
    {% endif %}
</head>
```

### Phase 4: User Preference Persistence

**Database migration:**
```sql
ALTER TABLE users ADD COLUMN theme_preference TEXT DEFAULT 'auto';
```

**New routes:**
- `GET /api/user/theme` - Get current theme preference
- `PUT /api/user/theme` - Set theme preference
- `GET /api/themes` - List available themes

**Frontend:**
- Theme picker component in account settings
- LocalStorage for unauthenticated users
- `prefers-color-scheme` media query fallback

### Phase 5: Theme Picker UI

**New files:**
- `static/js/theme-switcher.js`
- `templates/components/theme-picker.html`

**Features:**
- Visual theme preview cards
- Live preview on hover
- Respects system preference when set to "auto"
- Smooth transition between themes

```javascript
// static/js/theme-switcher.js
class ThemeSwitcher {
    constructor() {
        this.storageKey = 'theme-preference';
        this.init();
    }

    init() {
        // Load saved preference or default to 'auto'
        const saved = localStorage.getItem(this.storageKey) || 'auto';
        this.apply(saved);

        // Listen for system preference changes
        window.matchMedia('(prefers-color-scheme: dark)')
            .addEventListener('change', () => this.apply('auto'));
    }

    apply(themeId) {
        if (themeId === 'auto') {
            const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
            document.documentElement.dataset.theme = prefersDark ? 'dark' : 'light';
        } else {
            document.documentElement.dataset.theme = themeId;
        }
    }

    async setTheme(themeId) {
        localStorage.setItem(this.storageKey, themeId);
        this.apply(themeId);

        // Sync to server if authenticated
        try {
            await fetch('/api/user/theme', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ theme: themeId })
            });
        } catch (e) {
            // Graceful fallback - localStorage still works
        }
    }
}
```

---

## CSS File Structure (Final)

```
static/
└── css/
    ├── base.css              # Theme-agnostic base styles (refactored from styles.css)
    ├── components.css        # Optional: component library
    └── utilities.css         # Optional: utility classes

themes/
├── _shared/
│   └── transitions.css       # Smooth theme switching
├── default/
│   ├── theme.toml
│   ├── variables.css
│   └── overrides.css
├── dark/
│   ├── theme.toml
│   ├── variables.css
│   └── overrides.css
└── high-contrast/
    ├── theme.toml
    └── variables.css
```

---

## Modern CSS Features Utilized

| Feature | Browser Support | Purpose |
|---------|----------------|---------|
| `@layer` | 99%+ | Cascade control, predictable specificity |
| CSS Custom Properties | 98%+ | Dynamic theming, runtime changes |
| `color-scheme` | 95%+ | Native dark mode hints |
| `light-dark()` | ~80% | Simplified light/dark values |
| `color-mix()` | ~90% | Dynamic color variations |
| `@container` | ~90% | Component-responsive styling |
| `oklch()` | ~85% | Perceptually uniform color mixing |

All modern features include fallbacks for older browsers.

---

## Migration Path

1. **Backwards Compatible**: Existing `EXTERNAL_STYLESHEET` and `OVERRIDE_STYLESHEET` secrets continue to work
2. **Opt-in**: Themes directory is optional - system falls back to legacy config
3. **Gradual**: Can migrate one component at a time to use theme variables

---

## Example Themes

### Dark Theme (`themes/dark/theme.toml`)

```toml
[theme]
id = "dark"
name = "Crusty Dark"
description = "A dark theme for late-night coding sessions"
version = "1.0.0"

[theme.supports]
color_scheme = "dark"

[theme.colors]
primary = "#ff8f5a"
secondary = "#4a6785"
accent = "#ff6b6b"

bg-body = "#0d1117"
bg-surface = "#161b22"
bg-elevated = "#21262d"
bg-gradient-start = "#1a1a2e"
bg-gradient-end = "#16213e"

text-primary = "#f0f6fc"
text-secondary = "#8b949e"
text-muted = "#6e7681"
text-on-primary = "#ffffff"

border-color = "#30363d"
shadow = "0 2px 15px rgba(0, 0, 0, 0.3)"
shadow-hover = "0 5px 25px rgba(0, 0, 0, 0.4)"
```

### High Contrast Theme (`themes/high-contrast/theme.toml`)

```toml
[theme]
id = "high-contrast"
name = "High Contrast"
description = "Maximum readability for accessibility"
version = "1.0.0"

[theme.supports]
color_scheme = "light"
high_contrast = true

[theme.colors]
primary = "#0066cc"
secondary = "#000000"
accent = "#cc0000"

bg-body = "#ffffff"
bg-surface = "#ffffff"
text-primary = "#000000"
text-secondary = "#000000"

border-color = "#000000"
border-radius = "0px"
shadow = "none"
```

---

## Success Criteria

- [ ] Themes loadable from `themes/` directory via `theme.toml`
- [ ] At least 3 themes: default (light), dark, high-contrast
- [ ] Theme switching works without page reload (CSS custom properties)
- [ ] User preferences persist (DB for authenticated, localStorage otherwise)
- [ ] System preference (`prefers-color-scheme`) respected when set to "auto"
- [ ] All modern CSS features have fallbacks
- [ ] No visual regressions from current design
- [ ] Theme picker UI in account settings

---

## Files to Create/Modify Summary

### New Files
- `themes/default/theme.toml`
- `themes/default/variables.css`
- `themes/dark/theme.toml`
- `themes/dark/variables.css`
- `themes/high-contrast/theme.toml`
- `themes/high-contrast/variables.css`
- `src/lib/theme.rs`
- `static/js/theme-switcher.js`
- `static/css/base.css` (refactored from styles.css)

### Modified Files
- `src/lib/lib.rs` - Export theme module
- `src/lib/state.rs` - Add themes to AppState, new Tera functions
- `src/lib/config.rs` - Theme configuration options
- `templates/base.html` - Dynamic theme loading
- `templates/account/settings.html` - Theme picker UI
- `Shuttle.toml` - Include themes directory
- `static/css/styles.css` - Refactor to use theme variables

### Database Migration
- Add `theme_preference` column to `users` table
