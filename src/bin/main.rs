// src/bin/main.rs

//! Application entry point.
//!
//! This is the main entry point for the CrustyRustacean Dev Blog application.
//! It initializes tracing, loads configuration, and starts the Actix Web server.

use crustyrustacean_dev_blog_lib::configuration::get_configuration;
use crustyrustacean_dev_blog_lib::startup::Application;
use crustyrustacean_dev_blog_lib::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing/logging
    let subscriber = get_subscriber(
        "crustyrustacean-dev-blog".into(),
        "info".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    // Load configuration from YAML files and environment variables
    tracing::info!("Loading configuration...");
    let configuration = get_configuration().expect("Failed to read configuration.");

    // Build and run the application
    tracing::info!("Building application...");
    let application = Application::build(configuration).await?;

    tracing::info!(
        "Application started on port {}",
        application.port()
    );

    // Run until stopped (Ctrl+C or SIGTERM)
    application.run_until_stopped().await?;

    Ok(())
}
