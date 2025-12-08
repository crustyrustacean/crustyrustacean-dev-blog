// tests/api/rss.rs

use crate::helpers::{TestArticleBuilder, TestUserBuilder, spawn_app};

#[tokio::test]
async fn rss_feed_returns_valid_xml() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .client
        .get(format!("{}/rss", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let content_type = response
        .headers()
        .get("content-type")
        .expect("Content-Type header missing")
        .to_str()
        .unwrap();

    assert!(content_type.contains("application/rss+xml"));

    let body = response.text().await.expect("Failed to get response body");

    // Verify it's valid XML
    assert!(body.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert!(body.contains("<rss version=\"2.0\""));
    assert!(body.contains("<channel>"));
    assert!(body.contains("<title>CrustyRustacean Dev Blog</title>"));
    assert!(body.contains("</channel>"));
    assert!(body.contains("</rss>"));
}

#[tokio::test]
async fn rss_feed_includes_articles() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Create an article using helper
    app.create_article(
        &token,
        "Test Article for RSS",
        "This should appear in the RSS feed",
        "Article content here",
        vec!["test"],
    )
    .await;

    // Act - Get RSS feed
    let response = app
        .client
        .get(format!("{}/rss", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body = response.text().await.expect("Failed to get response body");

    // Verify the article appears in the feed
    assert!(body.contains("<item>"));
    assert!(body.contains("<title>Test Article for RSS</title>"));
    assert!(body.contains("<description>This should appear in the RSS feed</description>"));
    assert!(
        body.contains("<link>https://crusty-rustacean.com/articles/test-article-for-rss</link>")
    );
    assert!(body.contains("<pubDate>"));
    assert!(body.contains("</item>"));
}

#[tokio::test]
async fn rss_feed_escapes_xml_special_characters() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Create an article with special characters using helper
    app.create_article(
        &token,
        "Test & Special <Characters>",
        "Description with 'quotes' & \"tags\"",
        "Content",
        vec![],
    )
    .await;

    // Act
    let response = app
        .client
        .get(format!("{}/rss", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body = response.text().await.expect("Failed to get response body");

    // Verify XML escaping
    assert!(body.contains("Test &amp; Special &lt;Characters&gt;"));
    assert!(body.contains("Description with &apos;quotes&apos; &amp; &quot;tags&quot;"));
}

#[tokio::test]
async fn rss_feed_returns_empty_feed_when_no_articles() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .client
        .get(format!("{}/rss", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body = response.text().await.expect("Failed to get response body");

    // Should still be valid RSS, just without items
    assert!(body.contains("<channel>"));
    assert!(body.contains("<title>CrustyRustacean Dev Blog</title>"));
    // No <item> tags should be present
    assert!(!body.contains("<item>"));
}
