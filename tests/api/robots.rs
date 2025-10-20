// tests/api/robots.rs

use crate::helpers::spawn_app;
use reqwest::StatusCode;

#[tokio::test]
async fn robots_txt_returns_valid_content() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .client
        .get(format!("{}/robots.txt", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    // Verify content type
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    assert!(
        content_type.contains("text/plain"),
        "Expected text/plain content type, got: {}",
        content_type
    );

    // Verify content
    let body = response.text().await.expect("Failed to get response body");

    // Check for essential robots.txt directives
    assert!(
        body.contains("User-agent:"),
        "Should contain User-agent directive"
    );
    assert!(
        body.contains("Disallow:"),
        "Should contain Disallow directive"
    );

    // Verify admin pages are disallowed
    assert!(body.contains("Disallow: /admin"), "Should disallow /admin");
    assert!(body.contains("Disallow: /login"), "Should disallow /login");
    assert!(body.contains("Disallow: /api/"), "Should disallow /api/");

    // Verify public content is allowed
    assert!(body.contains("Allow: /articles"), "Should allow /articles");
    assert!(body.contains("Allow: /rss"), "Should allow /rss");

    // Verify sitemap reference
    assert!(
        body.contains("Sitemap:"),
        "Should contain Sitemap directive"
    );
}
