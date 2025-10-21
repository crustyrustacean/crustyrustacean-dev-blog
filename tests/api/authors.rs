// tests/api/authors.rs

use crate::helpers::{spawn_app, TestUserBuilder};
use crate::{assert_status, bearer_request, parse_json};
use reqwest::StatusCode;
use serde_json::{json, Value};

#[tokio::test]
async fn test_get_authors_requires_authentication() {
    // Arrange
    let app = spawn_app().await;

    // Act - Try to get authors without authentication
    let response = app
        .client
        .get(format!("{}/api/profiles", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_authors_happy_path() {
    // Arrange
    let app = spawn_app().await;
    let current_user_token = app.register_user_default("currentuser").await;

    // Register other users to find
    app.register_user_default("author1").await;
    app.register_user_default("author2").await;

    // Act - Get list of authors
    let response = bearer_request!(get &app,
        format!("{}/api/profiles", &app.address),
        &current_user_token
    )
    .await
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    assert!(response_body["profiles"].is_array());
    let profiles = response_body["profiles"].as_array().unwrap();

    // Should contain at least the 2 authors (excluding current user)
    assert!(profiles.len() >= 2);

    // Check profile structure
    for profile in profiles {
        assert!(profile["username"].is_string());
        assert!(profile["following"].is_boolean());
        // Bio and image can be null
    }
}

#[tokio::test]
async fn test_get_authors_with_search() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("searcher").await;

    // Register searchable authors
    app.register_user("rustguru", "rustguru@example.com", "securepassword123")
        .await;
    app.register_user_default("webdev").await;
    app.register_user_default("pythonista").await;

    // Act - Search for authors with "rust" in username
    let response = bearer_request!(get &app,
        format!("{}/api/profiles?search=rust", &app.address),
        &token
    )
    .await
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    let profiles = response_body["profiles"].as_array().unwrap();
    assert!(!profiles.is_empty());

    // Should contain rustguru
    let usernames: Vec<&str> = profiles
        .iter()
        .map(|p| p["username"].as_str().unwrap())
        .collect();
    assert!(usernames.contains(&"rustguru"));
}

#[tokio::test]
async fn test_get_authors_with_pagination() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("paginator").await;

    // Register multiple authors
    for i in 1..=5 {
        app.register_user(
            &format!("author{}", i),
            &format!("author{}@example.com", i),
            "securepassword123",
        )
        .await;
    }

    // Act - Get first 3 authors
    let response = bearer_request!(get &app,
        format!("{}/api/profiles?limit=3", &app.address),
        &token
    )
    .await
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    let profiles = response_body["profiles"].as_array().unwrap();
    assert_eq!(profiles.len(), 3);
    assert!(response_body["profilesCount"].as_i64().unwrap() >= 3);
}

#[tokio::test]
async fn test_get_authors_shows_follow_status() {
    // Arrange
    let app = spawn_app().await;
    let current_user_token = app.register_user_default("follower").await;

    // Register author to follow
    app.register_user("followme", "followme@example.com", "securepassword123")
        .await;

    // Follow the author
    app.client
        .post(format!("{}/api/profiles/followme/follow", &app.address))
        .header("Authorization", format!("Bearer {}", current_user_token))
        .send()
        .await
        .expect("Failed to follow author");

    // Act - Get authors list
    let response = bearer_request!(get &app,
        format!("{}/api/profiles", &app.address),
        &current_user_token
    )
    .await
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    let profiles = response_body["profiles"].as_array().unwrap();

    // Find the followed author
    let followed_author = profiles
        .iter()
        .find(|p| p["username"] == "followme")
        .expect("Should find followed author");

    assert_eq!(followed_author["following"], true);
}

#[tokio::test]
async fn test_authors_page_requires_authentication() {
    // Arrange
    let app = spawn_app().await;

    // Act - Try to access authors page without authentication
    let response = app
        .client
        .get(format!("{}/profiles", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_authors_page_with_authentication() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("browserpages").await;

    // Act - Access authors page with authentication
    let response = app
        .client
        .get(format!("{}/profiles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should return HTML page (200 OK)
    assert_status!(response, StatusCode::OK);

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
    assert!(body.contains("Find Authors")); // Should contain the authors discovery title
}
