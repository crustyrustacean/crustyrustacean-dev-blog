// tests/api/feed_page.rs

use crate::helpers::spawn_app;
use reqwest::StatusCode;
use serde_json::{Value, json};

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

    // Register user
    let user_data = json!({
        "user": {
            "username": "feeduser",
            "email": "feeduser@example.com",
            "password": "securepassword123"
        }
    });

    let registration_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user_data)
        .send()
        .await
        .expect("Failed to register user");

    let registration_body: Value = registration_response
        .json()
        .await
        .expect("Failed to parse registration response");

    let token = registration_body["user"]["token"].as_str().unwrap();

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