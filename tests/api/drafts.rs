// tests/api/drafts.rs - Integration tests for draft article functionality

use crate::helpers::{assert_body_contains, spawn_app, TestArticleBuilder, TestUserBuilder, HtmlResponseValidator};

#[tokio::test]
async fn test_create_draft_article() {
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Create a draft article
    let response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "article": {
                "title": "My Draft Article",
                "description": "This is a draft",
                "body": "Draft content",
                "draft": true
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 201);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    let article = &body["article"];
    assert_eq!(article["draft"], true);
    assert_eq!(article["title"], "My Draft Article");
}

#[tokio::test]
async fn test_list_user_drafts_api() {
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Create multiple drafts
    app.client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "article": {
                "title": "Draft 1",
                "description": "First draft",
                "body": "Content 1",
                "draft": true
            }
        }))
        .send()
        .await
        .expect("Failed to create draft 1");

    app.client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "article": {
                "title": "Draft 2",
                "description": "Second draft",
                "body": "Content 2",
                "draft": true
            }
        }))
        .send()
        .await
        .expect("Failed to create draft 2");

    // Create a published article (should not appear in drafts)
    app.create_article_simple(&token, "Published Article").await;

    // List drafts via API
    let response = app
        .client
        .get(format!("{}/api/articles/drafts", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    let articles = body["articles"].as_array().expect("Expected articles array");

    assert_eq!(articles.len(), 2);
    assert_eq!(body["articlesCount"], 2);

    // Verify both drafts are present
    let titles: Vec<&str> = articles
        .iter()
        .map(|a| a["title"].as_str().unwrap())
        .collect();
    assert!(titles.contains(&"Draft 1"));
    assert!(titles.contains(&"Draft 2"));
    assert!(!titles.contains(&"Published Article"));
}

#[tokio::test]
async fn test_drafts_admin_page() {
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Create a draft
    app.client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "article": {
                "title": "Test Draft",
                "description": "Draft description",
                "body": "Draft body",
                "draft": true
            }
        }))
        .send()
        .await
        .expect("Failed to create draft");

    // Access drafts admin page
    let response = app
        .client
        .get(format!("{}/admin/drafts", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body = response.assert_html_response().await;
    assert_body_contains(&body, &[
        "Draft Articles",
        "Test Draft",
        "Draft description",
        "testuser",
    ]);
}

#[tokio::test]
async fn test_publish_draft() {
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Create a draft
    let response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "article": {
                "title": "Draft to Publish",
                "description": "Will be published",
                "body": "Content",
                "draft": true
            }
        }))
        .send()
        .await
        .expect("Failed to create draft");

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    let slug = body["article"]["slug"].as_str().unwrap();

    // Publish the draft
    let response = app
        .client
        .put(format!("{}/api/articles/{}", &app.address, slug))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "article": {
                "draft": false
            }
        }))
        .send()
        .await
        .expect("Failed to publish draft");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["article"]["draft"], false);

    // Verify it no longer appears in drafts list
    let response = app
        .client
        .get(format!("{}/api/articles/drafts", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    let articles = body["articles"].as_array().expect("Expected articles array");
    assert_eq!(articles.len(), 0);
}

#[tokio::test]
async fn test_draft_visibility_author_only() {
    let app = spawn_app().await;
    let alice_token = app.register_user("alice", "alice@example.com", "password").await;
    let bob_token = app.register_user("bob", "bob@example.com", "password").await;

    // Alice creates a draft
    let response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", alice_token))
        .json(&serde_json::json!({
            "article": {
                "title": "Alice's Draft",
                "description": "Private draft",
                "body": "Secret content",
                "draft": true
            }
        }))
        .send()
        .await
        .expect("Failed to create draft");

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    let slug = body["article"]["slug"].as_str().unwrap();

    // Alice can view her own draft
    let response = app
        .client
        .get(format!("{}/api/articles/{}", &app.address, slug))
        .header("Authorization", format!("Bearer {}", alice_token))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    // Bob cannot view Alice's draft
    let response = app
        .client
        .get(format!("{}/api/articles/{}", &app.address, slug))
        .header("Authorization", format!("Bearer {}", bob_token))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 404);

    // Anonymous users cannot view the draft
    let response = app
        .client
        .get(format!("{}/api/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 404);

    // Bob's drafts list should be empty
    let response = app
        .client
        .get(format!("{}/api/articles/drafts", &app.address))
        .header("Authorization", format!("Bearer {}", bob_token))
        .send()
        .await
        .expect("Failed to execute request");

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    let articles = body["articles"].as_array().expect("Expected articles array");
    assert_eq!(articles.len(), 0);
}

#[tokio::test]
async fn test_draft_not_in_public_listing() {
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Create a draft and a published article
    app.client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "article": {
                "title": "Draft Article",
                "description": "Should not appear",
                "body": "Content",
                "draft": true
            }
        }))
        .send()
        .await
        .expect("Failed to create draft");

    app.create_article_simple(&token, "Published Article").await;

    // Check public articles list
    let response = app
        .client
        .get(format!("{}/api/articles", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    let articles = body["articles"].as_array().expect("Expected articles array");

    // Only the published article should appear
    assert_eq!(articles.len(), 1);
    assert_eq!(articles[0]["title"], "Published Article");
}

#[tokio::test]
async fn test_delete_draft() {
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Create a draft
    let response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "article": {
                "title": "Draft to Delete",
                "description": "Will be deleted",
                "body": "Content",
                "draft": true
            }
        }))
        .send()
        .await
        .expect("Failed to create draft");

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    let slug = body["article"]["slug"].as_str().unwrap();

    // Delete the draft
    let response = app
        .client
        .delete(format!("{}/api/articles/{}", &app.address, slug))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to delete draft");

    assert_eq!(response.status(), 204);

    // Verify it's deleted
    let response = app
        .client
        .get(format!("{}/api/articles/{}", &app.address, slug))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_empty_drafts_page() {
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Access drafts page with no drafts
    let response = app
        .client
        .get(format!("{}/admin/drafts", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body = response.assert_html_response().await;
    assert_body_contains(&body, &[
        "Draft Articles",
        "No Drafts Yet",
        "Create Your First Draft",
    ]);
}

#[tokio::test]
async fn test_drafts_require_authentication() {
    let app = spawn_app().await;

    // Try to access drafts API without authentication
    let response = app
        .client
        .get(format!("{}/api/articles/drafts", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 401);

    // Try to access drafts page without authentication
    let response = app
        .client
        .get(format!("{}/admin/drafts", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 401);
}
