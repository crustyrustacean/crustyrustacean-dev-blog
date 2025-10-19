// tests/api/comments.rs

use crate::helpers::spawn_app;
use reqwest::StatusCode;
use serde_json::{json, Value};

#[tokio::test]
async fn test_add_comment_to_article_happy_path() {
    // Arrange
    let app = spawn_app().await;

    // Register user and create article
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

    // Create an article
    let article_data = json!({
        "article": {
            "title": "Test Article for Comments",
            "description": "Testing comments",
            "body": "This article will have comments"
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

    // Register commenter
    let commenter_data = json!({
        "user": {
            "username": "commenter",
            "email": "commenter@example.com",
            "password": "securepassword123"
        }
    });

    let commenter_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&commenter_data)
        .send()
        .await
        .expect("Failed to register commenter");

    let commenter_body: Value = commenter_response
        .json()
        .await
        .expect("Failed to parse commenter response");

    let commenter_token = commenter_body["user"]["token"].as_str().unwrap();

    // Act - Add comment
    let comment_data = json!({
        "comment": {
            "body": "This is a great article!"
        }
    });

    let response = app
        .client
        .post(format!("{}/api/articles/{}/comments", &app.address, slug))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", commenter_token))
        .json(&comment_data)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response body");

    assert!(response_body["comment"].is_object());
    assert_eq!(response_body["comment"]["body"], "This is a great article!");
    assert!(response_body["comment"]["id"].is_string());
    assert!(response_body["comment"]["createdAt"].is_string());
    assert!(response_body["comment"]["updatedAt"].is_string());
    assert!(response_body["comment"]["author"].is_object());
    assert_eq!(response_body["comment"]["author"]["username"], "commenter");
}

#[tokio::test]
async fn test_add_comment_without_auth() {
    // Arrange
    let app = spawn_app().await;

    let comment_data = json!({
        "comment": {
            "body": "Unauthorized comment"
        }
    });

    // Act - Try to add comment without authentication
    let response = app
        .client
        .post(format!("{}/api/articles/some-slug/comments", &app.address))
        .header("Content-Type", "application/json")
        .json(&comment_data)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_add_comment_with_empty_body() {
    // Arrange
    let app = spawn_app().await;

    // Register user
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

    // Act - Try to add comment with empty body
    let comment_data = json!({
        "comment": {
            "body": ""
        }
    });

    let response = app
        .client
        .post(format!("{}/api/articles/some-slug/comments", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .json(&comment_data)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_add_comment_to_nonexistent_article() {
    // Arrange
    let app = spawn_app().await;

    // Register user
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

    // Act - Try to add comment to non-existent article
    let comment_data = json!({
        "comment": {
            "body": "Comment on non-existent article"
        }
    });

    let response = app
        .client
        .post(format!("{}/api/articles/non-existent-slug/comments", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .json(&comment_data)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_comments_from_article_happy_path() {
    // Arrange
    let app = spawn_app().await;

    // Register user and create article
    let user_data = json!({
        "user": {
            "username": "author",
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

    // Create article
    let article_data = json!({
        "article": {
            "title": "Article with Comments",
            "description": "Testing multiple comments",
            "body": "Article body"
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

    // Add multiple comments
    let comments = vec!["First comment", "Second comment", "Third comment"];
    
    for comment_text in &comments {
        let comment_data = json!({
            "comment": {
                "body": comment_text
            }
        });

        app.client
            .post(format!("{}/api/articles/{}/comments", &app.address, slug))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .json(&comment_data)
            .send()
            .await
            .expect("Failed to add comment");
    }

    // Act - Get all comments
    let response = app
        .client
        .get(format!("{}/api/articles/{}/comments", &app.address, slug))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response body");

    assert!(response_body["comments"].is_array());
    let comments_array = response_body["comments"].as_array().unwrap();
    assert_eq!(comments_array.len(), 3);

    // Verify comment structure
    for comment in comments_array {
        assert!(comment["id"].is_string());
        assert!(comment["body"].is_string());
        assert!(comment["createdAt"].is_string());
        assert!(comment["updatedAt"].is_string());
        assert!(comment["author"].is_object());
        assert_eq!(comment["author"]["username"], "author");
    }
}

#[tokio::test]
async fn test_get_comments_from_nonexistent_article() {
    // Arrange
    let app = spawn_app().await;

    // Act - Try to get comments from non-existent article
    let response = app
        .client
        .get(format!("{}/api/articles/non-existent-slug/comments", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_comments_from_article_with_no_comments() {
    // Arrange
    let app = spawn_app().await;

    // Register user and create article
    let user_data = json!({
        "user": {
            "username": "author",
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

    // Create article without comments
    let article_data = json!({
        "article": {
            "title": "Article without Comments",
            "description": "No comments yet",
            "body": "Article body"
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

    // Act - Get comments
    let response = app
        .client
        .get(format!("{}/api/articles/{}/comments", &app.address, slug))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response body");

    assert!(response_body["comments"].is_array());
    let comments_array = response_body["comments"].as_array().unwrap();
    assert_eq!(comments_array.len(), 0);
}

#[tokio::test]
async fn test_delete_comment_happy_path() {
    // Arrange
    let app = spawn_app().await;

    // Register user and create article
    let user_data = json!({
        "user": {
            "username": "author",
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

    // Create article
    let article_data = json!({
        "article": {
            "title": "Article for Comment Deletion",
            "description": "Testing comment deletion",
            "body": "Article body"
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

    // Add comment
    let comment_data = json!({
        "comment": {
            "body": "This comment will be deleted"
        }
    });

    let comment_response = app
        .client
        .post(format!("{}/api/articles/{}/comments", &app.address, slug))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .json(&comment_data)
        .send()
        .await
        .expect("Failed to add comment");

    let comment_body: Value = comment_response
        .json()
        .await
        .expect("Failed to parse comment response");

    let comment_id = comment_body["comment"]["id"].as_str().unwrap();

    // Act - Delete comment
    let response = app
        .client
        .delete(format!("{}/api/articles/{}/comments/{}", &app.address, slug, comment_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify comment is deleted
    let get_response = app
        .client
        .get(format!("{}/api/articles/{}/comments", &app.address, slug))
        .send()
        .await
        .expect("Failed to get comments");

    let get_body: Value = get_response
        .json()
        .await
        .expect("Failed to parse response");

    let comments_array = get_body["comments"].as_array().unwrap();
    assert_eq!(comments_array.len(), 0);
}

#[tokio::test]
async fn test_delete_comment_without_auth() {
    // Arrange
    let app = spawn_app().await;

    // Act - Try to delete comment without authentication
    let response = app
        .client
        .delete(format!("{}/api/articles/some-slug/comments/some-id", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_delete_comment_unauthorized_user() {
    // Arrange
    let app = spawn_app().await;

    // Register first user and create article with comment
    let user1_data = json!({
        "user": {
            "username": "author",
            "email": "author@example.com",
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

    let reg1_body: Value = reg1_response
        .json()
        .await
        .expect("Failed to parse registration response");

    let token1 = reg1_body["user"]["token"].as_str().unwrap();

    // Create article
    let article_data = json!({
        "article": {
            "title": "Protected Comments Article",
            "description": "Testing unauthorized deletion",
            "body": "Article body"
        }
    });

    let create_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token1))
        .json(&article_data)
        .send()
        .await
        .expect("Failed to create article");

    let create_body: Value = create_response
        .json()
        .await
        .expect("Failed to parse create response");

    let slug = create_body["article"]["slug"].as_str().unwrap();

    // Add comment
    let comment_data = json!({
        "comment": {
            "body": "Protected comment"
        }
    });

    let comment_response = app
        .client
        .post(format!("{}/api/articles/{}/comments", &app.address, slug))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token1))
        .json(&comment_data)
        .send()
        .await
        .expect("Failed to add comment");

    let comment_body: Value = comment_response
        .json()
        .await
        .expect("Failed to parse comment response");

    let comment_id = comment_body["comment"]["id"].as_str().unwrap();

    // Register second user
    let user2_data = json!({
        "user": {
            "username": "hacker",
            "email": "hacker@example.com",
            "password": "securepassword123"
        }
    });

    let reg2_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user2_data)
        .send()
        .await
        .expect("Failed to register user2");

    let reg2_body: Value = reg2_response
        .json()
        .await
        .expect("Failed to parse registration2 response");

    let token2 = reg2_body["user"]["token"].as_str().unwrap();

    // Act - Try to delete comment with different user
    let response = app
        .client
        .delete(format!("{}/api/articles/{}/comments/{}", &app.address, slug, comment_id))
        .header("Authorization", format!("Bearer {}", token2))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // Verify comment still exists
    let get_response = app
        .client
        .get(format!("{}/api/articles/{}/comments", &app.address, slug))
        .send()
        .await
        .expect("Failed to get comments");

    let get_body: Value = get_response
        .json()
        .await
        .expect("Failed to parse response");

    let comments_array = get_body["comments"].as_array().unwrap();
    assert_eq!(comments_array.len(), 1);
}

#[tokio::test]
async fn test_delete_nonexistent_comment() {
    // Arrange
    let app = spawn_app().await;

    // Register user and create article
    let user_data = json!({
        "user": {
            "username": "author",
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

    // Create article
    let article_data = json!({
        "article": {
            "title": "Test Article",
            "description": "Testing",
            "body": "Article body"
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

    // Act - Try to delete non-existent comment
    let response = app
        .client
        .delete(format!("{}/api/articles/{}/comments/non-existent-id", &app.address, slug))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
