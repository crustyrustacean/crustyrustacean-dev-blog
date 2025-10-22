// src/main.rs

use shuttle_runtime::{CustomError, SecretStore, Secrets};
// dependencies
use crustyrustacean_dev_blog_lib::config::AppConfig;
use crustyrustacean_dev_blog_lib::database::DatabaseConnection;
use crustyrustacean_dev_blog_lib::startup::App;
use crustyrustacean_dev_blog_lib::state::AppState;
use crustyrustacean_dev_blog_lib::telemetry::{get_subscriber, init_subscriber};

// Shuttle entry point
#[shuttle_runtime::main]
async fn main(
    #[shuttle_turso::Turso(
        addr = "{secrets.TURSO_DATABASE_URL}",
        token = "{secrets.TURSO_AUTH_TOKEN}"
    )]
    turso_client: libsql::Database,
    #[Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    // initialize tracing
    tracing::info!("Starting tracing...");
    let subscriber = get_subscriber(
        "crustyrustacean-dev-blog".into(),
        "info".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    // Initialize database connection
    tracing::info!("Getting database connection...");
    let db = DatabaseConnection {
        db: std::sync::Arc::new(turso_client),
    };

    // Run database migrations
    tracing::info!("Running database migrations...");
    db.run_migrations().await.map_err(CustomError::new)?;

    // Load configuration with JWT secret
    tracing::info!("Loading application configuration from secrets...");
    let app_config = AppConfig::try_from(&secrets)?;

    // Build the application state
    tracing::info!("Building application state...");
    let app_state = AppState::new(db, &app_config).map_err(CustomError::new)?;

    // Initialize the application
    tracing::info!("Initializing the application...");
    let app = App::new(app_config, app_state);

    // Return the Axum router to Shuttle
    Ok(app.router.into())
}
