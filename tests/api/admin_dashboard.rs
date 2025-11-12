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

#[tokio::test]
async fn test_admin_dashboard_displays_article_slugs() {
    // Arrange
    let app = spawn_app().await;

    // Register user and get token
    let token = app.register_user_default("slugtest").await;

    // Create an article with a known title (slug will be auto-generated)
    let slug = app
        .create_article(
            &token,
            "My Great Article Title",
            "A description for slug testing",
            "Body content for slug test.",
            vec!["test"],
        )
        .await;

    // Act - Access admin dashboard
    let response = app
        .client
        .get(format!("{}/admin", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should display slug in a dedicated column
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.assert_html_response().await;

    // Should contain table header for Slug column
    assert_body_contains(&body, &["Slug"]);

    // Should contain the actual slug value in the table
    assert!(
        body.contains(&slug),
        "Admin dashboard should display the article slug: {}",
        slug
    );

    // Should have a copy button for the slug (with data-slug attribute)
    assert!(
        body.contains(&format!("data-slug=\"{}\"", slug)),
        "Admin dashboard should have a copy button with data-slug attribute"
    );
}

#[tokio::test]
async fn test_admin_dashboard_displays_multiple_article_slugs() {
    // Arrange
    let app = spawn_app().await;

    // Register user and get token
    let token = app.register_user_default("multislug").await;

    // Create multiple articles with different titles
    let slug1 = app
        .create_article_simple(&token, "First Article")
        .await;

    let slug2 = app
        .create_article_simple(&token, "Second Article")
        .await;

    let slug3 = app
        .create_article_simple(&token, "Third Article")
        .await;

    // Act - Access admin dashboard
    let response = app
        .client
        .get(format!("{}/admin", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should display all slugs
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.assert_html_response().await;

    // All slugs should be present
    assert!(
        body.contains(&slug1),
        "Dashboard should contain first article slug: {}",
        slug1
    );
    assert!(
        body.contains(&slug2),
        "Dashboard should contain second article slug: {}",
        slug2
    );
    assert!(
        body.contains(&slug3),
        "Dashboard should contain third article slug: {}",
        slug3
    );
}

#[tokio::test]
async fn test_admin_dashboard_pagination_with_more_than_10_articles() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("paginationuser").await;

    // Create 15 articles to trigger pagination (default limit is 10)
    for i in 1..=15 {
        app.create_article_simple(&token, &format!("Article {}", i))
            .await;
    }

    // Act - Access admin dashboard page 1
    let response = app
        .client
        .get(format!("{}/admin", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should show pagination controls
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.assert_html_response().await;

    // Should contain pagination info
    assert_body_contains(
        &body,
        &[
            "Showing 10 of 15 articles",
            "Page 1 of 2",
            "Next &raquo;",
        ],
    );

    // Should have page 2 link
    assert!(
        body.contains("/admin?offset=10&limit=10"),
        "Should have link to page 2"
    );
}

#[tokio::test]
async fn test_admin_dashboard_pagination_page_2() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("page2user").await;

    // Create 15 articles
    for i in 1..=15 {
        app.create_article_simple(&token, &format!("Article {}", i))
            .await;
    }

    // Act - Access admin dashboard page 2
    let response = app
        .client
        .get(format!("{}/admin?offset=10&limit=10", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should show page 2 with correct pagination
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.assert_html_response().await;

    // Should contain pagination info for page 2
    assert_body_contains(
        &body,
        &[
            "Showing 5 of 15 articles",
            "Page 2 of 2",
            "&laquo; Previous",
        ],
    );

    // Should have previous page link
    assert!(
        body.contains("/admin?offset=0&limit=10"),
        "Should have link to page 1"
    );

    // Should show active page 2
    assert!(
        body.contains(r#"<li class="page-item active">"#),
        "Should have active page indicator"
    );
}

#[tokio::test]
async fn test_admin_dashboard_no_pagination_with_less_than_10_articles() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("nopaginationuser").await;

    // Create only 5 articles (less than the 10 per page limit)
    for i in 1..=5 {
        app.create_article_simple(&token, &format!("Article {}", i))
            .await;
    }

    // Act - Access admin dashboard
    let response = app
        .client
        .get(format!("{}/admin", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should NOT show pagination controls
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.assert_html_response().await;

    // Should NOT contain pagination controls
    assert!(
        !body.contains("Page 1 of"),
        "Should not show pagination with less than 10 articles"
    );
    assert!(
        !body.contains("Next &raquo;"),
        "Should not show next button"
    );
}

#[tokio::test]
async fn test_admin_dashboard_does_not_show_description_in_table() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("descriptiontest").await;

    // Create an article with a long description
    app.create_article(
        &token,
        "Test Article",
        "This is a very long description that should not appear in the admin dashboard table to keep rows compact",
        "Body content",
        vec!["test"],
    )
    .await;

    // Act - Access admin dashboard
    let response = app
        .client
        .get(format!("{}/admin", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Description should not be in the table
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.assert_html_response().await;

    // Should contain the title
    assert!(
        body.contains("Test Article"),
        "Should contain article title"
    );

    // Should NOT contain the description in a small text-muted element
    // (The description text itself might appear elsewhere, but not in the table row)
    assert!(
        !body.contains(r#"<small class="text-muted">This is a very long description"#),
        "Should not show description in table to keep rows compact"
    );
}

#[tokio::test]
async fn test_admin_dashboard_pagination_excludes_drafts_from_count() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("draftcountuser").await;

    // Create 12 published articles
    for i in 1..=12 {
        let slug = app
            .create_article_simple(&token, &format!("Published Article {}", i))
            .await;

        // Don't set as draft - they're published by default
        let _ = slug;
    }

    // Create 8 draft articles
    for i in 1..=8 {
        app.client
            .post(format!("{}/api/articles", &app.address))
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({
                "article": {
                    "title": format!("Draft Article {}", i),
                    "description": "This is a draft",
                    "body": "Draft content",
                    "draft": true
                }
            }))
            .send()
            .await
            .expect("Failed to create draft article");
    }

    // Total: 12 published + 8 drafts = 20 articles
    // But pagination should only count the 12 published articles

    // Act - Access admin dashboard page 1
    let response = app
        .client
        .get(format!("{}/admin", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should show pagination based on published articles only
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.assert_html_response().await;

    // Should show 2 pages (12 published / 10 per page = 2 pages)
    // NOT 3 pages (20 total / 10 per page = 2 pages)
    assert_body_contains(
        &body,
        &[
            "Showing 10 of 12 articles", // Should count only published articles
            "Page 1 of 2",                // Should have 2 pages, not 3
        ],
    );

    // Act - Access page 2 (last page)
    let response_page2 = app
        .client
        .get(format!("{}/admin?offset=10&limit=10", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Page 2 should show 2 remaining articles and NO next button
    assert_eq!(response_page2.status(), StatusCode::OK);
    let body_page2 = response_page2.assert_html_response().await;

    assert_body_contains(
        &body_page2,
        &[
            "Showing 2 of 12 articles", // Only 2 articles on last page
            "Page 2 of 2",               // This is the last page
        ],
    );

    // Next button should be DISABLED
    assert!(
        !body_page2.contains(r#"<a class="page-link" href="/admin?offset=20"#),
        "Next button should not be clickable on last page"
    );

    // Should show disabled next button
    assert!(
        body_page2.contains(r#"<li class="page-item disabled">"#)
            && body_page2.contains(r#"<span class="page-link">Next &raquo;</span>"#),
        "Next button should be disabled on last page"
    );
}
