// src/lib/theme.rs
//
// Theme configuration system for modular theming support.
// Loads theme.toml files from the themes/ directory and generates CSS variables.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Root configuration structure for theme.toml files
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThemeConfig {
    pub theme: ThemeMetadata,
}

/// Theme metadata and settings
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThemeMetadata {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub version: String,
    #[serde(default)]
    pub author: Option<String>,
    pub supports: ThemeSupports,
    pub colors: HashMap<String, String>,
    #[serde(default)]
    pub fonts: Option<ThemeFonts>,
    #[serde(default)]
    pub external: Option<ThemeExternal>,
    #[serde(default)]
    pub meta: Option<ThemeMeta>,
}

/// Theme support flags
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThemeSupports {
    pub color_scheme: String, // "light", "dark", or "auto"
    #[serde(default)]
    pub high_contrast: Option<bool>,
}

/// Font configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ThemeFonts {
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub heading: Option<String>,
    #[serde(default)]
    pub monospace: Option<String>,
}

/// External dependencies (CDN stylesheets, etc.)
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ThemeExternal {
    #[serde(default)]
    pub stylesheets: Option<Vec<String>>,
}

/// Theme metadata for UI display
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ThemeMeta {
    #[serde(default)]
    pub preview_colors: Option<Vec<String>>,
}

/// Error type for theme operations
#[derive(Debug)]
pub enum ThemeError {
    IoError(std::io::Error),
    ParseError(toml::de::Error),
    NotFound(String),
}

impl std::fmt::Display for ThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeError::IoError(e) => write!(f, "IO error: {}", e),
            ThemeError::ParseError(e) => write!(f, "Parse error: {}", e),
            ThemeError::NotFound(id) => write!(f, "Theme not found: {}", id),
        }
    }
}

impl std::error::Error for ThemeError {}

impl From<std::io::Error> for ThemeError {
    fn from(err: std::io::Error) -> Self {
        ThemeError::IoError(err)
    }
}

impl From<toml::de::Error> for ThemeError {
    fn from(err: toml::de::Error) -> Self {
        ThemeError::ParseError(err)
    }
}

impl ThemeConfig {
    /// Load a theme from a TOML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ThemeError> {
        let content = fs::read_to_string(path)?;
        let config: ThemeConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Generate CSS custom properties from theme colors
    pub fn to_css(&self) -> String {
        let mut css = String::from(":root {\n");

        // Add color variables
        for (key, value) in &self.theme.colors {
            css.push_str(&format!("    --theme-{}: {};\n", key, value));
        }

        // Add font variables if present
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

    /// Get the color scheme ("light", "dark", or "auto")
    pub fn color_scheme(&self) -> &str {
        &self.theme.supports.color_scheme
    }

    /// Check if this is a high contrast theme
    pub fn is_high_contrast(&self) -> bool {
        self.theme.supports.high_contrast.unwrap_or(false)
    }

    /// Get the primary external stylesheet URL
    pub fn external_stylesheet(&self) -> Option<&str> {
        self.theme
            .external
            .as_ref()
            .and_then(|ext| ext.stylesheets.as_ref())
            .and_then(|sheets| sheets.first())
            .map(|s| s.as_str())
    }

    /// Get preview colors for the theme picker UI
    pub fn preview_colors(&self) -> Vec<String> {
        self.theme
            .meta
            .as_ref()
            .and_then(|m| m.preview_colors.clone())
            .unwrap_or_else(|| {
                // Fallback to primary, secondary, and first background color
                vec![
                    self.theme.colors.get("primary").cloned().unwrap_or_default(),
                    self.theme
                        .colors
                        .get("secondary")
                        .cloned()
                        .unwrap_or_default(),
                    self.theme
                        .colors
                        .get("bg-body")
                        .cloned()
                        .unwrap_or_default(),
                ]
            })
    }
}

/// Manages a collection of available themes
#[derive(Debug, Clone)]
pub struct ThemeRegistry {
    themes: HashMap<String, ThemeConfig>,
    default_theme_id: String,
}

impl ThemeRegistry {
    /// Create a new theme registry
    pub fn new() -> Self {
        Self {
            themes: HashMap::new(),
            default_theme_id: "default".to_string(),
        }
    }

    /// Load all themes from a directory
    pub fn load_from_directory<P: AsRef<Path>>(themes_dir: P) -> Result<Self, ThemeError> {
        let mut registry = Self::new();
        let themes_path = themes_dir.as_ref();

        if !themes_path.exists() {
            tracing::warn!("Themes directory does not exist: {:?}", themes_path);
            return Ok(registry);
        }

        for entry in fs::read_dir(themes_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let theme_toml = path.join("theme.toml");
                if theme_toml.exists() {
                    match ThemeConfig::load(&theme_toml) {
                        Ok(config) => {
                            let theme_id = config.theme.id.clone();
                            tracing::info!("Loaded theme: {} ({})", config.theme.name, theme_id);
                            registry.themes.insert(theme_id, config);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to load theme from {:?}: {}", theme_toml, e);
                        }
                    }
                }
            }
        }

