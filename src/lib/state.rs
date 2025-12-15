// src/lib/state.rs

// dependencies
use crate::auth::Keys;
use crate::email::EmailService;
use crate::repositories::{
    ApiKeyRepository, ArticleRepository, CategoryRepository, CommentRepository,
    LibSqlApiKeyRepository, LibSqlArticleRepository, LibSqlCategoryRepository,
    LibSqlCommentRepository, LibSqlMediaRepository, LibSqlNewsletterRepository,
    LibSqlTagRepository, LibSqlTokenRepository, LibSqlUserRepository, MediaRepository,
    NewsletterRepository, TagRepository, TokenRepository, UserRepository,
};
use crate::storage::StorageBackend;
use crate::theme::{ThemeListItem, ThemeRegistry};
use crate::{ApiError, AppConfig, DatabaseConnection};
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tera::{Tera, Value};
use tokio::sync::RwLock;

// type declarations
pub type TagCache = Arc<RwLock<Option<CachedTags>>>;

// initialized a static variable to hold the compiled templates
static COMPILED_TEMPLATES: OnceCell<Tera> = OnceCell::new();

// static variable to hold the theme registry
static THEME_REGISTRY: OnceCell<ThemeRegistry> = OnceCell::new();

#[derive(Clone)]
pub struct CachedTags {
    pub tags: Vec<String>,
    pub cached_at: Instant,
}

// struct type to represent the application state
#[derive(Clone)]
pub struct AppState {
    pub templates: &'static Tera,
    pub db: DatabaseConnection,
    pub jwt_keys: Keys,
    pub storage: Arc<dyn StorageBackend>,
    pub cached_tags: TagCache,
    pub email: EmailService,
    pub allowed_origins: Vec<String>,
    pub themes: &'static ThemeRegistry,
    // Repositories
    pub users: Arc<dyn UserRepository>,
    pub articles: Arc<dyn ArticleRepository>,
    pub tags: Arc<dyn TagRepository>,
    pub categories: Arc<dyn CategoryRepository>,
    pub comments: Arc<dyn CommentRepository>,
    pub newsletters: Arc<dyn NewsletterRepository>,
    pub api_keys: Arc<dyn ApiKeyRepository>,
    pub media: Arc<dyn MediaRepository>,
    pub tokens: Arc<dyn TokenRepository>,
}

// simplified setup function
fn setup_templates(config: &AppConfig) -> Result<&'static Tera, tera::Error> {
    load_templates(&config.templates_dir, config)
}

// Load themes from the themes directory
fn setup_themes() -> &'static ThemeRegistry {
    THEME_REGISTRY.get_or_init(|| {
        match ThemeRegistry::load_from_directory("themes") {
            Ok(registry) => {
                tracing::info!("Loaded {} themes", registry.len());
                registry
            }
            Err(e) => {
                tracing::warn!("Failed to load themes: {}, using empty registry", e);
                ThemeRegistry::new()
            }
        }
    })
}

// utility function to load and compile the templates once, saving them in static memory
fn load_templates(templates_dir: &str, config: &AppConfig) -> Result<&'static Tera, tera::Error> {
    COMPILED_TEMPLATES.get_or_try_init(|| {
        let mut tera = Tera::new(templates_dir)?;

        let external = config.external_stylesheet.clone();
        let override_css = config.override_stylesheet.clone();

        // Legacy theme_stylesheet function (for backwards compatibility)
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

        // Register cache-busting function for static assets
        let app_version = config.app_version.clone();
        tera.register_function(
            "versioned_asset",
            move |args: &HashMap<String, Value>| -> tera::Result<Value> {
                use tera::from_value;

                let path = args
                    .get("path")
                    .and_then(|v| from_value::<String>(v.clone()).ok())
                    .ok_or_else(|| {
                        tera::Error::msg("versioned_asset requires a 'path' argument")
                    })?;

                let versioned_url = format!("{}?v={}", path, app_version);
                Ok(Value::String(versioned_url))
            },
        );

        // Register theme_css function to generate CSS variables from theme.toml
        tera.register_function(
            "theme_css",
            |args: &HashMap<String, Value>| -> tera::Result<Value> {
                use tera::from_value;

                let theme_id = args
                    .get("theme")
                    .and_then(|v| from_value::<String>(v.clone()).ok())
                    .unwrap_or_else(|| "default".to_string());

                let registry = THEME_REGISTRY.get();
                let css = registry
                    .and_then(|r| r.get(&theme_id))
                    .map(|theme| theme.to_css())
                    .unwrap_or_else(|| {
                        // Fallback to default theme or empty CSS
                        registry
                            .and_then(|r| r.default_theme())
                            .map(|t| t.to_css())
                            .unwrap_or_default()
                    });

                Ok(Value::String(css))
            },
        );

        // Register available_themes function to get list of themes
        tera.register_function(
            "available_themes",
            |_args: &HashMap<String, Value>| -> tera::Result<Value> {
                let registry = THEME_REGISTRY.get();
                let themes: Vec<ThemeListItem> = registry
                    .map(|r| r.to_theme_list())
                    .unwrap_or_default();

                serde_json::to_value(themes)
                    .map(|v| tera::to_value(v).unwrap_or(Value::Array(vec![])))
                    .map_err(|e| tera::Error::msg(format!("Failed to serialize themes: {}", e)))
            },
        );

        // Register theme_color_scheme function to get a theme's color scheme
        tera.register_function(
            "theme_color_scheme",
            |args: &HashMap<String, Value>| -> tera::Result<Value> {
                use tera::from_value;

                let theme_id = args
                    .get("theme")
                    .and_then(|v| from_value::<String>(v.clone()).ok())
                    .unwrap_or_else(|| "default".to_string());

                let registry = THEME_REGISTRY.get();
                let scheme = registry
                    .and_then(|r| r.get(&theme_id))
                    .map(|theme| theme.color_scheme().to_string())
                    .unwrap_or_else(|| "light".to_string());

                Ok(Value::String(scheme))
            },
        );

        // Register all_themes_css function to generate CSS for all themes
        // This enables client-side theme switching without additional server requests
        tera.register_function(
            "all_themes_css",
            |_args: &HashMap<String, Value>| -> tera::Result<Value> {
                let registry = THEME_REGISTRY.get();
                let css = registry
                    .map(|r| r.all_themes_css())
                    .unwrap_or_default();

                Ok(Value::String(css))
            },
        );

        Ok(tera)
    })
}

