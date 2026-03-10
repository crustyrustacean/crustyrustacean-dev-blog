// src/lib/startup.rs

//! Application startup and server configuration.
//!
//! This module provides the `Application` struct that handles building and
//! running the Actix Web server with all necessary configuration.

use crate::configuration::{DatabaseSettings, Settings};
use crate::routes;
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

/// The main application struct that wraps the Actix Web server.
pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    /// Build the application from configuration settings.
    ///
    /// This method:
    /// 1. Creates a database connection pool
    /// 2. Runs database migrations
    /// 3. Binds to the configured address
    /// 4. Sets up all routes and middleware
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let connection_pool = get_connection_pool(&configuration.database);

        // Run database migrations
        tracing::info!("Running database migrations...");
        sqlx::migrate!("./migrations")
            .run(&connection_pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to run migrations: {}", e))?;

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr()?.port();

        tracing::info!("Server starting on {}", address);

        let server = run(listener, connection_pool, configuration).await?;

        Ok(Self { port, server })
    }

    /// Get the port the server is listening on.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Run the server until stopped.
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

/// Create a database connection pool from configuration.
pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy_with(configuration.connect_options())
}

/// Application base URL wrapper for route handlers.
#[derive(Clone)]
pub struct ApplicationBaseUrl(pub String);

/// Configure and run the Actix Web server.
async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    config: Settings,
) -> Result<Server, anyhow::Error> {
    // Wrap configuration in Arc for sharing across workers
    let config_data = std::sync::Arc::new(config.clone());
    let db_pool_data = web::Data::new(db_pool);
    let base_url = web::Data::new(ApplicationBaseUrl(config.application.base_url.clone()));

    let server = HttpServer::new(move || {
        App::new()
            // Middleware
            .wrap(TracingLogger::default())
            // Static files
            .service(
                actix_files::Files::new("/static", "./static")
                    .use_last_modified(true)
                    .use_etag(true),
            )
            // Configure routes
            .configure(routes::configure)
            // Application data
            .app_data(db_pool_data.clone())
            .app_data(base_url.clone())
            .app_data(web::Data::new(config_data.clone()))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
