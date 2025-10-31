// tests/api/helpers.rs

// types and functions used across all integration tests

// dependencies
use anyhow::{Context, Result, anyhow};
use crustyrustacean_dev_blog_lib::config::AppConfig;
use crustyrustacean_dev_blog_lib::database::DatabaseConnection;
use crustyrustacean_dev_blog_lib::startup::App;
use crustyrustacean_dev_blog_lib::state::AppState;
use crustyrustacean_dev_blog_lib::storage::{OpenDalStorage, StorageBackend};
use crustyrustacean_dev_blog_lib::telemetry::{get_subscriber, init_subscriber};
use opendal::Operator;
use reqwest::Client;
use shuttle_common::secrets::Secret;
use shuttle_runtime::SecretStore;
use std::collections::BTreeMap;
use std::env::var;
use std::fs;
use std::io::{sink, stdout};
use std::sync::{Arc, LazyLock};
use tokio::net::TcpListener;
use toml::Value;

// static constant which creates one instance of tracing
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, sink);
        init_subscriber(subscriber);
    }
});

// function to create R2 storage operator with dev bucket configuration
fn create_r2_storage() -> Result<Operator, Box<dyn std::error::Error>> {
    // Load environment variables from Secrets.dev.toml
    let _ = dotenvy::from_filename("Secrets.dev.toml");

    let builder = opendal::services::S3::default()
        .region(&var("DEFAULT_REGION").unwrap_or_else(|_| "auto".to_string()))
        .bucket(&var("BUCKET").unwrap_or_else(|_| "photo-bucket-dev".to_string()))
        .endpoint(&var("ENDPOINT").unwrap_or_else(|_| {
            "https://9f8f9b268ba5c7d97ad1adfabf962f30.r2.cloudflarestorage.com".to_string()
        }))
        .access_key_id(
            &var("ACCESS_KEY_ID")
                .unwrap_or_else(|_| "9063762cd3059516a2af26a159c7a5ca".to_string()),
        )
        .secret_access_key(&var("SECRET_ACCESS_KEY").unwrap_or_else(|_| {
            "2c91ace42b2df0dd4d8c3d7dc36e8d59576e90dc55b8153d64da8086f87e4f16".to_string()
        }));

    let op = Operator::new(builder)?.finish();
    Ok(op)
}

// Load Shuttle secrets for tests from Secrets.dev.toml (preferred) or Secrets.toml.
fn load_test_secret_store() -> Result<SecretStore> {
    let path = if fs::metadata("Secrets.dev.toml").is_ok() {
        "Secrets.dev.toml"
    } else if fs::metadata("Secrets.toml").is_ok() {
        "Secrets.toml"
    } else {
        return Err(anyhow!(
            "Neither Secrets.dev.toml nor Secrets.toml found in project root"
        ));
    };

    let txt = fs::read_to_string(path).with_context(|| format!("Reading {}", path))?;
    let val: Value = toml::from_str(&txt).with_context(|| format!("Parsing {}", path))?;
    let table = val
        .as_table()
        .ok_or_else(|| anyhow!("Root of {} must be a TOML table", path))?;

    let mut map: BTreeMap<String, Secret<String>> = BTreeMap::new();
    for (k, v) in table {
        let s = v
            .as_str()
            .ok_or_else(|| anyhow!("Secret {k} must be a string in {}", path))?;
        map.insert(k.clone(), Secret::new(s.to_owned()));
    }

    Ok(SecretStore::new(map))
}

// struct type which models a test application
#[allow(dead_code)]
pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub client: Client,
    pub db: DatabaseConnection,
}

// helper function which builds and returns a test application
pub async fn spawn_app() -> TestApp {
    // ensure that the tracing is only initialized once
    LazyLock::force(&TRACING);

    // set up the configuration with test JWT secret
    let secrets = load_test_secret_store().expect("Failed to load Shuttle secrets for tests.");
    let app_config =
        AppConfig::try_from(&secrets).expect("Failed to build AppConfig from Shuttle secrets.");

    // create a test database connection (local SQLite for testing)
    use std::env::temp_dir;
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut temp_db_path = temp_dir();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    temp_db_path.push(format!("test_db_{}_{}.db", std::process::id(), timestamp));
    let test_db = libsql::Builder::new_local(&temp_db_path)
        .build()
        .await
        .expect("Failed to create test database");
    let db_connection = DatabaseConnection {
        db: std::sync::Arc::new(test_db),
    };

    // run migrations for test database
    db_connection
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    // Create R2 storage operator for testing with dev bucket
    let operator = create_r2_storage().expect("Failed to create R2 storage operator for testing");
    let storage: Arc<dyn StorageBackend> = Arc::new(OpenDalStorage::new(operator));

    // set up the app state
    let app_state = AppState::new(db_connection.clone(), storage, &app_config)
        .expect("Unable to build the Tera templates");

    // create the test application
    let application = App::new(app_config, app_state);

    // create a listener
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind port.");

    // get the address and port from the listener
    let addr = listener
        .local_addr()
        .expect("Unable to obtain the address of the listener.");
    let port = addr.port();

    // spawn the application
    tokio::spawn(application.run_until_stopped(listener));

    // build a client to make requests
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        port,
        client,
        db: db_connection,
    }
}

