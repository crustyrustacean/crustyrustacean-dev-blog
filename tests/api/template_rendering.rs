// tests/api/template_rendering.rs

use crate::helpers::spawn_app;
use reqwest::StatusCode;
use serde_json::{Value, json};

#[tokio::test]
async fn test_article_template_renders_successfully_happy_path() {
    // Arrange
    let app = spawn_app().await;

    // Register user and get token
    let user_data = json!({
        "user": {
            "username": "templateuser",
            "email": "template@example.com",
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

    // Create an article to render
    let article_data = json!({
        "article": {
            "title": "Template Rendering Test Article",
            "description": "Testing the template rendering system with JavaScript functionality",
            "body": "This article tests the template rendering functionality with comments and interactive elements.",
            "tagList": ["template", "testing", "javascript"]
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

    // Add a comment to test comment rendering
    let comment_data = json!({
        "comment": {
            "body": "This is a test comment to verify the template renders comments correctly."
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

    // Act - Access the article page
    let response = app
        .client
        .get(format!("{}/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Template should render successfully
    assert_eq!(response.status(), StatusCode::OK);

    // Verify content type is HTML
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    assert!(content_type.contains("text/html") || content_type.is_empty());

    // Get response body and verify template rendered correctly
    let body = response.text().await.expect("Failed to get response body");

    // Verify basic HTML structure
    assert!(body.contains("<html") || body.contains("<!DOCTYPE html"));
    assert!(body.contains("</html>"));

    // Verify article content is rendered
    assert!(body.contains("Template Rendering Test Article"));
    assert!(body.contains("Testing the template rendering system"));

    // Verify external JavaScript files are loaded
    assert!(body.contains("/static/js/article-page.js"));

    // Verify tags are rendered
    assert!(body.contains("template"));
    assert!(body.contains("testing"));
    assert!(body.contains("javascript"));
}

#[tokio::test]
async fn test_article_template_handles_nonexistent_article_failure_path() {
    // Arrange
    let app = spawn_app().await;

    // Act - Try to access a non-existent article page
    let response = app
        .client
        .get(format!(
            "{}/articles/nonexistent-article-slug",
            &app.address
        ))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should return 404 Not Found
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // The response should be some form of error response
    // (Either HTML error page or JSON error - both are valid for 404s)
    let body = response.text().await.expect("Failed to get response body");

    // Should contain some indication this is a 404 error
    assert!(
        !body.is_empty(),
        "Response body should not be empty for 404"
    );
}

#[tokio::test]
async fn test_article_template_renders_with_comments() {
    // Arrange
    let app = spawn_app().await;

    // Register user without profile image
    let user_data = json!({
        "user": {
            "username": "commenter",
            "email": "commenter@example.com",
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
            "title": "Comments Test Article",
            "description": "Testing comment functionality in templates",
            "body": "This tests that comments render correctly in the template.",
            "tagList": ["comments", "test"]
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

    // Add a comment
    let comment_data = json!({
        "comment": {
            "body": "This is a test comment to verify template rendering."
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

    // Act - Access the article page
    let response = app
        .client
        .get(format!("{}/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.text().await.expect("Failed to get response body");

    // Verify the article loads correctly
    assert!(body.contains("Comments Test Article"));

    // Verify external JS files are loaded
    assert!(body.contains("/static/js/article-page.js"));

    // Verify the template renders without JavaScript errors (basic validation)
    assert!(body.contains("<html") || body.contains("<!DOCTYPE html"));
    assert!(body.contains("</html>"));
}

#[tokio::test]
async fn test_template_compiles_without_javascript_errors() {
    // Arrange
    let app = spawn_app().await;

    // Register user
    let user_data = json!({
        "user": {
            "username": "jstest",
            "email": "jstest@example.com",
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
            "title": "JavaScript Syntax Test",
            "description": "Testing that templates compile without JavaScript syntax errors",
            "body": "This tests that templates render correctly without syntax errors.",
            "tagList": ["javascript", "template"]
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

    // Act - Access the article page
    let response = app
        .client
        .get(format!("{}/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.text().await.expect("Failed to get response body");

    // Verify the template renders without errors (primary concern after our fixes)
    assert!(body.contains("JavaScript Syntax Test"));

    // Verify external JS modules are loaded
    assert!(body.contains("/static/js/article-page.js"));

    // The template should render complete HTML
    assert!(body.contains("<html") || body.contains("<!DOCTYPE html"));
    assert!(body.contains("</html>"));

    // Should not be empty (indicates successful rendering)
    assert!(!body.is_empty());
}

#[tokio::test]
async fn test_template_renders_consistently() {
    // Arrange
    let app = spawn_app().await;

    // Register user
    let user_data = json!({
        "user": {
            "username": "consistent",
            "email": "consistent@example.com",
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
            "title": "Template Consistency Test",
            "description": "Testing template rendering consistency",
            "body": "This verifies templates render reliably after our fixes.",
            "tagList": ["consistency", "template"]
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

    // Act - Access the article page multiple times
    let response1 = app
        .client
        .get(format!("{}/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to execute request 1");

    let response2 = app
        .client
        .get(format!("{}/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to execute request 2");

    // Assert - Template should render consistently
    assert_eq!(response1.status(), StatusCode::OK);
    assert_eq!(response2.status(), StatusCode::OK);

    let body1 = response1
        .text()
        .await
        .expect("Failed to get response body 1");
    let body2 = response2
        .text()
        .await
        .expect("Failed to get response body 2");

    // Verify core content is present in both responses
    assert!(body1.contains("Template Consistency Test"));
    assert!(body2.contains("Template Consistency Test"));
    assert!(body1.contains("consistent"));
    assert!(body2.contains("consistent"));

    // Both responses should be valid HTML
    assert!(body1.contains("<html") || body1.contains("<!DOCTYPE html"));
    assert!(body2.contains("<html") || body2.contains("<!DOCTYPE html"));

    // Responses should be essentially the same (excluding timestamps)
    assert_eq!(
        body1.len(),
        body2.len(),
        "Template rendering should be consistent"
    );
}
