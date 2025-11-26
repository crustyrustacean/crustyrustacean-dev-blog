// tests/api/feed_page.rs

use crate::helpers::{TestUserBuilder, spawn_app};
use reqwest::StatusCode;

#[tokio::test]
async fn test_feed_page_requires_authentication() {
    // Arrange
    let app = spawn_app().await;

    // Act - Try to access feed page without authentication
    let response = app
        .client
        .get(format!("{}/feed", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_feed_page_with_authentication() {
    // Arrange
    let app = spawn_app().await;

    // Register user using helper
    let token = app
        .register_user("feeduser", "feeduser@example.com", "securepassword123")
        .await;

    // Act - Access feed page with authentication
    let response = app
        .client
        .get(format!("{}/feed", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should return HTML page (200 OK)
    assert_eq!(response.status(), StatusCode::OK);

    // Check that it returns HTML content
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Should be HTML, not JSON
    assert!(content_type.contains("text/html") || content_type.is_empty());

    // Check that the response body contains HTML
    let body = response.text().await.expect("Failed to get response body");
    assert!(body.contains("<html") || body.contains("<!DOCTYPE html"));
    assert!(body.contains("Your Personal Feed")); // Should contain the feed title
}
