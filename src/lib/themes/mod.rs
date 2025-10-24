// src/lib/themes/mod.rs

mod loader;

pub use loader::ThemeLoader;

use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration for a theme loaded from theme.toml
#[derive(Debug, Clone, Deserialize)]
pub struct ThemeConfig {
    pub metadata: ThemeMetadata,
    #[serde(default)]
    pub settings: HashMap<String, toml::Value>,
    #[serde(default)]
    pub overrides: ThemeOverrides,
}

/// Metadata about a theme
#[derive(Debug, Clone, Deserialize)]
pub struct ThemeMetadata {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
}

/// Theme overrides and inheritance configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ThemeOverrides {
    /// Optional parent theme to inherit from
    pub parent: Option<String>,
}

/// Represents a fully loaded theme with all paths resolved
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub path: PathBuf,
    pub config: ThemeConfig,
    /// Template paths ordered by priority: custom -> parent -> default
    pub template_paths: Vec<PathBuf>,
    /// Static asset paths for serving files
    pub static_paths: Vec<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_config_parsing_minimal() {
        let toml_content = r#"
            [metadata]
            name = "Test Theme"
            version = "1.0.0"
        "#;

        let config: ThemeConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.metadata.name, "Test Theme");
        assert_eq!(config.metadata.version, "1.0.0");
        assert!(config.metadata.author.is_none());
        assert!(config.metadata.description.is_none());
        assert!(config.settings.is_empty());
        assert!(config.overrides.parent.is_none());
    }

    #[test]
    fn test_theme_config_parsing_complete() {
        let toml_content = r#"
            [metadata]
            name = "Complete Theme"
            version = "2.0.0"
            author = "Test Author"
            description = "A complete test theme"

            [settings]
            brand_name = "My Brand"
            show_analytics = true
            footer_text = "Built with Rust"

            [overrides]
            parent = "default"
        "#;

        let config: ThemeConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.metadata.name, "Complete Theme");
        assert_eq!(config.metadata.version, "2.0.0");
        assert_eq!(config.metadata.author, Some("Test Author".to_string()));
        assert_eq!(
            config.metadata.description,
            Some("A complete test theme".to_string())
        );
        assert_eq!(config.settings.len(), 3);
        assert!(config.settings.contains_key("brand_name"));
        assert_eq!(config.overrides.parent, Some("default".to_string()));
    }

    #[test]
    fn test_theme_config_settings_types() {
        let toml_content = r#"
            [metadata]
            name = "Settings Test"
            version = "1.0.0"

            [settings]
            string_value = "text"
            number_value = 42
            bool_value = true
        "#;

        let config: ThemeConfig = toml::from_str(toml_content).unwrap();

        // Verify different setting types
        assert!(config.settings.contains_key("string_value"));
        assert!(config.settings.contains_key("number_value"));
        assert!(config.settings.contains_key("bool_value"));

        // Check string value
        if let Some(toml::Value::String(s)) = config.settings.get("string_value") {
            assert_eq!(s, "text");
        } else {
            panic!("string_value should be a string");
        }

        // Check number value
        if let Some(toml::Value::Integer(n)) = config.settings.get("number_value") {
            assert_eq!(*n, 42);
        } else {
            panic!("number_value should be an integer");
        }

        // Check bool value
        if let Some(toml::Value::Boolean(b)) = config.settings.get("bool_value") {
            assert!(*b);
        } else {
            panic!("bool_value should be a boolean");
        }
    }

    #[test]
    fn test_theme_config_invalid_toml() {
        let invalid_toml = r#"
            [metadata
            name = "Invalid"
        "#;

        let result: Result<ThemeConfig, _> = toml::from_str(invalid_toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_theme_config_missing_required_fields() {
        let toml_content = r#"
            [metadata]
            name = "Incomplete"
            # missing version field
        "#;

        let result: Result<ThemeConfig, _> = toml::from_str(toml_content);
        assert!(result.is_err());
    }
}