// ============================================================================
// Test Builder Traits
// ============================================================================

use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;

/// Trait for building test users with simplified registration
#[async_trait]
pub trait TestUserBuilder {
    /// Register a user and return their authentication token
    async fn register_user(&self, username: &str, email: &str, password: &str) -> String;

    /// Register a user with default email/password pattern
    async fn register_user_default(&self, username: &str) -> String {
        self.register_user(
            username,
            &format!("{}@example.com", username),
            "password123",
        )
        .await
    }
}

#[async_trait]
impl TestUserBuilder for TestApp {
    async fn register_user(&self, username: &str, email: &str, password: &str) -> String {
        let user_data = json!({
            "user": {
                "username": username,
                "email": email,
                "password": password
            }
        });

        let response = self
            .client
            .post(format!("{}/api/users", &self.address))
            .header("Content-Type", "application/json")
            .json(&user_data)
            .send()
            .await
            .expect("Failed to register user");

        let body: serde_json::Value = response
            .json()
            .await
            .expect("Failed to parse registration response");

        body["user"]["token"]
            .as_str()
            .expect("Token not found in response")
            .to_string()
    }
}

/// Trait for building test articles with simplified creation
#[async_trait]
pub trait TestArticleBuilder {
    /// Create an article and return its slug
    async fn create_article(
        &self,
        token: &str,
        title: &str,
        description: &str,
        body: &str,
        tags: Vec<&str>,
    ) -> String;

    /// Create an article with minimal parameters
    async fn create_article_simple(&self, token: &str, title: &str) -> String {
        self.create_article(
            token,
            title,
            "Test description",
            "Test body content",
            vec![],
        )
        .await
    }
}

#[async_trait]
impl TestArticleBuilder for TestApp {
    async fn create_article(
        &self,
        token: &str,
        title: &str,
        description: &str,
        body: &str,
        tags: Vec<&str>,
    ) -> String {
        let tag_list: Vec<serde_json::Value> = tags.iter().map(|t| json!(t)).collect();

        let article_data = json!({
            "article": {
                "title": title,
                "description": description,
                "body": body,
                "tagList": tag_list
            }
        });

        let response = self
            .client
            .post(format!("{}/api/articles", &self.address))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .json(&article_data)
            .send()
            .await
            .expect("Failed to create article");

        let body: serde_json::Value = response
            .json()
            .await
            .expect("Failed to parse article response");

        body["article"]["slug"]
            .as_str()
            .expect("Slug not found in response")
            .to_string()
    }
}

/// Trait for building test API keys with simplified operations
#[async_trait]
pub trait TestApiKeyBuilder {
    /// Create an API key and return the full response body
    async fn create_api_key(&self, token: &str, name: &str) -> serde_json::Value;

    /// Create an API key and return just the key ID
    async fn create_api_key_simple(&self, token: &str, name: &str) -> String;

    /// List all API keys for the authenticated user
    async fn list_api_keys(&self, token: &str) -> Vec<serde_json::Value>;

    /// Delete an API key by ID
    async fn delete_api_key(&self, token: &str, key_id: &str) -> reqwest::Response;

    /// Get the URL for API keys endpoint
    fn api_keys_url(&self) -> String;

    /// Get the URL for a specific API key
    fn api_key_url(&self, key_id: &str) -> String;
}

#[async_trait]
impl TestApiKeyBuilder for TestApp {
    async fn create_api_key(&self, token: &str, name: &str) -> serde_json::Value {
        let response = self.client
            .post(self.api_keys_url())
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({"name": name}))
            .send()
            .await
            .expect("Failed to create API key");

