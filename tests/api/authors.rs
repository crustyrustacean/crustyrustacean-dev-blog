// tests/api/authors.rs

use crate::helpers::spawn_app;
use reqwest::StatusCode;
use serde_json::{Value, json};

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
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_authors_happy_path() {
    // Arrange
    let app = spawn_app().await;

    // Register current user
    let current_user_data = json!({
        "user": {
            "username": "currentuser",
            "email": "current@example.com",
            "password": "securepassword123"
        }
    });

    let current_user_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&current_user_data)
        .send()
        .await
        .expect("Failed to register current user");

    let current_user_body: Value = current_user_response
        .json()
        .await
        .expect("Failed to parse current user response");
    let current_user_token = current_user_body["user"]["token"].as_str().unwrap();

    // Register other users to find
    let authors = vec![
        json!({
            "user": {
                "username": "author1",
                "email": "author1@example.com",
                "password": "securepassword123"
            }
        }),
        json!({
            "user": {
                "username": "author2",
                "email": "author2@example.com",
                "password": "securepassword123"
            }
        }),
    ];

    for author_data in &authors {
        app.client
            .post(format!("{}/api/users", &app.address))
            .header("Content-Type", "application/json")
            .json(author_data)
            .send()
            .await
            .expect("Failed to register author");
    }

    // Act - Get list of authors
    let response = app
        .client
        .get(format!("{}/api/profiles", &app.address))
        .header("Authorization", format!("Bearer {}", current_user_token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response body");

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

    // Register user
    let user_data = json!({
        "user": {
            "username": "searcher",
            "email": "searcher@example.com",
            "password": "securepassword123"
        }
    });

    let user_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user_data)
        .send()
        .await
        .expect("Failed to register user");

    let user_body: Value = user_response
        .json()
        .await
        .expect("Failed to parse user response");
    let token = user_body["user"]["token"].as_str().unwrap();

    // Register searchable authors
    let searchable_authors = vec![
        json!({
            "user": {
                "username": "rustguru",
                "email": "rustguru@example.com",
                "password": "securepassword123"
            }
        }),
        json!({
            "user": {
                "username": "webdev",
                "email": "webdev@example.com",
                "password": "securepassword123"
            }
        }),
        json!({
            "user": {
                "username": "pythonista",
                "email": "pythonista@example.com",
                "password": "securepassword123"
            }
        }),
    ];

    for author_data in &searchable_authors {
        app.client
            .post(format!("{}/api/users", &app.address))
            .header("Content-Type", "application/json")
            .json(author_data)
            .send()
            .await
            .expect("Failed to register author");
    }

    // Act - Search for authors with "rust" in username
    let response = app
        .client
        .get(format!("{}/api/profiles?search=rust", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response body");

    let profiles = response_body["profiles"].as_array().unwrap();
    assert!(profiles.len() >= 1);

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

    // Register user
    let user_data = json!({
        "user": {
            "username": "paginator",
            "email": "paginator@example.com",
            "password": "securepassword123"
        }
    });

    let user_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user_data)
        .send()
        .await
        .expect("Failed to register user");

    let user_body: Value = user_response
        .json()
        .await
        .expect("Failed to parse user response");
    let token = user_body["user"]["token"].as_str().unwrap();

    // Register multiple authors
    for i in 1..=5 {
        let author_data = json!({
            "user": {
                "username": format!("author{}", i),
                "email": format!("author{}@example.com", i),
                "password": "securepassword123"
            }
        });

        app.client
            .post(format!("{}/api/users", &app.address))
            .header("Content-Type", "application/json")
            .json(&author_data)
            .send()
            .await
            .expect("Failed to register author");
    }

    // Act - Get first 3 authors
    let response = app
        .client
        .get(format!("{}/api/profiles?limit=3", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response body");

    let profiles = response_body["profiles"].as_array().unwrap();
    assert_eq!(profiles.len(), 3);
    assert!(response_body["profilesCount"].as_i64().unwrap() >= 3);
}

#[tokio::test]
async fn test_get_authors_shows_follow_status() {
    // Arrange
    let app = spawn_app().await;

    // Register current user
    let current_user_data = json!({
        "user": {
            "username": "follower",
            "email": "follower@example.com",
            "password": "securepassword123"
        }
    });

    let current_user_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&current_user_data)
        .send()
        .await
        .expect("Failed to register current user");

    let current_user_body: Value = current_user_response
        .json()
        .await
        .expect("Failed to parse current user response");
    let current_user_token = current_user_body["user"]["token"].as_str().unwrap();

    // Register author to follow
    let author_data = json!({
        "user": {
            "username": "followme",
            "email": "followme@example.com",
            "password": "securepassword123"
        }
    });

    app.client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&author_data)
        .send()
        .await
        .expect("Failed to register author");

    // Follow the author
    app.client
        .post(format!("{}/api/profiles/followme/follow", &app.address))
        .header("Authorization", format!("Bearer {}", current_user_token))
        .send()
        .await
        .expect("Failed to follow author");

    // Act - Get authors list
    let response = app
        .client
        .get(format!("{}/api/profiles", &app.address))
        .header("Authorization", format!("Bearer {}", current_user_token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response body");

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
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_authors_page_with_authentication() {
    // Arrange
    let app = spawn_app().await;

    // Register user
    let user_data = json!({
        "user": {
            "username": "browserpages",
            "email": "browser@example.com",
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

    // Act - Access authors page with authentication
    let response = app
        .client
        .get(format!("{}/profiles", &app.address))
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
    assert!(body.contains("Find Authors")); // Should contain the authors discovery title
}
