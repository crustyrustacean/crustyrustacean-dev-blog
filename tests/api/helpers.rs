// tests/api/helpers.rs

// types and functions used across all integration tests

// dependencies
use anyhow::{Context, Result, anyhow};
use crustyrustacean_dev_blog_lib::config::AppConfig;
use crustyrustacean_dev_blog_lib::database::DatabaseConnection;
use crustyrustacean_dev_blog_lib::service::AppService;
use crustyrustacean_dev_blog_lib::state::AppState;
use crustyrustacean_dev_blog_lib::storage::{OpenDalStorage, StorageBackend};
use crustyrustacean_dev_blog_lib::telemetry::{get_subscriber, init_subscriber};
use opendal::Operator;
use reqwest::{Client, Response, StatusCode};
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
        // Skip Mailtrap API token in tests - we want to use LoggingEmailSender
        if k == "MAILTRAP_API_TOKEN" {
            continue;
        }
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
    let app_state = AppState::new(db_connection.clone(), storage, app_config)
        .expect("Unable to build the Tera templates");

    // create the test application
    let application = AppService::new(app_state);

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

        let status = response.status();
        if !status.is_success() {
            let body: serde_json::Value = response
                .json()
                .await
                .expect("Failed to parse error response");
            panic!("Registration failed with status {}: {:?}", status, body);
        }

        // Registration now requires email verification, so we need to:
        // 1. Directly mark the user as verified in the database
        // 2. Log in to get a token

        // Mark email as verified directly in database
        let conn = self
            .db
            .connect()
            .expect("Failed to connect to test database");
        conn.execute(
            "UPDATE users SET email_verified = 1 WHERE email = ?",
            libsql::params![email],
        )
        .await
        .expect("Failed to verify user email in test");

        // Now log in to get a token
        let login_data = json!({
            "user": {
                "email": email,
                "password": password
            }
        });

        let login_response = self
            .client
            .post(format!("{}/api/users/login", &self.address))
            .header("Content-Type", "application/json")
            .json(&login_data)
            .send()
            .await
            .expect("Failed to login user");

        let login_body: serde_json::Value = login_response
            .json()
            .await
            .expect("Failed to parse login response");

        login_body["user"]["token"]
            .as_str()
            .expect("Token not found in login response")
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

    /// Create a draft article and return its slug
    async fn create_draft_article(
        &self,
        token: &str,
        title: &str,
        description: &str,
        body: &str,
    ) -> String;

    /// Create a draft article with minimal parameters
    async fn create_draft_article_simple(&self, token: &str, title: &str) -> String {
        self.create_draft_article(token, title, "Draft description", "Draft body content")
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

        let status = response.status();
        let body: serde_json::Value = response
            .json()
            .await
            .expect("Failed to parse article response");

        if !status.is_success() {
            panic!("Article creation failed with status {}: {:?}", status, body);
        }

        body["article"]["slug"]
            .as_str()
            .expect("Slug not found in response")
            .to_string()
    }

    async fn create_draft_article(
        &self,
        token: &str,
        title: &str,
        description: &str,
        body: &str,
    ) -> String {
        let article_data = json!({
            "article": {
                "title": title,
                "description": description,
                "body": body,
                "draft": true
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
            .expect("Failed to create draft article");

        let status = response.status();
        let body: serde_json::Value = response
            .json()
            .await
            .expect("Failed to parse draft article response");

        if !status.is_success() {
            panic!(
                "Draft article creation failed with status {}: {:?}",
                status, body
            );
        }

        body["article"]["slug"]
            .as_str()
            .expect("Slug not found in draft response")
            .to_string()
    }
}

/// Trait for adding comments to articles
#[async_trait]
#[allow(dead_code)]
pub trait TestCommentBuilder {
    /// Add a comment to an article and return the comment ID as string
    async fn add_comment(&self, token: &str, slug: &str, body: &str) -> String;

    /// Add a simple comment with default text
    async fn add_comment_simple(&self, token: &str, slug: &str) -> String {
        self.add_comment(token, slug, "This is a test comment.")
            .await
    }
}

#[async_trait]
impl TestCommentBuilder for TestApp {
    async fn add_comment(&self, token: &str, slug: &str, comment_body: &str) -> String {
        let comment_data = json!({
            "comment": {
                "body": comment_body
            }
        });

        let response = self
            .client
            .post(format!("{}/api/articles/{}/comments", &self.address, slug))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .json(&comment_data)
            .send()
            .await
            .expect("Failed to add comment");

        let status = response.status();
        let body: serde_json::Value = response
            .json()
            .await
            .expect("Failed to parse comment response");

        if !status.is_success() {
            panic!("Comment creation failed with status {}: {:?}", status, body);
        }

        body["comment"]["id"]
            .as_str()
            .expect("Comment ID not found in response")
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
        let response = self
            .client
            .post(self.api_keys_url())
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({"name": name}))
            .send()
            .await
            .expect("Failed to create API key");

        response
            .json()
            .await
            .expect("Failed to parse API key response")
    }

    async fn create_api_key_simple(&self, token: &str, name: &str) -> String {
        let body = self.create_api_key(token, name).await;
        body["id"]
            .as_str()
            .expect("API key ID not found")
            .to_string()
    }

    async fn list_api_keys(&self, token: &str) -> Vec<serde_json::Value> {
        let response = self
            .client
            .get(self.api_keys_url())
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .expect("Failed to list API keys");

        let body: serde_json::Value = response.json().await.expect("Failed to parse response");
        body["api_keys"]
            .as_array()
            .expect("api_keys not found")
            .clone()
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

/// Assert that the navbar shows authenticated state (username visible, no login/register links)
pub fn assert_navbar_authenticated(body: &str, username: &str) {
    // Username should be visible in navbar
    assert!(
        body.contains(username),
        "Username '{}' should appear in navbar when authenticated",
        username
    );

    // Login and Register links should not both be visible
    let body_lowercase = body.to_lowercase();
    let has_login_link =
        body_lowercase.contains(">login<") || body_lowercase.contains("login</a>");
    let has_register_link =
        body_lowercase.contains(">register<") || body_lowercase.contains("register</a>");

    assert!(
        !(has_login_link && has_register_link),
        "Login and Register links should not both be visible when user is authenticated"
    );
}

/// Assert that an HTML page contains the expected JavaScript file with version
pub fn assert_js_loaded(body: &str, js_file: &str, version: &str) {
    let expected = format!("{}?v={}", js_file, version);
    assert!(
        body.contains(&expected),
        "Expected JavaScript file '{}' with version '{}' not found in response",
        js_file,
        version
    );
}

/// Get the current app version from Cargo.toml (matches APP_VERSION in templates)
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

// ============================================================================
// Test Fixture Builder
// ============================================================================

/// Test fixture for complex multi-user test scenarios
pub struct TestFixture {
    pub app: TestApp,
    pub users: HashMap<String, String>, // username -> token
    pub admin_user: Option<String>,     // Track the admin user
}

impl TestFixture {
    /// Create a new test fixture with a fresh test app
    pub async fn new() -> Self {
        TestFixture {
            app: spawn_app().await,
            users: HashMap::new(),
            admin_user: None,
        }
    }

    /// Add a user to the fixture with default email pattern
    pub async fn with_user_default(mut self, name: &str) -> Self {
        let token = self.app.register_user_default(name).await;
        self.users.insert(name.to_string(), token);

        // First user is always admin
        if self.admin_user.is_none() {
            self.admin_user = Some(name.to_string());
        }

        self
    }

    /// Get a user's authentication token by username
    pub fn get_token(&self, user: &str) -> String {
        self.users
            .get(user)
            .unwrap_or_else(|| panic!("User '{}' not found in fixture", user))
            .clone()
    }

    /// Promote a user to author role (requires first user to be admin)
    pub async fn promote_to_author(mut self, username: &str) -> Self {
        // Get admin token (first user is always admin)
        let admin_username = self
            .admin_user
            .as_ref()
            .expect("No admin user in fixture")
            .clone();
        let admin_token = self.get_token(&admin_username);

        // Get the user ID
        let users_response = self
            .app
            .client
            .get(format!(
                "{}/api/admin/users?search={}",
                &self.app.address, username
            ))
            .header("Authorization", format!("Bearer {}", admin_token))
            .send()
            .await
            .expect("Failed to search for user");

        let users_json: serde_json::Value = users_response
            .json()
            .await
            .expect("Failed to parse users response");

        // Find the exact username match (search uses LIKE so may return multiple matches)
        let users_array = users_json["data"]["users"]
            .as_array()
            .unwrap_or_else(|| panic!("Users array not found in response: {:?}", users_json));

        let user = users_array
            .iter()
            .find(|u| u["username"].as_str() == Some(username))
            .unwrap_or_else(|| {
                panic!(
                    "User '{}' not found in search results. Found: {:?}",
                    username, users_array
                )
            });

        let user_id = user["id"].as_str().unwrap_or_else(|| {
            panic!(
                "User ID not found for '{}'. User data: {:?}",
                username, user
            )
        });

        // Promote to author
        let promote_response = self
            .app
            .client
            .put(format!("{}/api/admin/users/{}", &self.app.address, user_id))
            .header("Authorization", format!("Bearer {}", admin_token))
            .json(&serde_json::json!({
                "role": "author"
            }))
            .send()
            .await
            .expect("Failed to promote user");

        let promote_status = promote_response.status();
        let promote_body: serde_json::Value = promote_response
            .json()
            .await
            .expect("Failed to parse promotion response");

        // Check promotion succeeded
        if !promote_status.is_success() {
            panic!(
                "Failed to promote user '{}' (status {}): {:?}",
                username, promote_status, promote_body
            );
        }

        // Verify the response shows the updated role
        let updated_role = promote_body["data"]["user"]["role"]
            .as_str()
            .expect("Role not found in promotion response");

        if updated_role != "author" && updated_role != "admin" {
            panic!(
                "Promotion API returned success but role is still: {}",
                updated_role
            );
        }

        // Get new token for the promoted user

        let login_response = self
            .app
            .client
            .post(format!("{}/api/users/login", &self.app.address))
            .json(&serde_json::json!({
                "user": {
                    "email": format!("{}@example.com", username),
                    "password": "password123"
                }
            }))
            .send()
            .await
            .expect("Failed to log in");

        let login_json: serde_json::Value = login_response
            .json()
            .await
            .expect("Failed to parse login response");

        // Verify the user has author role
        let role = login_json["user"]["role"]
            .as_str()
            .expect("Role not found in login response");

        if role != "author" && role != "admin" {
            panic!(
                "User '{}' was promoted but login still shows role: {}",
                username, role
            );
        }

        let new_token = login_json["user"]["token"]
            .as_str()
            .expect("Token not found")
            .to_string();

        // Update token in fixture
        self.users.insert(username.to_string(), new_token);
        self
    }
}

// ============================================================================
// Additional Assertion Helpers
// ============================================================================

/// Assert that response status is one of the expected status codes
pub fn assert_status_in(response: &Response, statuses: &[StatusCode]) {
    let actual = response.status();
    assert!(
        statuses.contains(&actual),
        "Expected one of {:?}, got {}",
        statuses,
        actual
    );
}

// ============================================================================
// Unauthorized Request Helpers
// ============================================================================

/// Trait for making unauthorized requests (without authentication headers)
#[async_trait]
pub trait UnauthorizedRequestHelper {
    /// Make an unauthorized POST request
    async fn post_unauthorized(&self, url: String, body: serde_json::Value) -> Response;

    /// Make an unauthorized GET request
    async fn get_unauthorized(&self, url: String) -> Response;

    /// Make an unauthorized DELETE request
    async fn delete_unauthorized(&self, url: String) -> Response;
}

#[async_trait]
impl UnauthorizedRequestHelper for TestApp {
    async fn post_unauthorized(&self, url: String, body: serde_json::Value) -> Response {
        self.client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    async fn get_unauthorized(&self, url: String) -> Response {
        self.client
            .get(url)
            .send()
            .await
            .expect("Failed to execute request")
    }

    async fn delete_unauthorized(&self, url: String) -> Response {
        self.client
            .delete(url)
            .send()
            .await
            .expect("Failed to execute request")
    }
}
