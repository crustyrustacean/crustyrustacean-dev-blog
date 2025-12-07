// src/lib/routes/sitemap.rs

use crate::{ApiError, AppState, escape_xml};
use axum::{
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::Utc;

pub async fn get_sitemap(State(state): State<AppState>) -> Result<Response, ApiError> {
    let conn = state
        .db
        .connect()
        .map_err(ApiError::from_connection_error)?;

    // Get all articles with their update times
    let mut article_rows = conn
        .query(
            r#"
            SELECT slug, updated_at
            FROM articles
            ORDER BY updated_at DESC
            "#,
            libsql::params![],
        )
        .await?;

    let mut urls = Vec::new();

    // Add static pages with priorities and change frequencies
    let static_pages = vec![
        ("", "1.0", "daily"),         // Homepage
        ("articles", "0.9", "daily"), // Articles listing
        ("about", "0.5", "monthly"),  // About page
        ("privacy", "0.3", "yearly"), // Privacy policy
        ("terms", "0.3", "yearly"),   // Terms of service
        ("rss", "0.8", "daily"),      // RSS feed
    ];

    for (path, priority, changefreq) in static_pages {
        let url = if path.is_empty() {
            format!(
                r#"
    <url>
        <loc>https://crusty-rustacean.com/</loc>
        <changefreq>{}</changefreq>
        <priority>{}</priority>
    </url>"#,
                changefreq, priority
            )
        } else {
            format!(
                r#"
    <url>
        <loc>https://crusty-rustacean.com/{}</loc>
        <changefreq>{}</changefreq>
        <priority>{}</priority>
    </url>"#,
                path, changefreq, priority
            )
        };
        urls.push(url);
    }

    // Add all articles
    while let Some(row) = article_rows.next().await? {
        let slug: String = row.get(0)?;
        let updated_at_str: String = row.get(1)?;

        // Parse the timestamp
        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| ApiError::InternalServerError(format!("Invalid timestamp: {}", e)))?
            .with_timezone(&Utc);

        // Format as W3C Datetime (ISO 8601) - use to_rfc3339() for correct timezone format
        let lastmod = updated_at.to_rfc3339();

        // Escape the slug for XML
        let slug_escaped = escape_xml(&slug);

        let url = format!(
            r#"
    <url>
        <loc>https://crusty-rustacean.com/articles/{}</loc>
        <lastmod>{}</lastmod>
        <changefreq>weekly</changefreq>
        <priority>0.8</priority>
    </url>"#,
            slug_escaped, lastmod
        );

        urls.push(url);
    }

    // Build the complete sitemap
    let sitemap = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
{}
</urlset>"#,
        urls.join("")
    );

    // Return response with proper content type
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        sitemap,
    )
        .into_response())
}