        // Ensure we have at least a default theme
        if !registry.themes.contains_key("default") && !registry.themes.is_empty() {
            // Use the first available theme as default
            if let Some(first_id) = registry.themes.keys().next().cloned() {
                registry.default_theme_id = first_id;
            }
        }

        Ok(registry)
    }

    /// Get a theme by ID
    pub fn get(&self, id: &str) -> Option<&ThemeConfig> {
        self.themes.get(id)
    }

    /// Get the default theme
    pub fn default_theme(&self) -> Option<&ThemeConfig> {
        self.themes.get(&self.default_theme_id)
    }

    /// Get all available themes
    pub fn all(&self) -> impl Iterator<Item = &ThemeConfig> {
        self.themes.values()
    }

    /// Get theme IDs
    pub fn theme_ids(&self) -> impl Iterator<Item = &String> {
        self.themes.keys()
    }

    /// Check if a theme exists
    pub fn contains(&self, id: &str) -> bool {
        self.themes.contains_key(id)
    }

    /// Get the number of loaded themes
    pub fn len(&self) -> usize {
        self.themes.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.themes.is_empty()
    }

    /// Generate a list of themes for API responses
    pub fn to_theme_list(&self) -> Vec<ThemeListItem> {
        self.themes
            .values()
            .map(|config| ThemeListItem {
                id: config.theme.id.clone(),
                name: config.theme.name.clone(),
                description: config.theme.description.clone(),
                color_scheme: config.theme.supports.color_scheme.clone(),
                high_contrast: config.is_high_contrast(),
                preview_colors: config.preview_colors(),
            })
            .collect()
    }
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Theme information for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeListItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color_scheme: String,
    pub high_contrast: bool,
    pub preview_colors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_theme_toml() -> String {
        r##"
[theme]
id = "test"
name = "Test Theme"
description = "A test theme"
version = "1.0.0"
author = "Test Author"

[theme.supports]
color_scheme = "light"
high_contrast = false

[theme.colors]
primary = "#ff6b35"
secondary = "#2c3e50"
bg-body = "#ffffff"
text-primary = "#000000"

[theme.fonts]
body = "Inter, sans-serif"
heading = "Inter, sans-serif"
monospace = "Monaco, monospace"

[theme.external]
stylesheets = ["https://example.com/bootstrap.css"]

[theme.meta]
preview_colors = ["#ff6b35", "#2c3e50", "#ffffff"]
"##
        .to_string()
    }

    #[test]
    fn test_parse_theme_toml() {
        let toml_content = create_test_theme_toml();
        let config: ThemeConfig = toml::from_str(&toml_content).unwrap();

        assert_eq!(config.theme.id, "test");
        assert_eq!(config.theme.name, "Test Theme");
        assert_eq!(config.theme.supports.color_scheme, "light");
        let expected_primary = "#ff6b35".to_string();
        assert_eq!(config.theme.colors.get("primary"), Some(&expected_primary));
    }

    #[test]
    fn test_theme_to_css() {
        let toml_content = create_test_theme_toml();
        let config: ThemeConfig = toml::from_str(&toml_content).unwrap();
        let css = config.to_css();

        assert!(css.contains(":root {"));
        assert!(css.contains("--theme-primary:"));
        assert!(css.contains("--theme-font-body:"));
    }

    #[test]
    fn test_theme_registry_load() {
        let temp_dir = TempDir::new().unwrap();
        let theme_dir = temp_dir.path().join("test");
        fs::create_dir(&theme_dir).unwrap();
        let toml_path = theme_dir.join("theme.toml");
        fs::write(&toml_path, create_test_theme_toml()).unwrap();

        let registry = ThemeRegistry::load_from_directory(temp_dir.path()).unwrap();

        assert!(registry.contains("test"));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_theme_list_generation() {
        let toml_content = create_test_theme_toml();
        let config: ThemeConfig = toml::from_str(&toml_content).unwrap();

        let mut registry = ThemeRegistry::new();
        registry.themes.insert("test".to_string(), config);

        let list = registry.to_theme_list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "test");
        assert_eq!(list[0].name, "Test Theme");
    }

    #[test]
    fn test_color_scheme_accessor() {
        let toml_content = create_test_theme_toml();
        let config: ThemeConfig = toml::from_str(&toml_content).unwrap();

        assert_eq!(config.color_scheme(), "light");
        assert!(!config.is_high_contrast());
    }

    #[test]
    fn test_external_stylesheet() {
        let toml_content = create_test_theme_toml();
        let config: ThemeConfig = toml::from_str(&toml_content).unwrap();

        assert_eq!(
            config.external_stylesheet(),
            Some("https://example.com/bootstrap.css")
        );
    }
}
