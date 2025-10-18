// tests/api/tags.rs

use crate::helpers::spawn_app;
use serde_json::json;

#[tokio::test]
async fn get_tags_returns_empty_list_when_no_tags_exist() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .client
        .get(format!("{}/api/tags", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    assert_eq!(body["tags"], json!([]));
}

#[tokio::test]
async fn get_tags_returns_all_unique_tags() {
    // Arrange
    let app = spawn_app().await;

    // Register a user and create articles with tags
    let register_body = json!({
        "user": {
            "username": "testuser",
            "email": "test@example.com",
            "password": "password123"
        }
    });

    let register_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .json(&register_body)
        .send()
        .await
        .expect("Failed to execute request.");

    let register_json: serde_json::Value = register_response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    let token = register_json["user"]["token"]
        .as_str()
        .expect("Token not found in response");

    // Create article with tags: rust, webdev
    let article1_body = json!({
        "article": {
            "title": "How to learn Rust",
            "description": "Ever wonder how?",
            "body": "It takes time",
            "tagList": ["rust", "webdev"]
        }
    });

    app.client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&article1_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Create another article with tags: rust, programming
    let article2_body = json!({
        "article": {
            "title": "Rust patterns",
            "description": "Common patterns",
            "body": "Here are some patterns",
            "tagList": ["rust", "programming"]
        }
    });

    app.client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&article2_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Act
    let response = app
        .client
        .get(format!("{}/api/tags", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    let tags = body["tags"].as_array().expect("tags should be an array");

    // Should have 3 unique tags
    assert_eq!(tags.len(), 3);

    // Convert to strings for easier assertion
    let tag_strings: Vec<String> = tags
        .iter()
        .map(|t| t.as_str().unwrap().to_string())
        .collect();

    assert!(tag_strings.contains(&"rust".to_string()));
    assert!(tag_strings.contains(&"webdev".to_string()));
    assert!(tag_strings.contains(&"programming".to_string()));
}

#[tokio::test]
async fn get_tags_returns_tags_in_alphabetical_order() {
    // Arrange
    let app = spawn_app().await;

    // Register a user
    let register_body = json!({
        "user": {
            "username": "testuser",
            "email": "test@example.com",
            "password": "password123"
        }
    });

    let register_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .json(&register_body)
        .send()
        .await
        .expect("Failed to execute request.");

    let register_json: serde_json::Value = register_response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    let token = register_json["user"]["token"]
        .as_str()
        .expect("Token not found in response");

    // Create article with tags in non-alphabetical order
    let article_body = json!({
        "article": {
            "title": "Test article",
            "description": "Testing tags",
            "body": "Body content",
            "tagList": ["zebra", "apple", "mango", "banana"]
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
        .get(format!("{}/api/tags", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    let tags = body["tags"].as_array().expect("tags should be an array");
    let tag_strings: Vec<String> = tags
        .iter()
        .map(|t| t.as_str().unwrap().to_string())
        .collect();

    // Should be in alphabetical order
    assert_eq!(tag_strings, vec!["apple", "banana", "mango", "zebra"]);
}
