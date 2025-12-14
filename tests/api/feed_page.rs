// tests/api/feed_page.rs

use crate::helpers::{HtmlResponseValidator, TestUserBuilder, assert_body_contains, spawn_app};
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
    let token = app.register_user_default("feeduser").await;

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

    let body = response.assert_html_response().await;
    assert_body_contains(&body, &["Your Personal Feed"]);
}