// methods to build the configuration
impl AppState {
    // Create a new application state instance
    pub fn new(
        db: DatabaseConnection,
        storage: Arc<dyn StorageBackend>,
        config: AppConfig,
    ) -> Result<Self, ApiError> {
        // Load themes first (so they're available during template setup)
        let themes = setup_themes();
        let templates = setup_templates(&config)?;
        let jwt_keys = Keys::from_config(&config);
        let email = EmailService::from_config(&config.email);
        let allowed_origins = config.allowed_origins;

        // Initialize repositories
        let users: Arc<dyn UserRepository> =
            Arc::new(LibSqlUserRepository::new(db.clone()));
        let articles: Arc<dyn ArticleRepository> =
            Arc::new(LibSqlArticleRepository::new(db.clone()));
        let tags: Arc<dyn TagRepository> =
            Arc::new(LibSqlTagRepository::new(db.clone()));
        let categories: Arc<dyn CategoryRepository> =
            Arc::new(LibSqlCategoryRepository::new(db.clone()));
        let comments: Arc<dyn CommentRepository> =
            Arc::new(LibSqlCommentRepository::new(db.clone()));
        let newsletters: Arc<dyn NewsletterRepository> =
            Arc::new(LibSqlNewsletterRepository::new(db.clone()));
        let api_keys: Arc<dyn ApiKeyRepository> =
            Arc::new(LibSqlApiKeyRepository::new(db.clone()));
        let media: Arc<dyn MediaRepository> =
            Arc::new(LibSqlMediaRepository::new(db.clone()));
        let tokens: Arc<dyn TokenRepository> =
            Arc::new(LibSqlTokenRepository::new(db.clone()));

        Ok(Self {
            templates,
            db,
            jwt_keys,
            storage,
            cached_tags: Arc::new(RwLock::new(None)),
            email,
            allowed_origins,
            themes,
            users,
            articles,
            tags,
            categories,
            comments,
            newsletters,
            api_keys,
            media,
            tokens,
        })
    }

    /// Get the theme registry
    pub fn theme_registry(&self) -> &ThemeRegistry {
        self.themes
    }

    /// Get available themes as a list
    pub fn available_themes(&self) -> Vec<ThemeListItem> {
        self.themes.to_theme_list()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::email::EmailConfig;
    use std::fs;
    use tempfile::TempDir;

    fn test_config() -> AppConfig {
        AppConfig {
            jwt_secret: "test-secret-key".to_string(),
            templates_dir: "templates/**/*".to_string(),
            external_stylesheet: "https://cdn.example.com/bootstrap.css".to_string(),
            override_stylesheet: "/static/overrides.css".to_string(),
            app_version: "2.9.0".to_string(),
            allowed_origins: vec!["http://localhost:8000".to_string()],
            email: EmailConfig::default(),
        }
    }

    fn setup_test_templates_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let templates_path = temp_dir.path().join("templates");
        fs::create_dir(&templates_path).unwrap();

        // Create a minimal valid template
        fs::write(
            templates_path.join("test.html"),
            "<html><body>{{ content }}</body></html>",
        )
        .unwrap();

        temp_dir
    }

    #[test]
    fn test_setup_templates_succeeds_with_valid_config() {
        let config = test_config();
        let _temp_dir = setup_test_templates_dir();

        let result = setup_templates(&config);

        assert!(
            result.is_ok(),
            "setup_templates should succeed with valid config"
        );
    }

    #[test]
    fn test_setup_templates_with_empty_stylesheets() {
        let config = AppConfig {
            jwt_secret: "test-secret".to_string(),
            templates_dir: "templates/**/*".to_string(),
            external_stylesheet: "".to_string(),
            override_stylesheet: "".to_string(),
            app_version: "2.9.0".to_string(),
            allowed_origins: vec!["http://localhost:8000".to_string()],
            email: EmailConfig::default(),
        };
        let _temp_dir = setup_test_templates_dir();

        let result = setup_templates(&config);

        assert!(
            result.is_ok(),
            "setup_templates should handle empty stylesheets"
        );
    }

    #[test]
    fn test_setup_templates_with_special_characters() {
        let config = AppConfig {
            jwt_secret: "test-secret".to_string(),
            templates_dir: "templates/**/*".to_string(),
            external_stylesheet: "https://example.com/style.css?v=1.0&theme=dark".to_string(),
            override_stylesheet: "/static/override-theme.css".to_string(),
            app_version: "2.9.0".to_string(),
            allowed_origins: vec!["http://localhost:8000".to_string()],
            email: EmailConfig::default(),
        };
        let _temp_dir = setup_test_templates_dir();

        let result = setup_templates(&config);

        assert!(
            result.is_ok(),
            "setup_templates should handle URLs with special characters"
        );
    }
}
