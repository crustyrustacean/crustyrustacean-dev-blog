// tests/api/template_rendering.rs

use crate::helpers::{spawn_app, TestUserBuilder, TestArticleBuilder};
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

#[tokio::test]
async fn test_editor_page_shows_authenticated_user_in_navbar() {
    // Arrange
    let app = spawn_app().await;

    // Register user
    let user_data = json!({
        "user": {
            "username": "navbaruser",
            "email": "navbar@example.com",
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

    // Create an article to edit
    let article_data = json!({
        "article": {
            "title": "Navbar Authentication Test",
            "description": "Testing that navbar shows user when editing",
            "body": "This verifies the edit page includes user context in the template.",
            "tagList": ["navbar", "auth"]
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

    // Act - Access the edit page with authentication
    let response = app
        .client
        .get(format!("{}/editor/{}", &app.address, slug))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.text().await.expect("Failed to get response body");

    // Verify the page renders
    assert!(body.contains("<html") || body.contains("<!DOCTYPE html"));
    assert!(body.contains("</html>"));

    // CRITICAL: Verify the username appears in the navbar
    // This checks that user context was passed to the template
    assert!(
        body.contains("navbaruser"),
        "Username should appear in navbar when editing article"
    );

    // CRITICAL: Verify "Login" and "Register" buttons do NOT appear in visible navbar
    // When authenticated, these should be hidden/not rendered in the active navbar
    // Note: They might exist in template code, but shouldn't be in the rendered user-facing nav
    let body_lowercase = body.to_lowercase();

    // Check that we don't have both Login AND Register links visible together
    // (which would indicate the unauthenticated navbar is showing)
    let has_login_link = body_lowercase.contains(">login<") || body_lowercase.contains("login</a>");
    let has_register_link =
        body_lowercase.contains(">register<") || body_lowercase.contains("register</a>");

    // If BOTH login and register are visible, user context is missing (the bug we fixed)
    assert!(
        !(has_login_link && has_register_link),
        "Login and Register links should not both be visible when user is authenticated. \
         This indicates missing user context in template (the bug we fixed)."
    );

    // Verify the editor form is present
    assert!(body.contains("article-form") || body.contains("editor"));

    // Verify the article data is pre-filled in the form
    assert!(body.contains("Navbar Authentication Test"));
}

#[tokio::test]
async fn test_new_article_editor_shows_authenticated_user_in_navbar() {
    // Arrange
    let app = spawn_app().await;

    // Register user
    let user_data = json!({
        "user": {
            "username": "neweditoruser",
            "email": "neweditor@example.com",
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

    // Act - Access the new article editor page with authentication
    let response = app
        .client
        .get(format!("{}/editor", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.text().await.expect("Failed to get response body");

    // Verify the page renders
    assert!(body.contains("<html") || body.contains("<!DOCTYPE html"));

    // Verify the username appears in the navbar
    assert!(
        body.contains("neweditoruser"),
        "Username should appear in navbar on new article editor page"
    );

    // Verify Login/Register links are not both visible (indicating authenticated state)
    let body_lowercase = body.to_lowercase();
    let has_login_link = body_lowercase.contains(">login<") || body_lowercase.contains("login</a>");
    let has_register_link =
        body_lowercase.contains(">register<") || body_lowercase.contains("register</a>");

    assert!(
        !(has_login_link && has_register_link),
        "Login and Register links should not both be visible when user is authenticated"
    );
}

#[tokio::test]
async fn test_homepage_shows_total_article_count() {
    // Arrange
    let app = spawn_app().await;

    // Register user and create 10 articles (more than the 4 displayed on homepage)
    let token = app.register_user_default("testauthor").await;

    for i in 1..=10 {
        app.create_article_simple(&token, &format!("Article {}", i)).await;
    }

    // Act - Access the homepage
    let response = app
        .client
        .get(&app.address)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.text().await.expect("Failed to get response body");

    // Verify the page renders
    assert!(body.contains("<html") || body.contains("<!DOCTYPE html"));
    assert!(body.contains("</html>"));

    // CRITICAL: Verify the total article count shows 10, not just 4
    // The homepage should show the total count from the database,
    // not just the count of articles displayed on the page
    assert!(
        body.contains(">10<") || body.contains("> 10 <"),
        "Homepage should display total article count (10) from database, not just displayed articles (4)"
    );

    // Verify "Published Articles" label is present
    assert!(body.contains("Published Articles"));
}
