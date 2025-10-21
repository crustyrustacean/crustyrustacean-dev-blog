// tests/api/helpers.rs

// types and functions used across all integration tests

// dependencies
use crustyrustacean_dev_blog_lib::config::AppConfig;
use crustyrustacean_dev_blog_lib::database::DatabaseConnection;
use crustyrustacean_dev_blog_lib::startup::App;
use crustyrustacean_dev_blog_lib::state::AppState;
use crustyrustacean_dev_blog_lib::telemetry::{get_subscriber, init_subscriber};
use reqwest::Client;
use std::env::var;
use std::io::{sink, stdout};
use std::sync::LazyLock;
use tokio::net::TcpListener;

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

// struct type which models a test application
#[allow(dead_code)]
pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub client: Client,
}

// helper function which builds and returns a test application
pub async fn spawn_app() -> TestApp {
    // ensure that the tracing is only initialized once
    LazyLock::force(&TRACING);

    // set up the configuration with test JWT secret
    let app_config = AppConfig::new("test-secret-key-for-integration-tests".to_string());

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

    // set up the app state
    let app_state =
        AppState::new(db_connection, &app_config).expect("Unable to build the Tera templates");

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
