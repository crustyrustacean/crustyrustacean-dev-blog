// src/lib/themes/loader.rs

use super::{Theme, ThemeConfig};
use anyhow::{Context, Result, bail};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// Loads themes from a directory, supporting parent theme inheritance
pub struct ThemeLoader {
    themes_dir: PathBuf,
}

impl ThemeLoader {
    /// Create a new ThemeLoader for the given themes directory
    pub fn new(themes_dir: PathBuf) -> Self {
        Self { themes_dir }
    }

    /// Load a theme by name, resolving parent chain and preventing circular dependencies
    pub fn load(&self, theme_name: &str) -> Result<Theme> {
        let mut visited = HashSet::new();
        self.load_with_visited(theme_name, &mut visited)
    }

    /// Internal method that tracks visited themes to prevent circular dependencies
    fn load_with_visited(
        &self,
        theme_name: &str,
        visited: &mut HashSet<String>,
    ) -> Result<Theme> {
        // Check for circular dependency
        if visited.contains(theme_name) {
            bail!(
                "Circular theme dependency detected: {} -> {}",
                visited.iter().cloned().collect::<Vec<_>>().join(" -> "),
                theme_name
            );
        }

        visited.insert(theme_name.to_string());

        let theme_path = self.themes_dir.join(theme_name);

        // Verify theme directory exists
        if !theme_path.exists() {
            bail!("Theme directory not found: {}", theme_path.display());
        }

        // Load theme.toml
        let config_path = theme_path.join("theme.toml");
        let config_str = fs::read_to_string(&config_path).context(format!(
            "Failed to read theme config for '{}' at {}",
            theme_name,
            config_path.display()
        ))?;

        let config: ThemeConfig = toml::from_str(&config_str)
            .context(format!("Failed to parse theme.toml for '{}'", theme_name))?;

        // Build template search paths (most specific to least specific)
        let mut template_paths = vec![];
        let mut static_paths = vec![];

        // Add current theme's paths
        let theme_templates = theme_path.join("templates");
        let theme_static = theme_path.join("static");

        if theme_templates.exists() {
            template_paths.push(theme_templates);
        }

        if theme_static.exists() {
            static_paths.push(theme_static);
        }

        // If theme has a parent, load it recursively
        if let Some(parent_name) = &config.overrides.parent {
            tracing::debug!(
                "Loading parent theme '{}' for '{}'",
                parent_name,
                theme_name
            );

            let parent = self.load_with_visited(parent_name, visited)?;
            template_paths.extend(parent.template_paths);
            static_paths.extend(parent.static_paths);
        }

        tracing::info!(
            "Loaded theme '{}' with {} template paths and {} static paths",
            theme_name,
            template_paths.len(),
            static_paths.len()
        );

        Ok(Theme {
            name: theme_name.to_string(),
            path: theme_path,
            config,
            template_paths,
            static_paths,
        })
    }

    /// List all available themes in the themes directory
    pub fn list_themes(&self) -> Result<Vec<String>> {
        let mut themes = Vec::new();

        // Check if themes directory exists
        if !self.themes_dir.exists() {
            tracing::warn!(
                "Themes directory does not exist: {}",
                self.themes_dir.display()
            );
            return Ok(themes);
        }

        for entry in fs::read_dir(&self.themes_dir)
            .context(format!("Failed to read themes directory: {}", self.themes_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();

            // Check if this is a directory with a theme.toml file
            if path.is_dir() && path.join("theme.toml").exists() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    themes.push(name.to_string());
                }
            }
        }

