// src/lib/routes/robots.rs

// dependencies
use axum::response::IntoResponse;

pub async fn get_robots_txt() -> impl IntoResponse {
    use axum::http::header;

    let robots_content = include_str!("../../../static/robots.txt");

    (
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        robots_content,
    )
}
