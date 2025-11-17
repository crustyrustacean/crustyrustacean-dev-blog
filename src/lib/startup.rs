// src/lib/startup.rs

// dependencies
use crate::config::AppConfig;
use crate::routes::{
    add_comment, change_password, complete_password_reset, create_api_key, create_article,
    create_category, create_newsletter, delete_api_key, delete_article, delete_category,
    delete_comment, delete_media, delete_newsletter, delete_tag, download_media,
    favorite_article, follow_user, get_about, get_admin_dashboard, get_api_keys_admin_page,
    get_article, get_article_page, get_articles_feed, get_articles_feed_page,
    get_articles_list_page, get_authors_page, get_categories, get_categories_admin_page,
    get_category, get_comments, get_current_user, get_drafts_admin_page, get_edit_article_page,
    get_editor_page, get_index, get_login_page, get_media_library_page, get_media_metadata,
    get_my_favorites_page, get_newsletter, get_newsletter_stats, get_password_reset_page,
    get_password_reset_request_page, get_privacy, get_profile, get_profile_page, get_register_page,
    get_robots_txt, get_rss_feed,
    get_sitemap, get_tags, get_tags_admin_page, get_terms, handle_404_simple, health_check,
    list_api_keys, list_articles, list_media, list_newsletters, list_profiles, list_user_drafts,
    login_user, mobile_upload_article, newsletter_confirmed_page, newsletter_page,
    newsletter_unsubscribed_page, admin_newsletters_page, register_user,
    request_password_reset, search_articles, subscribe, confirm_subscription, unsubscribe,
    unfavorite_article, unfollow_user, update_article, update_category, update_current_user,
    update_media_metadata, update_newsletter, update_tag, upload_markdown_file, upload_media,
    send_newsletter,
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
    compression::CompressionLayer,
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
            .route("/robots.txt", get(get_robots_txt))
            .route("/sitemap.xml", get(get_sitemap))
            .route("/", get(get_index))
            .route("/about", get(get_about))
            .route("/privacy", get(get_privacy))
            .route("/terms", get(get_terms))
            .route("/rss", get(get_rss_feed))
            // HTML page routes
            .route("/login", get(get_login_page))
            .route("/register", get(get_register_page))
            .route("/password-reset/request", get(get_password_reset_request_page))
            .route("/profiles/{username}", get(get_profile_page))
            .route("/profiles", get(get_authors_page))
            .route("/favorites", get(get_my_favorites_page))
            .route("/feed", get(get_articles_feed_page))
            .route("/editor", get(get_editor_page))
            .route("/editor/{slug}", get(get_edit_article_page))
            .route("/articles", get(get_articles_list_page))
            .route("/articles/{slug}", get(get_article_page))
            .route("/admin", get(get_admin_dashboard))
            .route("/admin/tags", get(get_tags_admin_page))
            .route("/admin/categories", get(get_categories_admin_page))
            .route("/admin/api-keys", get(get_api_keys_admin_page))
            .route("/admin/media", get(get_media_library_page))
            .route("/admin/drafts", get(get_drafts_admin_page))
            .route("/admin/newsletters", get(admin_newsletters_page))
            .route("/newsletter", get(newsletter_page))
            .route("/newsletter/confirmed/{token}", get(newsletter_confirmed_page))
            .route("/newsletter/unsubscribed/{token}", get(newsletter_unsubscribed_page))
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
            .route("/api/articles/drafts", get(list_user_drafts))
            .route("/api/articles/upload-markdown", post(upload_markdown_file))
            .route(
                "/api/articles/{slug}",
                get(get_article).put(update_article).delete(delete_article),
            )
            .route(
                "/api/articles/{slug}/favorite",
                post(favorite_article).delete(unfavorite_article),
            )
            .route(
                "/api/articles/{slug}/comments",
                post(add_comment).get(get_comments),
            )
            .route(
                "/api/articles/{slug}/comments/{comment_id}",
                axum::routing::delete(delete_comment),
            )
            .route("/api/tags", get(get_tags))
            .route(
                "/api/tags/{name}",
                axum::routing::put(update_tag).delete(delete_tag),
            )
            .route("/api/categories", post(create_category).get(get_categories))
            .route(
                "/api/categories/{slug}",
                get(get_category)
                    .put(update_category)
                    .delete(delete_category),
            )
            .route("/api/keys", post(create_api_key).get(list_api_keys))
            .route("/api/keys/{key_id}", axum::routing::delete(delete_api_key))
            .route("/api/mobile/upload", post(mobile_upload_article))
            .route("/api/media", post(upload_media).get(list_media))
            .route(
                "/api/media/{id}",
                get(get_media_metadata)
                    .put(update_media_metadata)
                    .delete(delete_media),
            )
            .route("/api/media/{id}/download", get(download_media))
            .route("/api/search", get(search_articles))
            // Newsletter routes
            .route("/api/newsletters/subscribe", post(subscribe))
            .route("/api/newsletters/confirm/{token}", post(confirm_subscription).get(confirm_subscription))
            .route("/api/newsletters/unsubscribe/{token}", post(unsubscribe).get(unsubscribe))
            .route("/api/admin/newsletters", post(create_newsletter).get(list_newsletters))
            .route("/api/admin/newsletters/stats", get(get_newsletter_stats))
            .route(
                "/api/admin/newsletters/{id}",
                get(get_newsletter)
                    .put(update_newsletter)
                    .delete(delete_newsletter),
            )
            .route("/api/admin/newsletters/{id}/send", post(send_newsletter))
            // Password reset routes
            .route("/api/password-reset/request", post(request_password_reset))
            .route("/password-reset/{token}", get(get_password_reset_page))
            .route("/api/password-reset/{token}", post(complete_password_reset))
            .route("/api/account/password", post(change_password))
            .nest_service(
                "/static",
                // Static file service with aggressive caching for versioned assets
                tower::ServiceBuilder::new()
                    .layer(SetResponseHeaderLayer::if_not_present(
                        header::CACHE_CONTROL,
                        HeaderValue::from_static("public, max-age=31536000, immutable"),
                    ))
                    .service(
                        ServeDir::new("static")
                            .precompressed_gzip()
                            .precompressed_br()
                            .append_index_html_on_directories(false)
                    )
            )
            .fallback(handle_404_simple)
            .with_state(state)
            .layer(CompressionLayer::new())
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
