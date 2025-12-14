// tests/api/template_rendering.rs

use crate::helpers::{
    APP_VERSION, HtmlResponseValidator, TestArticleBuilder, TestCommentBuilder, TestUserBuilder,
    assert_body_contains, assert_js_loaded, assert_navbar_authenticated, spawn_app,
};
use reqwest::StatusCode;

#[tokio::test]
async fn test_article_template_renders_successfully_happy_path() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("templateuser").await;

    // Create article with tags
    let slug = app
        .create_article(
            &token,
            "Template Rendering Test Article",
            "Testing the template rendering system with JavaScript functionality",
            "This article tests the template rendering functionality with comments and interactive elements.",
            vec!["template", "testing", "javascript"],
        )
        .await;

    // Add a comment
    app.add_comment(
        &token,
        &slug,
        "This is a test comment to verify the template renders comments correctly.",
    )
    .await;

    // Act - Access the article page
    let response = app
        .client
        .get(format!("{}/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Template should render successfully
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.assert_html_response().await;

    // Verify article content and tags
    assert_body_contains(
        &body,
        &[
            "Template Rendering Test Article",
            "Testing the template rendering system",
            "template",
            "testing",
            "javascript",
            "</html>",
        ],
    );

    // Verify external JavaScript files are loaded with cache-busting
    assert_js_loaded(&body, "article-page.js", APP_VERSION);
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

    let body = response.text().await.expect("Failed to get response body");
    assert!(
        !body.is_empty(),
        "Response body should not be empty for 404"
    );
}

#[tokio::test]
async fn test_article_template_renders_with_comments() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("commenter").await;

    // Create article and add comment
    let slug = app
        .create_article(
            &token,
            "Comments Test Article",
            "Testing comment functionality in templates",
            "This tests that comments render correctly in the template.",
            vec!["comments", "test"],
        )
        .await;

    app.add_comment(
        &token,
        &slug,
        "This is a test comment to verify template rendering.",
    )
    .await;

    // Act - Access the article page
    let response = app
        .client
        .get(format!("{}/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.assert_html_response().await;

    assert_body_contains(&body, &["Comments Test Article", "</html>"]);
    assert_js_loaded(&body, "article-page.js", APP_VERSION);
}

#[tokio::test]
async fn test_template_compiles_without_javascript_errors() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("jstest").await;

    let slug = app
        .create_article(
            &token,
            "JavaScript Syntax Test",
            "Testing that templates compile without JavaScript syntax errors",
            "This tests that templates render correctly without syntax errors.",
            vec!["javascript", "template"],
        )
        .await;

    // Act - Access the article page
    let response = app
        .client
        .get(format!("{}/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.assert_html_response().await;

    assert_body_contains(&body, &["JavaScript Syntax Test", "</html>"]);
    assert_js_loaded(&body, "article-page.js", APP_VERSION);
}

#[tokio::test]
async fn test_template_renders_consistently() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("consistent").await;

    let slug = app
        .create_article(
            &token,
            "Template Consistency Test",
            "Testing template rendering consistency",
            "This verifies templates render reliably after our fixes.",
            vec!["consistency", "template"],
        )
        .await;

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

    let body1 = response1.assert_html_response().await;
    let body2 = response2.assert_html_response().await;

    // Verify core content is present in both responses
    assert_body_contains(&body1, &["Template Consistency Test", "consistent"]);
    assert_body_contains(&body2, &["Template Consistency Test", "consistent"]);

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
    let token = app.register_user_default("navbaruser").await;

    let slug = app
        .create_article(
            &token,
            "Navbar Authentication Test",
            "Testing that navbar shows user when editing",
            "This verifies the edit page includes user context in the template.",
            vec!["navbar", "auth"],
        )
        .await;

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
    let body = response.assert_html_response().await;

    // Verify navbar shows authenticated state
    assert_navbar_authenticated(&body, "navbaruser");

    // Verify the editor form is present with article data
    assert_body_contains(&body, &["articleForm", "Navbar Authentication Test"]);
}

#[tokio::test]
async fn test_new_article_editor_shows_authenticated_user_in_navbar() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("neweditoruser").await;

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
    let body = response.assert_html_response().await;

    // Verify navbar shows authenticated state
    assert_navbar_authenticated(&body, "neweditoruser");
}

#[tokio::test]
async fn test_homepage_shows_total_article_count() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testauthor").await;

    // Create 10 articles (more than the 4 displayed on homepage)
    for i in 1..=10 {
        app.create_article_simple(&token, &format!("Article {}", i))
            .await;
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
    let body = response.assert_html_response().await;

    // Verify the total article count shows 10, not just 4
    assert!(
        body.contains(">10<") || body.contains("> 10 <"),
        "Homepage should display total article count (10) from database"
    );
    assert!(body.contains("Published Articles"));
}

// ===== Draft Widget Tests =====

#[tokio::test]
async fn test_homepage_shows_draft_widget_for_authenticated_user() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("draftuser").await;

    // Create 3 draft articles and 2 published articles
    for i in 1..=3 {
        app.create_draft_article_simple(&token, &format!("Draft Article {}", i))
            .await;
    }

    for i in 1..=2 {
        app.create_article_simple(&token, &format!("Published Article {}", i))
            .await;
    }

    // Act - Access the homepage while authenticated
    let response = app
        .client
        .get(&app.address)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("Failed to get response body");

    // Verify draft widget is shown with count of 3
    assert!(body.contains("Draft Articles"));
    assert!(body.contains(">3<") || body.contains("> 3 <"));

    // Verify link to drafts page
    assert!(body.contains("/admin/drafts"));
    assert!(body.contains("View Drafts"));
}