        themes.sort();
        Ok(themes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn create_test_theme(
        base_dir: &Path,
        name: &str,
        parent: Option<&str>,
    ) -> PathBuf {
        let theme_dir = base_dir.join(name);
        fs::create_dir_all(&theme_dir).unwrap();

        let templates_dir = theme_dir.join("templates");
        fs::create_dir_all(&templates_dir).unwrap();

        let static_dir = theme_dir.join("static");
        fs::create_dir_all(&static_dir).unwrap();

        let parent_line = if let Some(p) = parent {
            format!("\n[overrides]\nparent = \"{}\"", p)
        } else {
            String::new()
        };

        let toml_content = format!(
            r#"
[metadata]
name = "{}"
version = "1.0.0"
author = "Test Author"
description = "Test theme"
{}"#,
            name, parent_line
        );

        fs::write(theme_dir.join("theme.toml"), toml_content).unwrap();
        theme_dir
    }

    #[test]
    fn test_load_simple_theme() {
        let temp_dir = TempDir::new().unwrap();
        create_test_theme(temp_dir.path(), "simple", None);

        let loader = ThemeLoader::new(temp_dir.path().to_path_buf());
        let theme = loader.load("simple").unwrap();

        assert_eq!(theme.name, "simple");
        assert_eq!(theme.config.metadata.name, "simple");
        assert_eq!(theme.template_paths.len(), 1);
        assert_eq!(theme.static_paths.len(), 1);
    }

    #[test]
    fn test_load_theme_with_parent() {
        let temp_dir = TempDir::new().unwrap();
        create_test_theme(temp_dir.path(), "base", None);
        create_test_theme(temp_dir.path(), "child", Some("base"));

        let loader = ThemeLoader::new(temp_dir.path().to_path_buf());
        let theme = loader.load("child").unwrap();

        assert_eq!(theme.name, "child");
        // Should have 2 template paths: child's and parent's
        assert_eq!(theme.template_paths.len(), 2);
        assert_eq!(theme.static_paths.len(), 2);

        // Verify order: child first, then parent
        assert!(theme.template_paths[0].ends_with("child/templates"));
        assert!(theme.template_paths[1].ends_with("base/templates"));
    }

    #[test]
    fn test_load_theme_with_grandparent() {
        let temp_dir = TempDir::new().unwrap();
        create_test_theme(temp_dir.path(), "grandparent", None);
        create_test_theme(temp_dir.path(), "parent", Some("grandparent"));
        create_test_theme(temp_dir.path(), "child", Some("parent"));

        let loader = ThemeLoader::new(temp_dir.path().to_path_buf());
        let theme = loader.load("child").unwrap();

        assert_eq!(theme.name, "child");
        // Should have 3 template paths
        assert_eq!(theme.template_paths.len(), 3);
        assert_eq!(theme.static_paths.len(), 3);

        // Verify order: child -> parent -> grandparent
        assert!(theme.template_paths[0].ends_with("child/templates"));
        assert!(theme.template_paths[1].ends_with("parent/templates"));
        assert!(theme.template_paths[2].ends_with("grandparent/templates"));
    }

    #[test]
    fn test_circular_dependency_detection() {
        let temp_dir = TempDir::new().unwrap();
        create_test_theme(temp_dir.path(), "theme-a", Some("theme-b"));
        create_test_theme(temp_dir.path(), "theme-b", Some("theme-a"));

        let loader = ThemeLoader::new(temp_dir.path().to_path_buf());
        let result = loader.load("theme-a");

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Circular theme dependency"));
    }

    #[test]
    fn test_self_referencing_theme() {
        let temp_dir = TempDir::new().unwrap();
        create_test_theme(temp_dir.path(), "self-ref", Some("self-ref"));

        let loader = ThemeLoader::new(temp_dir.path().to_path_buf());
        let result = loader.load("self-ref");

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Circular theme dependency"));
    }

    #[test]
    fn test_nonexistent_theme() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ThemeLoader::new(temp_dir.path().to_path_buf());
        let result = loader.load("nonexistent");

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("not found"));
    }

    #[test]
    fn test_nonexistent_parent_theme() {
        let temp_dir = TempDir::new().unwrap();
        create_test_theme(temp_dir.path(), "child", Some("nonexistent-parent"));

        let loader = ThemeLoader::new(temp_dir.path().to_path_buf());
        let result = loader.load("child");

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("not found"));
    }

    #[test]
    fn test_list_themes() {
        let temp_dir = TempDir::new().unwrap();
        create_test_theme(temp_dir.path(), "theme-a", None);
        create_test_theme(temp_dir.path(), "theme-b", None);
        create_test_theme(temp_dir.path(), "theme-c", None);

        // Create a directory without theme.toml (should be ignored)
        let non_theme_dir = temp_dir.path().join("not-a-theme");
        fs::create_dir(&non_theme_dir).unwrap();

        let loader = ThemeLoader::new(temp_dir.path().to_path_buf());
        let themes = loader.list_themes().unwrap();

        assert_eq!(themes.len(), 3);
        assert!(themes.contains(&"theme-a".to_string()));
        assert!(themes.contains(&"theme-b".to_string()));
        assert!(themes.contains(&"theme-c".to_string()));
        assert!(!themes.contains(&"not-a-theme".to_string()));
    }

    #[test]
    fn test_list_themes_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ThemeLoader::new(temp_dir.path().to_path_buf());
        let themes = loader.list_themes().unwrap();

        assert_eq!(themes.len(), 0);
    }

    #[test]
    fn test_list_themes_nonexistent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("nonexistent");
        let loader = ThemeLoader::new(nonexistent);
        let themes = loader.list_themes().unwrap();

        // Should return empty list, not error
        assert_eq!(themes.len(), 0);
    }

    #[test]
    fn test_theme_without_templates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let theme_dir = temp_dir.path().join("no-templates");
        fs::create_dir_all(&theme_dir).unwrap();

        let toml_content = r#"
[metadata]
name = "No Templates"
version = "1.0.0"
"#;
        fs::write(theme_dir.join("theme.toml"), toml_content).unwrap();

        let loader = ThemeLoader::new(temp_dir.path().to_path_buf());
        let theme = loader.load("no-templates").unwrap();

        // Should load successfully with empty template paths
        assert_eq!(theme.template_paths.len(), 0);
        assert_eq!(theme.static_paths.len(), 0);
    }
}
