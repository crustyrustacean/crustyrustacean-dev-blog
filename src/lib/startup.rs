// src/lib/startup.rs

// dependencies
use crate::config::AppConfig;
use crate::routes::{
    get_index, health_check, register_user, login_user, get_current_user, update_current_user,
    get_profile, follow_user, unfollow_user, get_login_page, get_register_page, get_profile_page,
    handle_404_simple,
};
use crate::state::AppState;
use crate::telemetry::MakeRequestUuid;
use axum::{
    Router,
    http::HeaderName,
    routing::{get, post},
};
use tokio::net::TcpListener;
use tower_http::{
    request_id::{PropagateRequestIdLayer, SetRequestIdLayer},
    services::ServeDir,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    cors::CorsLayer,
};
use tracing::Level;

// struct type to represent the application
pub struct App {
    pub config: AppConfig,
    pub router: Router,
}

// methods to build the application
impl App {
    // create a new application instance
    pub fn new(config: AppConfig, state: AppState) -> Self {
        let router = Self::build_router(state);
        Self { config, router }
    }

    // build the application router with all routes and middleware layers
    pub fn build_router(state: AppState) -> Router {
        // define the tracing layer
        let trace_layer = TraceLayer::new_for_http()
            .make_span_with(
                DefaultMakeSpan::new()
                    .include_headers(true)
                    .level(Level::INFO),
            )
            .on_response(DefaultOnResponse::new().include_headers(true));
        let x_request_id = HeaderName::from_static("x-request-id");

        // build the application router
        Router::new()
            .route("/health_check", get(health_check))
            .route("/", get(get_index))
            // HTML page routes
            .route("/login", get(get_login_page))
            .route("/register", get(get_register_page))
            .route("/profiles/{username}", get(get_profile_page))
            // API routes
            .route("/api/users", post(register_user))
            .route("/api/users/login", post(login_user))
            .route("/api/user", get(get_current_user).put(update_current_user))
            .route("/api/profiles/{username}", get(get_profile))
            .route("/api/profiles/{username}/follow", post(follow_user).delete(unfollow_user))
            .nest_service("/static", ServeDir::new("static"))
            .fallback(handle_404_simple)
            .with_state(state)
            .layer(CorsLayer::permissive())
            .layer(SetRequestIdLayer::new(
                x_request_id.clone(),
                MakeRequestUuid,
            ))
            .layer(PropagateRequestIdLayer::new(x_request_id))
            .layer(trace_layer)
    }

    /// run the application until stopped (utility function to faciliate local integration testing)
    pub async fn run_until_stopped(self, listener: TcpListener) -> Result<(), anyhow::Error> {
        axum::serve(listener, self.router).await?;
        Ok(())
    }
}
