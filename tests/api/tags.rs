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

// Add these tests to tests/api/tags.rs

#[tokio::test]
async fn update_tag_happy_path() {
    // Arrange
    let app = spawn_app().await;

    // Register a user
    let register_body = json!({
        "user": {
            "username": "tagupdater",
            "email": "tagupdater@example.com",
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

    // Create article with a tag
    let article_body = json!({
        "article": {
            "title": "Test article",
            "description": "Testing tag update",
            "body": "Body content",
            "tagList": ["oldtag"]
        }
    });

    app.client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&article_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Act - Update the tag
    let update_body = json!({
        "tag": {
            "name": "newtag"
        }
    });

    let response = app
        .client
        .put(format!("{}/api/tags/oldtag", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&update_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    assert_eq!(body["tag"], "newtag");

    // Verify the old tag no longer exists and new tag exists
    let tags_response = app
        .client
        .get(format!("{}/api/tags", &app.address))
        .send()
        .await
        .expect("Failed to get tags");

    let tags_body: serde_json::Value = tags_response
        .json()
        .await
        .expect("Failed to parse tags response");

    let tag_strings: Vec<String> = tags_body["tags"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.as_str().unwrap().to_string())
        .collect();

    assert!(!tag_strings.contains(&"oldtag".to_string()));
    assert!(tag_strings.contains(&"newtag".to_string()));
}

#[tokio::test]
async fn update_tag_requires_authentication() {
    // Arrange
    let app = spawn_app().await;

    let update_body = json!({
        "tag": {
            "name": "newtag"
        }
    });

    // Act - Try to update without authentication
    let response = app
        .client
        .put(format!("{}/api/tags/oldtag", &app.address))
        .json(&update_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn update_nonexistent_tag() {
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

    // Act - Try to update a tag that doesn't exist
    let update_body = json!({
        "tag": {
            "name": "newtag"
        }
    });

    let response = app
        .client
        .put(format!("{}/api/tags/nonexistent", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&update_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn update_tag_with_duplicate_name() {
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

    // Create article with two tags
    let article_body = json!({
        "article": {
            "title": "Test article",
            "description": "Testing",
            "body": "Body content",
            "tagList": ["tag1", "tag2"]
        }
    });

    app.client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&article_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Act - Try to rename tag1 to tag2 (which already exists)
    let update_body = json!({
        "tag": {
            "name": "tag2"
        }
    });

    let response = app
        .client
        .put(format!("{}/api/tags/tag1", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&update_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert - Should return conflict
    assert_eq!(response.status().as_u16(), 409);
}

#[tokio::test]
async fn update_tag_with_empty_name() {
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

    // Create article with a tag
    let article_body = json!({
        "article": {
            "title": "Test article",
            "description": "Testing",
            "body": "Body content",
            "tagList": ["testtag"]
        }
    });

    app.client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&article_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Act - Try to update with empty name
    let update_body = json!({
        "tag": {
            "name": ""
        }
    });

    let response = app
        .client
        .put(format!("{}/api/tags/testtag", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&update_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn update_tag_with_invalid_payload() {
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

    // Act - Try to update with missing "tag" wrapper
    let update_body = json!({
        "name": "newtag"
    });

    let response = app
        .client
        .put(format!("{}/api/tags/sometag", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&update_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn delete_tag_happy_path() {
    // Arrange
    let app = spawn_app().await;

    // Register a user
    let register_body = json!({
        "user": {
            "username": "tagdeleter",
            "email": "tagdeleter@example.com",
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

    // Create article with tags
    let article_body = json!({
        "article": {
            "title": "Test article",
            "description": "Testing tag deletion",
            "body": "Body content",
            "tagList": ["tagtokeep", "tagtodelete"]
        }
    });

    app.client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&article_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Act - Delete one tag
    let response = app
        .client
        .delete(format!("{}/api/tags/tagtodelete", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 204);

    // Verify the tag no longer exists
    let tags_response = app
        .client
        .get(format!("{}/api/tags", &app.address))
        .send()
        .await
        .expect("Failed to get tags");

    let tags_body: serde_json::Value = tags_response
        .json()
        .await
        .expect("Failed to parse tags response");

    let tag_strings: Vec<String> = tags_body["tags"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.as_str().unwrap().to_string())
        .collect();

    assert!(!tag_strings.contains(&"tagtodelete".to_string()));
    assert!(tag_strings.contains(&"tagtokeep".to_string()));
}

#[tokio::test]
async fn delete_tag_requires_authentication() {
    // Arrange
    let app = spawn_app().await;

    // Act - Try to delete without authentication
    let response = app
        .client
        .delete(format!("{}/api/tags/sometag", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn delete_nonexistent_tag() {
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

    // Act - Try to delete a tag that doesn't exist
    let response = app
        .client
        .delete(format!("{}/api/tags/nonexistent", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn delete_tag_removes_from_articles() {
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

    // Create article with tags
    let article_body = json!({
        "article": {
            "title": "Article with tags",
            "description": "Testing",
            "body": "Content",
            "tagList": ["keep", "remove"]
        }
    });

    let create_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&article_body)
        .send()
        .await
        .expect("Failed to execute request.");

    let create_json: serde_json::Value = create_response
        .json()
        .await
        .expect("Failed to parse response");

    let slug = create_json["article"]["slug"].as_str().unwrap();

    // Delete the tag
    let delete_response = app
        .client
        .delete(format!("{}/api/tags/remove", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(delete_response.status().as_u16(), 204);

    // Verify the article no longer has the deleted tag
    let article_response = app
        .client
        .get(format!("{}/api/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to get article");

    let article_json: serde_json::Value = article_response
        .json()
        .await
        .expect("Failed to parse article response");

    let tag_list = article_json["article"]["tagList"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.as_str().unwrap().to_string())
        .collect::<Vec<String>>();

    assert!(!tag_list.contains(&"remove".to_string()));
    assert!(tag_list.contains(&"keep".to_string()));
}

#[tokio::test]
async fn update_tag_same_name_is_idempotent() {
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

    // Create article with a tag
    let article_body = json!({
        "article": {
            "title": "Test article",
            "description": "Testing",
            "body": "Body content",
            "tagList": ["sametag"]
        }
    });

    app.client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&article_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Act - Update tag to the same name
    let update_body = json!({
        "tag": {
            "name": "sametag"
        }
    });

    let response = app
        .client
        .put(format!("{}/api/tags/sametag", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&update_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert - Should succeed (idempotent operation)
    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    assert_eq!(body["tag"], "sametag");
}
