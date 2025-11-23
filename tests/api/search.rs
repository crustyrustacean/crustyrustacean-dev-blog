// tests/api/search.rs

use crate::helpers::{TestArticleBuilder, TestUserBuilder, spawn_app};
use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn test_search_articles_happy_path() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testauthor").await;

    // Create test articles with distinct content
    app.create_article(
        &token,
        "Learning Rust Basics",
        "A guide to Rust fundamentals",
        "Rust is a systems programming language that runs blazingly fast.",
        vec!["rust", "programming"],
    )
    .await;

    app.create_article(
        &token,
        "Advanced Rust Patterns",
        "Deep dive into Rust patterns",
        "Exploring advanced patterns in Rust development.",
        vec!["rust", "advanced"],
    )
    .await;

    app.create_article(
        &token,
        "JavaScript Tips",
        "Modern JavaScript techniques",
        "Learn about async/await in JavaScript.",
        vec!["javascript"],
    )
    .await;

    // Act - Search for "rust"
    let response = app
        .client
        .get(format!("{}/api/search", &app.address))
        .query(&[("q", "rust")])
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response as JSON");

    // Should find 2 articles containing "rust"
    assert!(response_body["results"].is_array());
    let results = response_body["results"].as_array().unwrap();
    assert_eq!(results.len(), 2);

    // Verify the results contain rust-related articles
    assert!(
        results
            .iter()
            .any(|r| r["title"].as_str().unwrap().contains("Rust"))
    );

    // Verify total count
    assert_eq!(response_body["resultsCount"], 2);
}

#[tokio::test]
async fn test_search_articles_no_results() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("author").await;

    app.create_article_simple(&token, "Test Article").await;

    // Act - Search for something that doesn't exist
    let response = app
        .client
        .get(format!("{}/api/search", &app.address))
        .query(&[("q", "nonexistent")])
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response as JSON");

    assert_eq!(response_body["results"].as_array().unwrap().len(), 0);
    assert_eq!(response_body["resultsCount"], 0);
}

#[tokio::test]
async fn test_search_articles_empty_query() {
    // Arrange
    let app = spawn_app().await;

    // Act - Search with empty query
    let response = app
        .client
        .get(format!("{}/api/search", &app.address))
        .query(&[("q", "")])
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should return bad request
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response as JSON");

    assert_eq!(response_body["success"], false);
    assert!(
        response_body["message"]
            .as_str()
            .unwrap()
            .contains("Search query cannot be empty")
    );
}

#[tokio::test]
async fn test_search_articles_missing_query() {
    // Arrange
    let app = spawn_app().await;

    // Act - Search without query parameter
    let response = app
        .client
        .get(format!("{}/api/search", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should return bad request
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response as JSON");

    assert_eq!(response_body["success"], false);
    assert!(
        response_body["message"]
            .as_str()
            .unwrap()
            .contains("Search query is required")
    );
}

#[tokio::test]
async fn test_search_matches_title_description_and_body() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("searchauthor").await;

    // Article with query in title
    app.create_article(
        &token,
        "PostgreSQL Database Tutorial",
        "Learn about databases",
        "This is about MySQL and other systems.",
        vec![],
    )
    .await;

    // Article with query in description
    app.create_article(
        &token,
        "Database Systems",
        "Complete guide to PostgreSQL",
        "This is about MySQL and other systems.",
        vec![],
    )
    .await;

    // Article with query in body
    app.create_article(
        &token,
        "Database Guide",
        "Learn about databases",
        "PostgreSQL is a powerful relational database.",
        vec![],
    )
    .await;

    // Article without the query
    app.create_article(
        &token,
        "Redis Tutorial",
        "Learn about Redis",
        "Redis is an in-memory data store.",
        vec![],
    )
    .await;

    // Act - Search for "PostgreSQL"
    let response = app
        .client
        .get(format!("{}/api/search", &app.address))
        .query(&[("q", "PostgreSQL")])
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response as JSON");

    // Should find 3 articles (title, description, body)
    assert_eq!(response_body["resultsCount"], 3);
}

#[tokio::test]
async fn test_search_case_insensitive() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("author").await;

    app.create_article(
        &token,
        "Rust Programming",
        "Learn RUST",
        "rust is awesome",
        vec![],
    )
    .await;

    // Act - Search with different cases
    let response_lower = app
        .client
        .get(format!("{}/api/search", &app.address))
        .query(&[("q", "rust")])
        .send()
        .await
        .expect("Failed to execute request");

    let response_upper = app
        .client
        .get(format!("{}/api/search", &app.address))
        .query(&[("q", "RUST")])
        .send()
        .await
        .expect("Failed to execute request");

    let response_mixed = app
        .client
        .get(format!("{}/api/search", &app.address))
        .query(&[("q", "RuSt")])
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - All should return the same result
    assert_eq!(response_lower.status(), StatusCode::OK);
    assert_eq!(response_upper.status(), StatusCode::OK);
    assert_eq!(response_mixed.status(), StatusCode::OK);

    let body_lower: Value = response_lower.json().await.unwrap();
    let body_upper: Value = response_upper.json().await.unwrap();
    let body_mixed: Value = response_mixed.json().await.unwrap();

    assert_eq!(body_lower["resultsCount"], 1);
    assert_eq!(body_upper["resultsCount"], 1);
    assert_eq!(body_mixed["resultsCount"], 1);
}

#[tokio::test]
async fn test_search_includes_author_info() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("searchtest").await;

    app.create_article_simple(&token, "Searchable Article")
        .await;

    // Act
    let response = app
        .client
        .get(format!("{}/api/search", &app.address))
        .query(&[("q", "searchable")])
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    let response_body: Value = response.json().await.unwrap();
    let results = response_body["results"].as_array().unwrap();

    assert!(results[0]["author"].is_object());
    assert_eq!(results[0]["author"]["username"], "searchtest");
}
