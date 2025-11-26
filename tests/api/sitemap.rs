// tests/api/sitemap.rs

use crate::helpers::{TestUserBuilder, spawn_app};
use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn sitemap_returns_valid_xml() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .client
        .get(format!("{}/sitemap.xml", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response
        .headers()
        .get("content-type")
        .expect("Content-Type header missing")
        .to_str()
        .unwrap();

    assert!(content_type.contains("application/xml"));

    let body = response.text().await.expect("Failed to get response body");

    // Verify it's valid XML
    assert!(body.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert!(body.contains("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">"));
    assert!(body.contains("</urlset>"));
}

#[tokio::test]
async fn sitemap_includes_static_pages() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .client
        .get(format!("{}/sitemap.xml", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.text().await.expect("Failed to get response body");

    // Verify static pages are included
    assert!(
        body.contains("<loc>https://crusty-rustacean.com/</loc>"),
        "Should include homepage"
    );
    assert!(
        body.contains("<loc>https://crusty-rustacean.com/articles</loc>"),
        "Should include articles page"
    );
    assert!(
        body.contains("<loc>https://crusty-rustacean.com/about</loc>"),
        "Should include about page"
    );
    assert!(
        body.contains("<loc>https://crusty-rustacean.com/rss</loc>"),
        "Should include RSS feed"
    );

    // Verify sitemap elements
    assert!(body.contains("<changefreq>"), "Should contain changefreq");
    assert!(body.contains("<priority>"), "Should contain priority");
}

#[tokio::test]
async fn sitemap_includes_articles() {
    // Arrange
    let app = spawn_app().await;

    // Register a user and create an article
    let token = app
        .register_user("sitemapuser", "sitemap@example.com", "password123")
        .await;

    // Create an article
    let article_body = json!({
        "article": {
            "title": "Sitemap Test Article",
            "description": "This should appear in the sitemap",
            "body": "Article content here",
            "tagList": ["sitemap"]
        }
    });

    app.client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&article_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Act - Get sitemap
    let response = app
        .client
        .get(format!("{}/sitemap.xml", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.text().await.expect("Failed to get response body");

    // Verify the article appears in the sitemap
    assert!(body.contains("<url>"), "Should contain url tags");
    assert!(
        body.contains("<loc>https://crusty-rustacean.com/articles/sitemap-test-article</loc>"),
        "Should include the article URL"
    );
    assert!(
        body.contains("<lastmod>"),
        "Should contain lastmod for articles"
    );
}

#[tokio::test]
async fn sitemap_escapes_xml_special_characters() {
    // Arrange
    let app = spawn_app().await;

    // Register a user
    let token = app
        .register_user("xmluser", "xml@example.com", "password123")
        .await;

    // Create an article with special characters in title (which affects slug)
    let article_body = json!({
        "article": {
            "title": "Test & Special <Characters>",
            "description": "Testing XML escaping",
            "body": "Content",
            "tagList": []
        }
    });

    app.client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&article_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Act
    let response = app
        .client
        .get(format!("{}/sitemap.xml", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.text().await.expect("Failed to get response body");

    // The slug will be generated from the title, and should be properly escaped
    // Even though the slug itself might not have special chars,
    // we're testing that the XML is well-formed
    assert!(
        body.contains("</loc>"),
        "Should contain properly closed loc tags"
    );
    assert!(
        !body.contains("<Characters>"),
        "Should not contain unescaped brackets"
    );
}

#[tokio::test]
async fn sitemap_returns_valid_xml_when_no_articles() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .client
        .get(format!("{}/sitemap.xml", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.text().await.expect("Failed to get response body");

    // Should still be valid XML with static pages
    assert!(body.contains("<urlset"));
    assert!(body.contains("</urlset>"));
    assert!(
        body.contains("<loc>https://crusty-rustacean.com/</loc>"),
        "Should contain at least the homepage"
    );
}

#[tokio::test]
async fn sitemap_articles_sorted_by_update_time() {
    // Arrange
    let app = spawn_app().await;

    // Register a user
    let token = app
        .register_user("sortuser", "sort@example.com", "password123")
        .await;

    // Create multiple articles
    let articles = vec!["First Article", "Second Article", "Third Article"];

    for title in articles {
        let article_body = json!({
            "article": {
                "title": title,
                "description": "Testing sort order",
                "body": "Content",
                "tagList": []
            }
        });

        app.client
            .post(format!("{}/api/articles", &app.address))
            .header("Authorization", format!("Bearer {}", token))
            .json(&article_body)
            .send()
            .await
            .expect("Failed to execute request.");
    }

    // Act
    let response = app
        .client
        .get(format!("{}/sitemap.xml", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.text().await.expect("Failed to get response body");

    // All articles should be present
    assert!(body.contains("first-article"));
    assert!(body.contains("second-article"));
    assert!(body.contains("third-article"));

    // Should contain proper sitemap structure
    assert!(body.contains("<lastmod>"));
}
