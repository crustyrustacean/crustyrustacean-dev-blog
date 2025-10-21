// tests/api/articles.rs

use crate::helpers::{TestArticleBuilder, TestFixture, TestUserBuilder, spawn_app};
use crate::{assert_status, bearer_request, parse_json};
use reqwest::StatusCode;
use serde_json::{Value, json};

#[tokio::test]
async fn test_create_article_happy_path() {
    // Arrange
    let app = spawn_app().await;
    let token = app
        .register_user("articleauthor", "author@example.com", "securepassword123")
        .await;

    let article_data = json!({
        "article": {
            "title": "How to Train Your Dragon",
            "description": "Ever wonder how?",
            "body": "You have to believe in yourself. That's the secret to life.",
            "tagList": ["dragons", "training"]
        }
    });

    // Act - Create article
    let response = bearer_request!(
        post & app,
        format!("{}/api/articles", &app.address),
        &token,
        article_data
    )
    .expect("Failed to execute request");

    // Assert
    let status = response.status();
    if status != StatusCode::OK {
        let error_body = response.text().await.expect("Failed to get error text");
        panic!("Expected 200 OK, got {}: {}", status, error_body);
    }

    let response_body: Value = parse_json!(response);

    // Verify response structure
    assert!(response_body["article"].is_object());
    assert_eq!(
        response_body["article"]["title"],
        "How to Train Your Dragon"
    );
    assert_eq!(response_body["article"]["description"], "Ever wonder how?");
    assert_eq!(
        response_body["article"]["body"],
        "You have to believe in yourself. That's the secret to life."
    );
    assert_eq!(response_body["article"]["slug"], "how-to-train-your-dragon");
    assert_eq!(
        response_body["article"]["tagList"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(response_body["article"]["favorited"], false);
    assert_eq!(response_body["article"]["favoritesCount"], 0);
    assert!(response_body["article"]["author"].is_object());
    assert_eq!(
        response_body["article"]["author"]["username"],
        "articleauthor"
    );
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
    assert_status!(response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_article_with_invalid_data() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

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
        let response = bearer_request!(
            post & app,
            format!("{}/api/articles", &app.address),
            &token,
            invalid_data
        )
        .expect("Failed to execute request");

        // Assert
        assert_status!(response, StatusCode::BAD_REQUEST);
    }
}

#[tokio::test]
async fn test_get_article_happy_path() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("articlereader").await;

    // Create an article first
    let slug = app
        .create_article(
            &token,
            "Test Article for Reading",
            "This is a test article",
            "Content of the test article",
            vec!["test", "reading"],
        )
        .await;

    // Act - Get the article
    let response = app
        .client
        .get(format!("{}/api/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    assert!(response_body["article"].is_object());
    assert_eq!(
        response_body["article"]["title"],
        "Test Article for Reading"
    );
    assert_eq!(
        response_body["article"]["description"],
        "This is a test article"
    );
    assert_eq!(
        response_body["article"]["body"],
        "Content of the test article"
    );
    assert_eq!(response_body["article"]["slug"], "test-article-for-reading");
    assert_eq!(
        response_body["article"]["tagList"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(response_body["article"]["favorited"], false);
    assert_eq!(response_body["article"]["favoritesCount"], 0);
    assert!(response_body["article"]["author"].is_object());
    assert_eq!(
        response_body["article"]["author"]["username"],
        "articlereader"
    );
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
    assert_status!(response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_article_page_route() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("pageuser").await;

    // Create an article first
    let slug = app
        .create_article(
            &token,
            "Test Article Page",
            "Testing the article page",
            "This is the article content for testing the page.",
            vec!["test", "page"],
        )
        .await;

    // Act - Access the article page (HTML route)
    let response = app
        .client
        .get(format!("{}/articles/{}", &app.address, slug))
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
    let token = app.register_user_default("slugtester").await;

    // Create first article
    let article_data1 = json!({
        "article": {
            "title": "Duplicate Title",
            "description": "First article",
            "body": "This is the first article with duplicate title"
        }
    });

    let response1 = bearer_request!(
        post & app,
        format!("{}/api/articles", &app.address),
        &token,
        article_data1
    )
    .expect("Failed to create first article");

    assert_status!(response1, StatusCode::OK);

    let body1: Value = parse_json!(response1);
    let slug1 = body1["article"]["slug"].as_str().unwrap();

    // Create second article with same title
    let article_data2 = json!({
        "article": {
            "title": "Duplicate Title",
            "description": "Second article",
            "body": "This is the second article with duplicate title"
        }
    });

    let response2 = bearer_request!(
        post & app,
        format!("{}/api/articles", &app.address),
        &token,
        article_data2
    )
    .expect("Failed to create second article");

    assert_status!(response2, StatusCode::OK);

    let body2: Value = parse_json!(response2);
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
    let token = app.register_user_default("updateuser").await;

    // Create an article first
    let slug = app
        .create_article(
            &token,
            "Original Title",
            "Original description",
            "Original body content",
            vec!["original"],
        )
        .await;

    // Prepare update data
    let update_data = json!({
        "article": {
            "title": "Updated Title",
            "description": "Updated description",
            "body": "Updated body content"
        }
    });

    // Act - Update the article
    let response = bearer_request!(
        put & app,
        format!("{}/api/articles/{}", &app.address, slug),
        &token,
        update_data
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    assert!(response_body["article"].is_object());
    assert_eq!(response_body["article"]["title"], "Updated Title");
    assert_eq!(
        response_body["article"]["description"],
        "Updated description"
    );
    assert_eq!(response_body["article"]["body"], "Updated body content");
    assert_eq!(response_body["article"]["slug"], slug); // Slug should remain the same
    assert!(response_body["article"]["author"].is_object());
    assert_eq!(response_body["article"]["author"]["username"], "updateuser");
}

#[tokio::test]
async fn test_update_article_unauthorized() {
    // Arrange - Use TestFixture for multi-user scenario
    let fixture = TestFixture::new()
        .await
        .with_user_default("author1")
        .await
        .with_user_default("author2")
        .await;

    let token1 = fixture.get_token("author1");
    let token2 = fixture.get_token("author2");

    // Create article with user1
    let slug = fixture
        .app
        .create_article_simple(&token1, "User1 Article")
        .await;

    // Try to update user1's article with user2's token
    let update_data = json!({
        "article": {
            "title": "Hacked Title"
        }
    });

    // Act
    let response = bearer_request!(
        put & fixture.app,
        format!("{}/api/articles/{}", &fixture.app.address, slug),
        &token2,
        update_data
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_delete_article_happy_path() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("deleteuser").await;

    // Create an article first
    let slug = app
        .create_article(
            &token,
            "Article to Delete",
            "This will be deleted",
            "Delete me",
            vec!["delete", "test"],
        )
        .await;

    // Act - Delete the article
    let response = bearer_request!(
        delete & app,
        format!("{}/api/articles/{}", &app.address, slug),
        &token
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::NO_CONTENT);

    // Verify article is actually deleted
    let get_response = app
        .client
        .get(format!("{}/api/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to execute get request");

    assert_status!(get_response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_article_unauthorized() {
    // Arrange - Use TestFixture for multi-user scenario
    let fixture = TestFixture::new()
        .await
        .with_user_default("owner")
        .await
        .with_user_default("hacker")
        .await;

    let token1 = fixture.get_token("owner");
    let token2 = fixture.get_token("hacker");

    // Create article
    let slug = fixture
        .app
        .create_article_simple(&token1, "Protected Article")
        .await;

    // Act - Try to delete with wrong user
    let response = bearer_request!(
        delete & fixture.app,
        format!("{}/api/articles/{}", &fixture.app.address, slug),
        &token2
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::FORBIDDEN);

    // Verify article still exists
    let get_response = fixture
        .app
        .client
        .get(format!("{}/api/articles/{}", &fixture.app.address, slug))
        .send()
        .await
        .expect("Failed to execute get request");

    assert_status!(get_response, StatusCode::OK);
}

#[tokio::test]
async fn test_list_articles_happy_path() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("listuser").await;

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
        bearer_request!(
            post & app,
            format!("{}/api/articles", &app.address),
            &token,
            article_data
        )
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
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

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
    // Arrange - Use TestFixture for multi-user scenario
    let fixture = TestFixture::new()
        .await
        .with_user_default("author1")
        .await
        .with_user_default("author2")
        .await;

    let token1 = fixture.get_token("author1");
    let token2 = fixture.get_token("author2");

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
    bearer_request!(
        post & fixture.app,
        format!("{}/api/articles", &fixture.app.address),
        &token1,
        article1
    )
    .expect("Failed to create article1");

    bearer_request!(
        post & fixture.app,
        format!("{}/api/articles", &fixture.app.address),
        &token2,
        article2
    )
    .expect("Failed to create article2");

    // Act & Assert - Filter by author
    let response = fixture
        .app
        .client
        .get(format!(
            "{}/api/articles?author=author1",
            &fixture.app.address
        ))
        .send()
        .await
        .expect("Failed to execute request");

    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    assert_eq!(response_body["articlesCount"], 1);
    assert_eq!(
        response_body["articles"][0]["author"]["username"],
        "author1"
    );

    // Act & Assert - Filter by tag
    let response = fixture
        .app
        .client
        .get(format!("{}/api/articles?tag=rust", &fixture.app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    assert_eq!(response_body["articlesCount"], 1);
    let tag_list = response_body["articles"][0]["tagList"].as_array().unwrap();
    assert!(tag_list.contains(&json!("rust")));
}

#[tokio::test]
async fn test_update_nonexistent_article() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    let update_data = json!({
        "article": {
            "title": "Updated Title"
        }
    });

    // Act - Try to update non-existent article
    let response = bearer_request!(
        put & app,
        format!("{}/api/articles/non-existent-slug", &app.address),
        &token,
        update_data
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_nonexistent_article() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act - Try to delete non-existent article
    let response = bearer_request!(
        delete & app,
        format!("{}/api/articles/non-existent-slug", &app.address),
        &token
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::NOT_FOUND);
}
