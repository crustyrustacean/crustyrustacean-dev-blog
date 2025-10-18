// src/lib/startup.rs

// dependencies
use crate::config::AppConfig;
use crate::routes::{
    create_article, delete_article, favorite_article, follow_user, get_admin_dashboard,
    get_article, get_article_page, get_articles_feed, get_articles_feed_page,
    get_articles_list_page, get_authors_page, get_current_user, get_edit_article_page,
    get_editor_page, get_index, get_login_page, get_my_favorites_page, get_profile,
    get_profile_page, get_register_page, get_rss_feed, get_tags, handle_404_simple, health_check,
    list_articles, list_profiles, login_user, register_user, unfavorite_article, unfollow_user,
    update_article, update_current_user,
};
use crate::state::AppState;
use crate::telemetry::MakeRequestUuid;
use axum::{
    Router,
    http::{HeaderName, HeaderValue, header},
    routing::{get, post},
};
use tokio::net::TcpListener;
use tower_http::{
    cors::CorsLayer,
    request_id::{PropagateRequestIdLayer, SetRequestIdLayer},
    services::ServeDir,
    set_header::SetResponseHeaderLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
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
            .route("/rss", get(get_rss_feed))
            // HTML page routes
            .route("/login", get(get_login_page))
            .route("/register", get(get_register_page))
            .route("/profiles/{username}", get(get_profile_page))
            .route("/profiles", get(get_authors_page))
            .route("/favorites", get(get_my_favorites_page))
            .route("/feed", get(get_articles_feed_page))
            .route("/editor", get(get_editor_page))
            .route("/editor/{slug}", get(get_edit_article_page))
            .route("/articles", get(get_articles_list_page))
            .route("/articles/{slug}", get(get_article_page))
            .route("/admin", get(get_admin_dashboard))
            // API routes
            .route("/api/users", post(register_user))
            .route("/api/users/login", post(login_user))
            .route("/api/user", get(get_current_user).put(update_current_user))
            .route("/api/profiles", get(list_profiles))
            .route("/api/profiles/{username}", get(get_profile))
            .route(
                "/api/profiles/{username}/follow",
                post(follow_user).delete(unfollow_user),
            )
            .route("/api/articles", post(create_article).get(list_articles))
            .route("/api/articles/feed", get(get_articles_feed))
            .route(
                "/api/articles/{slug}",
                get(get_article).put(update_article).delete(delete_article),
            )
            .route(
                "/api/articles/{slug}/favorite",
                post(favorite_article).delete(unfavorite_article),
            )
            .route("/api/tags", get(get_tags))
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
            .layer(SetResponseHeaderLayer::if_not_present(
                header::X_CONTENT_TYPE_OPTIONS,
                HeaderValue::from_static("nosniff"),
            ))
            .layer(SetResponseHeaderLayer::if_not_present(
                header::X_FRAME_OPTIONS,
                HeaderValue::from_static("DENY"),
            ))
            .layer(SetResponseHeaderLayer::if_not_present(
                header::STRICT_TRANSPORT_SECURITY,
                HeaderValue::from_static("max-age=31536000; includeSubDomains"),
            ))
    }

    /// run the application until stopped (utility function to faciliate local integration testing)
    pub async fn run_until_stopped(self, listener: TcpListener) -> Result<(), anyhow::Error> {
        axum::serve(listener, self.router).await?;
        Ok(())
    }
}
