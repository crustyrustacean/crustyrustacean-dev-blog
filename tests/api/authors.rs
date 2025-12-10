// tests/api/authors.rs

use crate::helpers::{TestFixture, TestUserBuilder, spawn_app};
use crate::{assert_status, bearer_request, parse_json};
use reqwest::StatusCode;
use serde_json::Value;

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
async fn test_get_authors_only_shows_authors_not_subscribers() {
    // Arrange - Create a fixture with an admin and users with different roles
    let fixture = TestFixture::new()
        .await
        .with_user_default("admin") // First user becomes admin
        .await
        .with_user_default("author1")
        .await
        .with_user_default("subscriber1") // This user stays as subscriber
        .await;

    // Promote author1 to author role
    let fixture = fixture.promote_to_author("author1").await;

    let admin_token = fixture.get_token("admin");

    // Act - Get list of profiles
    let response = bearer_request!(
        get & fixture.app,
        format!("{}/api/profiles", &fixture.app.address),
        &admin_token
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    let profiles = response_body["profiles"].as_array().unwrap();
    let usernames: Vec<&str> = profiles
        .iter()
        .map(|p| p["username"].as_str().unwrap())
        .collect();

    // author1 should be shown (has author role)
    assert!(
        usernames.contains(&"author1"),
        "author1 should be visible in profiles"
    );

    // subscriber1 should NOT be shown (has subscriber role)
    assert!(
        !usernames.contains(&"subscriber1"),
        "subscriber1 should NOT be visible in profiles (has subscriber role)"
    );

    // admin should NOT be shown (current user is excluded from results)
    assert!(
        !usernames.contains(&"admin"),
        "admin (current user) should be excluded from results"
    );
}

#[tokio::test]
async fn test_get_authors_happy_path() {
    // Arrange - Create fixture with admin and authors
    let fixture = TestFixture::new()
        .await
        .with_user_default("currentuser") // First user becomes admin
        .await
        .with_user_default("author1")
        .await
        .with_user_default("author2")
        .await;

    // Promote authors
    let fixture = fixture.promote_to_author("author1").await;
    let fixture = fixture.promote_to_author("author2").await;

    let current_user_token = fixture.get_token("currentuser");

    // Act - Get list of authors
    let response = bearer_request!(
        get & fixture.app,
        format!("{}/api/profiles", &fixture.app.address),
        &current_user_token
    )
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
    let fixture = TestFixture::new()
        .await
        .with_user_default("searcher") // First user becomes admin
        .await
        .with_user_default("rustguru")
        .await
        .with_user_default("webdev")
        .await
        .with_user_default("pythonista")
        .await;

    // Promote all to author so they can be searched
    let fixture = fixture.promote_to_author("rustguru").await;
    let fixture = fixture.promote_to_author("webdev").await;
    let fixture = fixture.promote_to_author("pythonista").await;

    let token = fixture.get_token("searcher");

    // Act - Search for authors with "rust" in username
    let response = bearer_request!(
        get & fixture.app,
        format!("{}/api/profiles?search=rust", &fixture.app.address),
        &token
    )
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
async fn test_get_authors_search_excludes_subscribers() {
    // Arrange - Create users where some match search but are subscribers
    let fixture = TestFixture::new()
        .await
        .with_user_default("searcher") // First user becomes admin
        .await
        .with_user_default("rustauthor") // Will be promoted
        .await
        .with_user_default("rustsubscriber") // Will stay as subscriber
        .await;

    // Only promote rustauthor
    let fixture = fixture.promote_to_author("rustauthor").await;

    let token = fixture.get_token("searcher");

    // Act - Search for "rust"
    let response = bearer_request!(
        get & fixture.app,
        format!("{}/api/profiles?search=rust", &fixture.app.address),
        &token
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    let profiles = response_body["profiles"].as_array().unwrap();
    let usernames: Vec<&str> = profiles
        .iter()
        .map(|p| p["username"].as_str().unwrap())
        .collect();

    // rustauthor should be found (author role)
    assert!(
        usernames.contains(&"rustauthor"),
        "rustauthor should be found in search"
    );

    // rustsubscriber should NOT be found (subscriber role, even though username matches)
    assert!(
        !usernames.contains(&"rustsubscriber"),
        "rustsubscriber should NOT be found (subscriber role)"
    );
}

#[tokio::test]
async fn test_get_authors_with_pagination() {
    // Arrange
    let fixture = TestFixture::new()
        .await
        .with_user_default("paginator") // First user becomes admin
        .await
        .with_user_default("author1")
        .await
        .with_user_default("author2")
        .await
        .with_user_default("author3")
        .await
        .with_user_default("author4")
        .await
        .with_user_default("author5")
        .await;

    // Promote all to author
    let fixture = fixture.promote_to_author("author1").await;
    let fixture = fixture.promote_to_author("author2").await;
    let fixture = fixture.promote_to_author("author3").await;
    let fixture = fixture.promote_to_author("author4").await;
    let fixture = fixture.promote_to_author("author5").await;

    let token = fixture.get_token("paginator");

    // Act - Get first 3 authors
    let response = bearer_request!(
        get & fixture.app,
        format!("{}/api/profiles?limit=3", &fixture.app.address),
        &token
    )
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
    let fixture = TestFixture::new()
        .await
        .with_user_default("follower") // First user becomes admin
        .await
        .with_user_default("followme")
        .await;

    // Promote followme to author so they appear in the list
    let fixture = fixture.promote_to_author("followme").await;

    let current_user_token = fixture.get_token("follower");

    // Follow the author
    fixture
        .app
        .client
        .post(format!(
            "{}/api/profiles/followme/follow",
            &fixture.app.address
        ))
        .header("Authorization", format!("Bearer {}", current_user_token))
        .send()
        .await
        .expect("Failed to follow author");

    // Act - Get authors list
    let response = bearer_request!(
        get & fixture.app,
        format!("{}/api/profiles", &fixture.app.address),
        &current_user_token
    )
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

#[tokio::test]
async fn test_get_authors_empty_when_no_authors_exist() {
    // Arrange - Only create subscribers (no authors)
    let app = spawn_app().await;
    let current_user_token = app.register_user_default("subscriber1").await;
    app.register_user_default("subscriber2").await;
    app.register_user_default("subscriber3").await;

    // Act - Get list of authors (none exist, only subscribers)
    let response = bearer_request!(
        get & app,
        format!("{}/api/profiles", &app.address),
        &current_user_token
    )
    .expect("Failed to execute request");

    // Assert - Should return empty list (first user is admin, but excluded as current user)
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    let profiles = response_body["profiles"].as_array().unwrap();

    // Should be empty - no authors (only subscribers, and admin is excluded as current user)
    assert!(
        profiles.is_empty(),
        "Should return empty list when no authors exist (only subscribers)"
    );
}
