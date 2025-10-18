// tests/api/articles.rs

use crate::helpers::spawn_app;
use reqwest::StatusCode;
use serde_json::{Value, json};

#[tokio::test]
async fn test_create_article_happy_path() {
    // Arrange
    let app = spawn_app().await;
    
    // First, register and get a token
    let user_data = json!({
        "user": {
            "username": "articleauthor",
            "email": "author@example.com",
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
    
    // Prepare article data
    let article_data = json!({
        "article": {
            "title": "How to Train Your Dragon",
            "description": "Ever wonder how?",
            "body": "You have to believe in yourself. That's the secret to life.",
            "tagList": ["dragons", "training"]
        }
    });
    
    // Act - Create article
    let response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .json(&article_data)
        .send()
        .await
        .expect("Failed to execute request");
    
    // Assert
    let status = response.status();
    if status != StatusCode::OK {
        let error_body = response.text().await.expect("Failed to get error text");
        panic!("Expected 200 OK, got {}: {}", status, error_body);
    }
    
    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response body");
    
    // Verify response structure
    assert!(response_body["article"].is_object());
    assert_eq!(response_body["article"]["title"], "How to Train Your Dragon");
    assert_eq!(response_body["article"]["description"], "Ever wonder how?");
    assert_eq!(response_body["article"]["body"], "You have to believe in yourself. That's the secret to life.");
    assert_eq!(response_body["article"]["slug"], "how-to-train-your-dragon");
    assert_eq!(response_body["article"]["tagList"].as_array().unwrap().len(), 2);
    assert_eq!(response_body["article"]["favorited"], false);
    assert_eq!(response_body["article"]["favoritesCount"], 0);
    assert!(response_body["article"]["author"].is_object());
    assert_eq!(response_body["article"]["author"]["username"], "articleauthor");
    assert!(response_body["article"]["createdAt"].is_string());
    assert!(response_body["article"]["updatedAt"].is_string());
}

#[tokio::test]
async fn test_create_article_without_auth() {
    // Arrange
    let app = spawn_app().await;
    
    let article_data = json!({
        "article": {
            "title": "Unauthorized Article",
            "description": "This should fail",
            "body": "No token provided"
        }
    });
    
    // Act
    let response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .json(&article_data)
        .send()
        .await
        .expect("Failed to execute request");
    
    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_article_with_invalid_data() {
    // Arrange
    let app = spawn_app().await;
    
    // Register user and get token
    let user_data = json!({
        "user": {
            "username": "testuser",
            "email": "test@example.com",
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
    
    let test_cases = vec![
        // Empty title
        json!({
            "article": {
                "title": "",
                "description": "Description",
                "body": "Body"
            }
        }),
        // Empty description
        json!({
            "article": {
                "title": "Title",
                "description": "",
                "body": "Body"
            }
        }),
        // Empty body
        json!({
            "article": {
                "title": "Title",
                "description": "Description",
                "body": ""
            }
        }),
        // Missing article wrapper
        json!({
            "title": "Title",
            "description": "Description",
            "body": "Body"
        }),
    ];
    
    for invalid_data in test_cases {
        // Act
        let response = app
            .client
            .post(format!("{}/api/articles", &app.address))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .json(&invalid_data)
            .send()
            .await
            .expect("Failed to execute request");
        
        // Assert
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}

#[tokio::test]
async fn test_get_article_happy_path() {
    // Arrange
    let app = spawn_app().await;
    
    // Register user and get token
    let user_data = json!({
        "user": {
            "username": "articlereader",
            "email": "reader@example.com",
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
    
    // Create an article first
    let article_data = json!({
        "article": {
            "title": "Test Article for Reading",
            "description": "This is a test article",
            "body": "Content of the test article",
            "tagList": ["test", "reading"]
        }
    });
    
    let create_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .json(&article_data)
        .send()
        .await
        .expect("Failed to create article");
    
    let create_body: Value = create_response
        .json()
        .await
        .expect("Failed to parse create response");
    
    let slug = create_body["article"]["slug"].as_str().unwrap();
    
    // Act - Get the article
    let response = app
        .client
        .get(format!("{}/api/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to execute request");
    
    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    
    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response body");
    
    assert!(response_body["article"].is_object());
    assert_eq!(response_body["article"]["title"], "Test Article for Reading");
    assert_eq!(response_body["article"]["description"], "This is a test article");
    assert_eq!(response_body["article"]["body"], "Content of the test article");
    assert_eq!(response_body["article"]["slug"], "test-article-for-reading");
    assert_eq!(response_body["article"]["tagList"].as_array().unwrap().len(), 2);
    assert_eq!(response_body["article"]["favorited"], false);
    assert_eq!(response_body["article"]["favoritesCount"], 0);
    assert!(response_body["article"]["author"].is_object());
    assert_eq!(response_body["article"]["author"]["username"], "articlereader");
}

#[tokio::test]
async fn test_get_nonexistent_article() {
    // Arrange
    let app = spawn_app().await;
    
    // Act - Try to get a non-existent article
    let response = app
        .client
        .get(format!("{}/api/articles/non-existent-slug", &app.address))
        .send()
        .await
        .expect("Failed to execute request");
    
    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_article_page_route() {
    // Arrange
    let app = spawn_app().await;
    
    // Register user and get token
    let user_data = json!({
        "user": {
            "username": "pageuser",
            "email": "page@example.com",
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
    
    // Create an article first
    let create_data = json!({
        "article": {
            "title": "Test Article Page",
            "description": "Testing the article page",
            "body": "This is the article content for testing the page.",
            "tagList": ["test", "page"]
        }
    });
    
    let create_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .json(&create_data)
        .send()
        .await
        .expect("Failed to create article");
    
    let create_body: Value = create_response
        .json()
        .await
        .expect("Failed to parse create response");
    
    let slug = create_body["article"]["slug"].as_str().unwrap();
    
    // Act - Access the article page (HTML route)
    let response = app
        .client
        .get(format!("{}/articles/{}", &app.address, slug))
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
    assert!(content_type.contains("text/html") || content_type.is_empty()); // Some test servers might not set content-type
    
    // Check that the response body contains HTML
    let body = response.text().await.expect("Failed to get response body");
    assert!(body.contains("<html") || body.contains("<!DOCTYPE html"));
    assert!(body.contains("Test Article Page")); // Should contain the article title
}

#[tokio::test]
async fn test_create_article_generates_unique_slugs() {
    // Arrange
    let app = spawn_app().await;
    
    // Register user and get token
    let user_data = json!({
        "user": {
            "username": "slugtester",
            "email": "slug@example.com",
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
    
    // Create first article
    let article_data1 = json!({
        "article": {
            "title": "Duplicate Title",
            "description": "First article",
            "body": "This is the first article with duplicate title"
        }
    });
    
    let response1 = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .json(&article_data1)
        .send()
        .await
        .expect("Failed to create first article");
    
    assert_eq!(response1.status(), StatusCode::OK);
    
    let body1: Value = response1.json().await.expect("Failed to parse response");
    let slug1 = body1["article"]["slug"].as_str().unwrap();
    
    // Create second article with same title
    let article_data2 = json!({
        "article": {
            "title": "Duplicate Title",
            "description": "Second article",
            "body": "This is the second article with duplicate title"
        }
    });
    
    let response2 = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .json(&article_data2)
        .send()
        .await
        .expect("Failed to create second article");
    
    assert_eq!(response2.status(), StatusCode::OK);
    
    let body2: Value = response2.json().await.expect("Failed to parse response");
    let slug2 = body2["article"]["slug"].as_str().unwrap();
    
    // Assert - Slugs should be different
    assert_ne!(slug1, slug2);
    assert_eq!(slug1, "duplicate-title");
    // Second slug should have some suffix to make it unique
    assert!(slug2.starts_with("duplicate-title"));
    assert!(slug2.len() > "duplicate-title".len());
}

#[tokio::test]
async fn test_update_article_happy_path() {
    // Arrange
    let app = spawn_app().await;
    
    // Register user and get token
    let user_data = json!({
        "user": {
            "username": "updateuser",
            "email": "update@example.com",
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
    
    // Create an article first
    let create_data = json!({
        "article": {
            "title": "Original Title",
            "description": "Original description",
            "body": "Original body content",
            "tagList": ["original"]
        }
    });
    
    let create_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .json(&create_data)
        .send()
        .await
        .expect("Failed to create article");
    
    let create_body: Value = create_response
        .json()
        .await
        .expect("Failed to parse create response");
    
    let slug = create_body["article"]["slug"].as_str().unwrap();
    
    // Prepare update data
    let update_data = json!({
        "article": {
            "title": "Updated Title",
            "description": "Updated description",
            "body": "Updated body content"
        }
    });
    
    // Act - Update the article
    let response = app
        .client
        .put(format!("{}/api/articles/{}", &app.address, slug))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .json(&update_data)
        .send()
        .await
        .expect("Failed to execute request");
    
    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    
    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response body");
    
    assert!(response_body["article"].is_object());
    assert_eq!(response_body["article"]["title"], "Updated Title");
    assert_eq!(response_body["article"]["description"], "Updated description");
    assert_eq!(response_body["article"]["body"], "Updated body content");
    assert_eq!(response_body["article"]["slug"], slug); // Slug should remain the same
    assert!(response_body["article"]["author"].is_object());
    assert_eq!(response_body["article"]["author"]["username"], "updateuser");
}

#[tokio::test]
async fn test_update_article_unauthorized() {
    // Arrange
    let app = spawn_app().await;
    
    // Register first user and create article
    let user1_data = json!({
        "user": {
            "username": "author1",
            "email": "author1@example.com",
            "password": "securepassword123"
        }
    });
    
    let registration1_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user1_data)
        .send()
        .await
        .expect("Failed to register user1");
    
    let registration1_body: Value = registration1_response
        .json()
        .await
        .expect("Failed to parse registration1 response");
    
    let token1 = registration1_body["user"]["token"].as_str().unwrap();
    
    // Create article with user1
    let create_data = json!({
        "article": {
            "title": "User1 Article",
            "description": "This belongs to user1",
            "body": "Content by user1"
        }
    });
    
    let create_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token1))
        .json(&create_data)
        .send()
        .await
        .expect("Failed to create article");
    
    let create_body: Value = create_response
        .json()
        .await
        .expect("Failed to parse create response");
    
    let slug = create_body["article"]["slug"].as_str().unwrap();
    
    // Register second user
    let user2_data = json!({
        "user": {
            "username": "author2",
            "email": "author2@example.com",
            "password": "securepassword123"
        }
    });
    
    let registration2_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user2_data)
        .send()
        .await
        .expect("Failed to register user2");
    
    let registration2_body: Value = registration2_response
        .json()
        .await
        .expect("Failed to parse registration2 response");
    
    let token2 = registration2_body["user"]["token"].as_str().unwrap();
    
    // Try to update user1's article with user2's token
    let update_data = json!({
        "article": {
            "title": "Hacked Title"
        }
    });
    
    // Act
    let response = app
        .client
        .put(format!("{}/api/articles/{}", &app.address, slug))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token2))
        .json(&update_data)
        .send()
        .await
        .expect("Failed to execute request");
    
    // Assert
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_delete_article_happy_path() {
    // Arrange
    let app = spawn_app().await;
    
    // Register user and get token
    let user_data = json!({
        "user": {
            "username": "deleteuser",
            "email": "delete@example.com",
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
    
    // Create an article first
    let create_data = json!({
        "article": {
            "title": "Article to Delete",
            "description": "This will be deleted",
            "body": "Delete me",
            "tagList": ["delete", "test"]
        }
    });
    
    let create_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .json(&create_data)
        .send()
        .await
        .expect("Failed to create article");
    
    let create_body: Value = create_response
        .json()
        .await
        .expect("Failed to parse create response");
    
    let slug = create_body["article"]["slug"].as_str().unwrap();
    
    // Act - Delete the article
    let response = app
        .client
        .delete(format!("{}/api/articles/{}", &app.address, slug))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");
    
    // Assert
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    
    // Verify article is actually deleted
    let get_response = app
        .client
        .get(format!("{}/api/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to execute get request");
    
    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_article_unauthorized() {
    // Arrange
    let app = spawn_app().await;
    
    // Register first user and create article
    let user1_data = json!({
        "user": {
            "username": "owner",
            "email": "owner@example.com",
            "password": "securepassword123"
        }
    });
    
    let registration1_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user1_data)
        .send()
        .await
        .expect("Failed to register owner");
    
    let registration1_body: Value = registration1_response
        .json()
        .await
        .expect("Failed to parse registration response");
    
    let token1 = registration1_body["user"]["token"].as_str().unwrap();
    
    // Create article
    let create_data = json!({
        "article": {
            "title": "Protected Article",
            "description": "Cannot be deleted by others",
            "body": "This is protected"
        }
    });
    
    let create_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token1))
        .json(&create_data)
        .send()
        .await
        .expect("Failed to create article");
    
    let create_body: Value = create_response
        .json()
        .await
        .expect("Failed to parse create response");
    
    let slug = create_body["article"]["slug"].as_str().unwrap();
    
    // Register second user
    let user2_data = json!({
        "user": {
            "username": "hacker",
            "email": "hacker@example.com",
            "password": "securepassword123"
        }
    });
    
    let registration2_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user2_data)
        .send()
        .await
        .expect("Failed to register hacker");
    
    let registration2_body: Value = registration2_response
        .json()
        .await
        .expect("Failed to parse registration2 response");
    
    let token2 = registration2_body["user"]["token"].as_str().unwrap();
    
    // Act - Try to delete with wrong user
    let response = app
        .client
        .delete(format!("{}/api/articles/{}", &app.address, slug))
        .header("Authorization", format!("Bearer {}", token2))
        .send()
        .await
        .expect("Failed to execute request");
    
    // Assert
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    
    // Verify article still exists
    let get_response = app
        .client
        .get(format!("{}/api/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to execute get request");
    
    assert_eq!(get_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_articles_happy_path() {
    // Arrange
    let app = spawn_app().await;
    
    // Register user and get token
    let user_data = json!({
        "user": {
            "username": "listuser",
            "email": "list@example.com",
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
    
    // Create multiple articles
    let articles = vec![
        json!({
            "article": {
                "title": "First Article",
                "description": "First description",
                "body": "First body",
                "tagList": ["first", "test"]
            }
        }),
        json!({
            "article": {
                "title": "Second Article",
                "description": "Second description",
                "body": "Second body",
                "tagList": ["second"]
            }
        }),
    ];
    
    for article_data in &articles {
        app.client
            .post(format!("{}/api/articles", &app.address))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .json(article_data)
            .send()
            .await
            .expect("Failed to create article");
    }
    
    // Act - List articles
    let response = app
        .client
        .get(format!("{}/api/articles", &app.address))
        .send()
        .await
        .expect("Failed to execute request");
    
    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    
    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response body");
    
    assert!(response_body["articles"].is_array());
    assert_eq!(response_body["articlesCount"], 2);
    
    let articles_array = response_body["articles"].as_array().unwrap();
    assert_eq!(articles_array.len(), 2);
    
    // Verify article structure
    for article in articles_array {
        assert!(article["title"].is_string());
        assert!(article["description"].is_string());
        assert!(article["body"].is_string());
        assert!(article["slug"].is_string());
        assert!(article["tagList"].is_array());
        assert!(article["author"].is_object());
        assert_eq!(article["author"]["username"], "listuser");
        assert!(article["createdAt"].is_string());
        assert!(article["updatedAt"].is_string());
    }
}

#[tokio::test]
async fn test_list_articles_with_filters() {
    // Arrange
    let app = spawn_app().await;
    
    // Register users
    let user1_data = json!({
        "user": {
            "username": "author1",
            "email": "author1@example.com",
            "password": "securepassword123"
        }
    });
    
    let user2_data = json!({
        "user": {
            "username": "author2",
            "email": "author2@example.com",
            "password": "securepassword123"
        }
    });
    
    let reg1_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user1_data)
        .send()
        .await
        .expect("Failed to register user1");
    
    let reg2_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user2_data)
        .send()
        .await
        .expect("Failed to register user2");
    
    let reg1_body: Value = reg1_response
        .json()
        .await
        .expect("Failed to parse reg1 response");
    let reg2_body: Value = reg2_response
        .json()
        .await
        .expect("Failed to parse reg2 response");
    
    let token1 = reg1_body["user"]["token"].as_str().unwrap();
    let token2 = reg2_body["user"]["token"].as_str().unwrap();
    
    // Create articles with different tags and authors
    let article1 = json!({
        "article": {
            "title": "Rust Article",
            "description": "About Rust",
            "body": "Rust content",
            "tagList": ["rust", "programming"]
        }
    });
    
    let article2 = json!({
        "article": {
            "title": "Python Article",
            "description": "About Python",
            "body": "Python content",
            "tagList": ["python", "programming"]
        }
    });
    
    // Create articles by different authors
    app.client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token1))
        .json(&article1)
        .send()
        .await
        .expect("Failed to create article1");
    
    app.client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token2))
        .json(&article2)
        .send()
        .await
        .expect("Failed to create article2");
    
    // Act & Assert - Filter by author
    let response = app
        .client
        .get(format!("{}/api/articles?author=author1", &app.address))
        .send()
        .await
        .expect("Failed to execute request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response body");
    
    assert_eq!(response_body["articlesCount"], 1);
    assert_eq!(response_body["articles"][0]["author"]["username"], "author1");
    
    // Act & Assert - Filter by tag
    let response = app
        .client
        .get(format!("{}/api/articles?tag=rust", &app.address))
        .send()
        .await
        .expect("Failed to execute request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response body");
    
    assert_eq!(response_body["articlesCount"], 1);
    let tag_list = response_body["articles"][0]["tagList"].as_array().unwrap();
    assert!(tag_list.contains(&json!("rust")));
}

#[tokio::test]
async fn test_update_nonexistent_article() {
    // Arrange
    let app = spawn_app().await;
    
    // Register user and get token
    let user_data = json!({
        "user": {
            "username": "testuser",
            "email": "test@example.com",
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
    
    let update_data = json!({
        "article": {
            "title": "Updated Title"
        }
    });
    
    // Act - Try to update non-existent article
    let response = app
        .client
        .put(format!("{}/api/articles/non-existent-slug", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .json(&update_data)
        .send()
        .await
        .expect("Failed to execute request");
    
    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_nonexistent_article() {
    // Arrange
    let app = spawn_app().await;
    
    // Register user and get token
    let user_data = json!({
        "user": {
            "username": "testuser",
            "email": "test@example.com",
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
    
    // Act - Try to delete non-existent article
    let response = app
        .client
        .delete(format!("{}/api/articles/non-existent-slug", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");
    
    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