        response.json().await.expect("Failed to parse API key response")
    }

    async fn create_api_key_simple(&self, token: &str, name: &str) -> String {
        let body = self.create_api_key(token, name).await;
        body["id"].as_str().expect("API key ID not found").to_string()
    }

    async fn list_api_keys(&self, token: &str) -> Vec<serde_json::Value> {
        let response = self.client
            .get(self.api_keys_url())
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .expect("Failed to list API keys");

        let body: serde_json::Value = response.json().await.expect("Failed to parse response");
        body["api_keys"].as_array().expect("api_keys not found").clone()
    }

    async fn delete_api_key(&self, token: &str, key_id: &str) -> reqwest::Response {
        self.client
            .delete(self.api_key_url(key_id))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .expect("Failed to delete API key")
    }

    fn api_keys_url(&self) -> String {
        format!("{}/api/keys", &self.address)
    }

    fn api_key_url(&self, key_id: &str) -> String {
        format!("{}/api/keys/{}", &self.address, key_id)
    }
}

// ============================================================================
// Helper Macros
// ============================================================================

/// Creates an Authorization header with Bearer token
#[macro_export]
macro_rules! auth_header {
    ($token:expr) => {
        ("Authorization", format!("Bearer {}", $token))
    };
}

/// Makes authenticated HTTP requests with bearer token
#[macro_export]
macro_rules! bearer_request {
    (get $app:expr, $url:expr, $token:expr) => {
        $app.client
            .get($url)
            .header(
                $crate::auth_header!($token).0,
                $crate::auth_header!($token).1,
            )
            .send()
            .await
    };

    (post $app:expr, $url:expr, $token:expr, $body:expr) => {
        $app.client
            .post($url)
            .header("Content-Type", "application/json")
            .header(
                $crate::auth_header!($token).0,
                $crate::auth_header!($token).1,
            )
            .json(&$body)
            .send()
            .await
    };

    (put $app:expr, $url:expr, $token:expr, $body:expr) => {
        $app.client
            .put($url)
            .header("Content-Type", "application/json")
            .header(
                $crate::auth_header!($token).0,
                $crate::auth_header!($token).1,
            )
            .json(&$body)
            .send()
            .await
    };

    (delete $app:expr, $url:expr, $token:expr) => {
        $app.client
            .delete($url)
            .header(
                $crate::auth_header!($token).0,
                $crate::auth_header!($token).1,
            )
            .send()
            .await
    };
}

/// Parses JSON response body
#[macro_export]
macro_rules! parse_json {
    ($response:expr) => {{
        $response
            .json::<serde_json::Value>()
            .await
            .expect("Failed to parse response as JSON")
    }};
}

/// Asserts HTTP status code
#[macro_export]
macro_rules! assert_status {
    ($response:expr, $expected:expr) => {
        assert_eq!(
            $response.status(),
            $expected,
            "Expected status {}, got {}",
            $expected,
            $response.status()
        )
    };
}

// ============================================================================
// HTML Response Validation
// ============================================================================

/// Trait for validating HTML responses from page routes
#[async_trait]
pub trait HtmlResponseValidator {
    /// Assert response is HTML and return the body text
    async fn assert_html_response(self) -> String;
}

#[async_trait]
impl HtmlResponseValidator for reqwest::Response {
    async fn assert_html_response(self) -> String {
        // Check content type
        let content_type = self
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(
            content_type.contains("text/html") || content_type.is_empty(),
            "Expected HTML content type, got: {}",
            content_type
        );

        // Get and validate HTML structure
        let body = self.text().await.expect("Failed to get response body");
        assert!(
            body.contains("<html") || body.contains("<!DOCTYPE html"),
            "Response does not contain valid HTML structure"
        );

        body
    }
}

/// Assert that response body contains all expected strings
pub fn assert_body_contains(body: &str, expected: &[&str]) {
    for text in expected {
        assert!(
            body.contains(text),
            "Response body missing expected text: '{}'",
            text
        );
    }
}

// ============================================================================
// Test Fixture Builder
// ============================================================================

/// Test fixture for complex multi-user test scenarios
pub struct TestFixture {
    pub app: TestApp,
    pub users: HashMap<String, String>, // username -> token
}

impl TestFixture {
    /// Create a new test fixture with a fresh test app
    pub async fn new() -> Self {
        TestFixture {
            app: spawn_app().await,
            users: HashMap::new(),
        }
    }

    /// Add a user to the fixture with default email pattern
    pub async fn with_user_default(mut self, name: &str) -> Self {
        let token = self.app.register_user_default(name).await;
        self.users.insert(name.to_string(), token);
        self
    }

    /// Get a user's authentication token by username
    pub fn get_token(&self, user: &str) -> String {
        self.users
            .get(user)
            .unwrap_or_else(|| panic!("User '{}' not found in fixture", user))
            .clone()
    }
}
