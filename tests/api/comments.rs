// tests/api/comments.rs

use crate::helpers::{TestArticleBuilder, TestFixture, TestUserBuilder, spawn_app};
use crate::{assert_status, bearer_request, parse_json};
use reqwest::StatusCode;
use serde_json::{Value, json};

#[tokio::test]
async fn test_add_comment_to_article_happy_path() {
    // Arrange - Use TestFixture for multi-user scenario
    let fixture = TestFixture::new()
        .await
        .with_user_default("articleauthor")
        .await
        .with_user_default("commenter")
        .await;

    let author_token = fixture.get_token("articleauthor");
    let commenter_token = fixture.get_token("commenter");

    // Create an article
    let slug = fixture
        .app
        .create_article_simple(&author_token, "Test Article for Comments")
        .await;

    // Act - Add comment
    let comment_data = json!({
        "comment": {
            "body": "This is a great article!"
        }
    });

    let response = bearer_request!(
        post & fixture.app,
        format!("{}/api/articles/{}/comments", &fixture.app.address, slug),
        &commenter_token,
        comment_data
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

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
    assert_status!(response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_add_comment_with_empty_body() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act - Try to add comment with empty body
    let comment_data = json!({
        "comment": {
            "body": ""
        }
    });

    let response = bearer_request!(
        post & app,
        format!("{}/api/articles/some-slug/comments", &app.address),
        &token,
        comment_data
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_add_comment_to_nonexistent_article() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act - Try to add comment to non-existent article
    let comment_data = json!({
        "comment": {
            "body": "Comment on non-existent article"
        }
    });

    let response = bearer_request!(
        post & app,
        format!("{}/api/articles/non-existent-slug/comments", &app.address),
        &token,
        comment_data
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_comments_from_article_happy_path() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("author").await;

    // Create article
    let slug = app
        .create_article_simple(&token, "Article with Comments")
        .await;

    // Add multiple comments
    let comments = vec!["First comment", "Second comment", "Third comment"];

    for comment_text in &comments {
        let comment_data = json!({
            "comment": {
                "body": comment_text
            }
        });

        bearer_request!(
            post & app,
            format!("{}/api/articles/{}/comments", &app.address, slug),
            &token,
            comment_data
        )
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
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

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
        .get(format!(
            "{}/api/articles/non-existent-slug/comments",
            &app.address
        ))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_comments_from_article_with_no_comments() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("author").await;

    // Create article without comments
    let slug = app
        .create_article_simple(&token, "Article without Comments")
        .await;

    // Act - Get comments
    let response = app
        .client
        .get(format!("{}/api/articles/{}/comments", &app.address, slug))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    assert!(response_body["comments"].is_array());
    let comments_array = response_body["comments"].as_array().unwrap();
    assert_eq!(comments_array.len(), 0);
}

#[tokio::test]
async fn test_delete_comment_happy_path() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("author").await;

    // Create article
    let slug = app
        .create_article_simple(&token, "Article for Comment Deletion")
        .await;

    // Add comment
    let comment_data = json!({
        "comment": {
            "body": "This comment will be deleted"
        }
    });

    let comment_response = bearer_request!(
        post & app,
        format!("{}/api/articles/{}/comments", &app.address, slug),
        &token,
        comment_data
    )
    .expect("Failed to add comment");

    let comment_body: Value = parse_json!(comment_response);
    let comment_id = comment_body["comment"]["id"].as_str().unwrap();

    // Act - Delete comment
    let response = bearer_request!(
        delete & app,
        format!(
            "{}/api/articles/{}/comments/{}",
            &app.address, slug, comment_id
        ),
        &token
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::NO_CONTENT);

    // Verify comment is deleted
    let get_response = app
        .client
        .get(format!("{}/api/articles/{}/comments", &app.address, slug))
        .send()
        .await
        .expect("Failed to get comments");

    let get_body: Value = parse_json!(get_response);
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
        .delete(format!(
            "{}/api/articles/some-slug/comments/some-id",
            &app.address
        ))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_delete_comment_unauthorized_user() {
    // Arrange - Use TestFixture for multi-user scenario
    let fixture = TestFixture::new()
        .await
        .with_user_default("author")
        .await
        .with_user_default("hacker")
        .await;

    let author_token = fixture.get_token("author");
    let hacker_token = fixture.get_token("hacker");

    // Create article
    let slug = fixture
        .app
        .create_article_simple(&author_token, "Protected Comments Article")
        .await;

    // Add comment
    let comment_data = json!({
        "comment": {
            "body": "Protected comment"
        }
    });

    let comment_response = bearer_request!(
        post & fixture.app,
        format!("{}/api/articles/{}/comments", &fixture.app.address, slug),
        &author_token,
        comment_data
    )
    .expect("Failed to add comment");

    let comment_body: Value = parse_json!(comment_response);
    let comment_id = comment_body["comment"]["id"].as_str().unwrap();

    // Act - Try to delete comment with different user
    let response = bearer_request!(
        delete & fixture.app,
        format!(
            "{}/api/articles/{}/comments/{}",
            &fixture.app.address, slug, comment_id
        ),
        &hacker_token
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::FORBIDDEN);

    // Verify comment still exists
    let get_response = fixture
        .app
        .client
        .get(format!(
            "{}/api/articles/{}/comments",
            &fixture.app.address, slug
        ))
        .send()
        .await
        .expect("Failed to get comments");

    let get_body: Value = parse_json!(get_response);
    let comments_array = get_body["comments"].as_array().unwrap();
    assert_eq!(comments_array.len(), 1);
}

#[tokio::test]
async fn test_delete_nonexistent_comment() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("author").await;

    // Create article
    let slug = app.create_article_simple(&token, "Test Article").await;

    // Act - Try to delete non-existent comment
    let response = bearer_request!(
        delete & app,
        format!(
            "{}/api/articles/{}/comments/non-existent-id",
            &app.address, slug
        ),
        &token
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::NOT_FOUND);
}
