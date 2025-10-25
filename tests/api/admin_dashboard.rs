// tests/api/admin_dashboard.rs

use crate::helpers::{
    HtmlResponseValidator, TestArticleBuilder, TestUserBuilder, assert_body_contains, spawn_app,
};
use reqwest::StatusCode;

#[tokio::test]
async fn test_admin_dashboard_requires_auth() {
    // Arrange
    let app = spawn_app().await;

    // Act - Try to access admin dashboard without auth
    let response = app
        .client
        .get(format!("{}/admin", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should redirect or return unauthorized
    assert!(response.status() == StatusCode::UNAUTHORIZED || response.status().is_redirection());
}

#[tokio::test]
async fn test_admin_dashboard_with_valid_auth() {
    // Arrange
    let app = spawn_app().await;

    // Register user and get token
    let token = app
        .register_user("adminuser", "admin@example.com", "securepassword123")
        .await;

    // Act - Access admin dashboard with authentication
    let response = app
        .client
        .get(format!("{}/admin", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should return successful HTML response
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.assert_html_response().await;
    assert!(body.contains("Admin Dashboard"));
}

#[tokio::test]
async fn test_admin_dashboard_with_articles() {
    // Arrange
    let app = spawn_app().await;

    // Register user and get token
    let token = app.register_user_default("dashboarduser").await;

    // Create an article to show in dashboard
    app.create_article(
        &token,
        "Dashboard Test Article",
        "This article should appear in the dashboard",
        "This is content for the dashboard test article.",
        vec!["test", "dashboard"],
    )
    .await;

    // Act - Access admin dashboard with articles present
    let response = app
        .client
        .get(format!("{}/admin", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should return successful HTML response with article content
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.assert_html_response().await;

    // Should contain the dashboard structure and article content
    assert_body_contains(
        &body,
        &[
            "Admin Dashboard",
            "Dashboard Test Article",
            "dashboarduser",
            "New Article",
            "Actions",
        ],
    );
}

#[tokio::test]
async fn test_admin_dashboard_template_renders_with_special_characters() {
    // Arrange
    let app = spawn_app().await;

    // Register user with special characters in username
    let token = app
        .register_user("test-user_123", "special@example.com", "securepassword123")
        .await;

    // Create an article with special characters in title (potential template issue)
    app.create_article(
        &token,
        "Test Article with 'Quotes' and \"Double Quotes\"",
        "Description with <HTML> & special chars",
        "Body with special characters: & < > ' \"",
        vec!["special-chars", "test"],
    )
    .await;

    // Act - Access admin dashboard (this should not fail due to special characters)
    let response = app
        .client
        .get(format!("{}/admin", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should handle special characters properly and return successful response
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Dashboard should render with special characters"
    );

    let body = response.assert_html_response().await;

    // Should contain escaped/safe versions of special characters
    assert_body_contains(&body, &["Admin Dashboard", "Test Article with"]);
}