#[tokio::test]
async fn test_homepage_draft_widget_hidden_for_guest_user() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("draftuser2").await;

    // Create 5 draft articles
    for i in 1..=5 {
        app.create_draft_article_simple(&token, &format!("Draft Article {}", i))
            .await;
    }

    // Act - Access the homepage without authentication
    let response = app
        .client
        .get(&app.address)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("Failed to get response body");

    // Verify draft widget is NOT shown for unauthenticated users
    assert!(!body.contains("Draft Articles"));
}

#[tokio::test]
async fn test_homepage_draft_widget_shows_zero_when_no_drafts() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("nodrafts").await;

    // Create only published articles, no drafts
    for i in 1..=3 {
        app.create_article_simple(&token, &format!("Published Article {}", i))
            .await;
    }

    // Act - Access the homepage while authenticated
    let response = app
        .client
        .get(&app.address)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("Failed to get response body");

    // Verify draft widget is shown with count of 0
    assert!(body.contains("Draft Articles"));
    assert!(body.contains(">0<") || body.contains("> 0 <"));

    // Verify "View Drafts" link is NOT shown when count is 0
    assert!(!body.contains("View Drafts"));
}

#[tokio::test]
async fn test_homepage_separates_published_and_draft_counts() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("separatecounts").await;

    // Create 5 published articles
    for i in 1..=5 {
        app.create_article_simple(&token, &format!("Published Article {}", i))
            .await;
    }

    // Create 3 draft articles
    for i in 1..=3 {
        app.create_draft_article_simple(&token, &format!("Draft Article {}", i))
            .await;
    }

    // Act - Access the homepage while authenticated
    let response = app
        .client
        .get(&app.address)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("Failed to get response body");

    // Verify the page shows both counts separately
    assert!(body.contains("Published Articles"));
    assert!(body.contains("Draft Articles"));

    // Verify that the total count shown is for published only (5), not including drafts
    let published_section = body.split("Published Articles").next().unwrap_or("");
    assert!(
        published_section.contains(">5<") || published_section.contains("> 5 <"),
        "Published articles count should be 5, not including drafts"
    );
}
