// tests/api/helpers.rs

// types and functions used across all integration tests

// dependencies
use reqwest::Client;
use crustyrustacean_dev_blog_lib::config::AppConfig;
use crustyrustacean_dev_blog_lib::database::DatabaseConnection;
use crustyrustacean_dev_blog_lib::startup::App;
use crustyrustacean_dev_blog_lib::state::AppState;
use crustyrustacean_dev_blog_lib::telemetry::{get_subscriber, init_subscriber};
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
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    temp_db_path.push(format!("test_db_{}_{}.db", std::process::id(), timestamp));
    let test_db = libsql::Builder::new_local(&temp_db_path).build().await.expect("Failed to create test database");
    let db_connection = DatabaseConnection { db: std::sync::Arc::new(test_db) };

    // run migrations for test database
    db_connection.run_migrations().await.expect("Failed to run migrations");

    // set up the app state
    let app_state = AppState::new(db_connection, &app_config).expect("Unable to build the Tera templates");

    // create the test application
    let application = App::new(
        app_config,
        app_state,
    );

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
