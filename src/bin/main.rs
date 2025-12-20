// src/main.rs

// dependencies
use anyhow::{Context, Result};
use crustyrustacean_dev_blog_lib::config::AppConfig;
use crustyrustacean_dev_blog_lib::database::DatabaseConnection;
use crustyrustacean_dev_blog_lib::service::AppService;
use crustyrustacean_dev_blog_lib::state::AppState;
use crustyrustacean_dev_blog_lib::storage::{OpenDalStorage, StorageBackend};
use crustyrustacean_dev_blog_lib::telemetry::{get_subscriber, init_subscriber};
use opendal::Operator;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file for local development (ignored in production if not present)
    dotenvy::dotenv().ok();

    // Initialize tracing
    let subscriber = get_subscriber(
        "crustyrustacean-dev-blog".into(),
        "info".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    tracing::info!("Starting application...");

    // Load configuration from environment variables
    tracing::info!("Loading application configuration from environment...");
    let app_config = AppConfig::from_env().context("Failed to load application configuration")?;

    // Initialize Turso database connection
    tracing::info!("Connecting to Turso database...");
    let turso_url = env::var("TURSO_DATABASE_URL")
        .context("Missing required environment variable: TURSO_DATABASE_URL")?;
    let turso_token = env::var("TURSO_AUTH_TOKEN")
        .context("Missing required environment variable: TURSO_AUTH_TOKEN")?;

    let turso_client = libsql::Builder::new_remote(turso_url, turso_token)
        .build()
        .await
        .context("Failed to connect to Turso database")?;

    let db = DatabaseConnection {
        db: Arc::new(turso_client),
    };

    // Run database migrations
    tracing::info!("Running database migrations...");
    db.run_migrations()
        .await
        .context("Failed to run database migrations")?;

    // Initialize S3 storage via OpenDAL
    tracing::info!("Initializing storage backend...");
    let operator = create_s3_operator().context("Failed to initialize S3 storage")?;
    let storage: Arc<dyn StorageBackend> = Arc::new(OpenDalStorage::new(operator));

    // Build the application state
    tracing::info!("Building application state...");
    let app_state =
        AppState::new(db, storage, app_config).context("Failed to build application state")?;

    // Initialize the application service
    tracing::info!("Initializing application service...");
    let app_service = AppService::new(app_state);

    // Bind to port (Railway provides PORT env var, default to 8000 for local dev)
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse()
        .context("Invalid PORT value")?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!("Starting server on {}...", addr);
    app_service
        .run(addr)
        .await
        .context("Server error")?;

    Ok(())
}

/// Create an S3-compatible storage operator using environment variables.
///
/// Required environment variables:
/// - `S3_BUCKET`: Bucket name
/// - `S3_REGION`: AWS region (use "auto" for Cloudflare R2)
/// - `S3_ACCESS_KEY_ID`: Access key ID
/// - `S3_SECRET_ACCESS_KEY`: Secret access key
///
/// Optional environment variables:
/// - `S3_ENDPOINT`: Custom endpoint URL (required for R2, MinIO, etc.)
fn create_s3_operator() -> Result<Operator> {
    let bucket = env::var("S3_BUCKET").context("Missing S3_BUCKET")?;
    let region = env::var("S3_REGION").context("Missing S3_REGION")?;
    let access_key_id = env::var("S3_ACCESS_KEY_ID").context("Missing S3_ACCESS_KEY_ID")?;
    let secret_access_key =
        env::var("S3_SECRET_ACCESS_KEY").context("Missing S3_SECRET_ACCESS_KEY")?;

    let mut builder = opendal::services::S3::default()
        .bucket(&bucket)
        .region(&region)
        .access_key_id(&access_key_id)
        .secret_access_key(&secret_access_key);

    // Optional: Custom endpoint for S3-compatible services (Cloudflare R2, MinIO, etc.)
    if let Ok(endpoint) = env::var("S3_ENDPOINT") {
        builder = builder.endpoint(&endpoint);
    }

    let operator = Operator::new(builder)?.finish();
    Ok(operator)
}
