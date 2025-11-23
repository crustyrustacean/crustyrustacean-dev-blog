// src/lib/routes/rss.rs

use crate::{ApiError, AppState};
use axum::{
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::Utc;

pub async fn get_rss_feed(State(state): State<AppState>) -> Result<Response, ApiError> {
    let conn = state.db.connect().map_err(ApiError::from_connection_error)?;

    // Get the latest 20 articles with author information
    let mut article_rows = conn
        .query(
            r#"
            SELECT
                a.slug, a.title, a.description, a.body, a.created_at, a.updated_at,
                u.username, u.email
            FROM articles a
            JOIN users u ON a.author_id = u.id
            ORDER BY a.created_at DESC
            LIMIT 20
            "#,
            libsql::params![],
        )
        .await
        ?;

    let mut items = Vec::new();

    while let Some(row) = article_rows
        .next()
        .await
        ?
    {
        let slug: String = row
            .get(0)
            ?;
        let title: String = row
            .get(1)
            ?;
        let description: String = row
            .get(2)
            ?;
        let _body: String = row
            .get(3)
            ?;
        let created_at_str: String = row
            .get(4)
            ?;
        let _updated_at_str: String = row
            .get(5)
            ?;
        let username: String = row
            .get(6)
            ?;
        let author_email: Option<String> = row.get(7).ok();

        // Parse the created_at timestamp
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| ApiError::InternalServerError(format!("Invalid timestamp: {}", e)))?
            .with_timezone(&Utc);

        // Convert to RFC 2822 format for RSS pubDate
        let pub_date = created_at.to_rfc2822();

        // Escape XML special characters
        let title_escaped = escape_xml(&title);
        let description_escaped = escape_xml(&description);
        let username_escaped = escape_xml(&username);

        // Build the item XML
        let item = format!(
            r#"
        <item>
            <title>{}</title>
            <link>https://crusty-rustacean.com/articles/{}</link>
            <description>{}</description>
            <author>{}</author>
            <guid isPermaLink="true">https://crusty-rustacean.com/articles/{}</guid>
            <pubDate>{}</pubDate>
        </item>"#,
            title_escaped,
            slug,
            description_escaped,
            author_email
                .unwrap_or_else(|| format!("crusty.rustacean@gmail.com ({})", username_escaped)),
            slug,
            pub_date
        );

        items.push(item);
    }

    // Get current date for lastBuildDate
    let build_date = Utc::now().to_rfc2822();

    // Build the complete RSS feed
    let rss_feed = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
    <channel>
        <title>CrustyRustacean Dev Blog</title>
        <link>https://crusty-rustacean.com</link>
        <description>A developer's journey through Rust, systems programming, and modern web development</description>
        <language>en-us</language>
        <lastBuildDate>{}</lastBuildDate>
        <atom:link href="https://crusty-rustacean.com/rss" rel="self" type="application/rss+xml" />
        {}
    </channel>
</rss>"#,
        build_date,
        items.join("\n")
    );

    // Return response with proper content type
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")],
        rss_feed,
    )
        .into_response())
}

// Helper function to escape XML special characters
fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("Hello & goodbye"), "Hello &amp; goodbye");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(
            escape_xml("It's \"quoted\" & <escaped>"),
            "It&apos;s &quot;quoted&quot; &amp; &lt;escaped&gt;"
        );
    }
}
