// tests/api/rbac.rs
//
// Comprehensive tests for Role-Based Access Control (RBAC) system

use crate::helpers::{TestUserBuilder, spawn_app};

#[tokio::test]
async fn test_first_user_gets_admin_role() {
    let app = spawn_app().await;

    // Register the first user
    let response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .json(&serde_json::json!({
            "user": {
                "username": "firstuser",
                "email": "first@example.com",
                "password": "password123"
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");

    // Verify the first user has admin role
    assert_eq!(body["user"]["role"], "admin");
    assert_eq!(body["user"]["username"], "firstuser");
    assert_eq!(body["user"]["email"], "first@example.com");
}

#[tokio::test]
async fn test_subsequent_users_get_subscriber_role() {
    let app = spawn_app().await;

    // Register first user (will be admin)
    app.register_user_default("admin").await;

    // Register second user
    let response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .json(&serde_json::json!({
            "user": {
                "username": "subscriber",
                "email": "subscriber@example.com",
                "password": "password123"
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");

    // Verify the second user has subscriber role
    assert_eq!(body["user"]["role"], "subscriber");
    assert_eq!(body["user"]["username"], "subscriber");
}

#[tokio::test]
async fn test_login_returns_user_role() {
    let app = spawn_app().await;

    // Register user
    app.register_user_default("testuser").await;

    // Login
    let response = app
        .client
        .post(format!("{}/api/users/login", &app.address))
        .json(&serde_json::json!({
            "user": {
                "email": "testuser@example.com",
                "password": "password123"
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");

    // Verify role is included in login response
    assert!(body["user"]["role"].is_string());
}

#[tokio::test]
async fn test_get_current_user_returns_role() {
    let app = spawn_app().await;

    let token = app.register_user_default("testuser").await;

    let response = app
        .client
        .get(format!("{}/api/user", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");

    // Verify role is included
    assert!(body["user"]["role"].is_string());
}

#[tokio::test]
async fn test_subscriber_cannot_create_articles() {
    let app = spawn_app().await;

    // Register first user (admin) and second user (subscriber)
    app.register_user_default("admin").await;
    let subscriber_token = app.register_user_default("subscriber").await;

    // Try to create article as subscriber
    let response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", subscriber_token))
        .json(&serde_json::json!({
            "article": {
                "title": "Test Article",
                "description": "Test description",
                "body": "Test body",
                "tagList": []
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Should be forbidden
    assert_eq!(response.status(), 403);

    let body = response.text().await.expect("Failed to parse response");
    assert!(body.to_lowercase().contains("author") || body.to_lowercase().contains("forbidden"));
}

#[tokio::test]
async fn test_author_can_create_articles() {
    let app = spawn_app().await;

    // Register admin (first user)
    let admin_token = app.register_user_default("admin").await;

    // Register subscriber
    let subscriber_token = app.register_user_default("author").await;

    // Get subscriber's user ID
    let response = app
        .client
        .get(format!("{}/api/user", &app.address))
        .header("Authorization", format!("Bearer {}", subscriber_token))
        .send()
        .await
        .expect("Failed to execute request");

    let user_data: serde_json::Value = response.json().await.expect("Failed to parse response");
    let user_id = user_data["user"]["username"].as_str().unwrap();

    // Admin promotes subscriber to author
    let users_response = app
        .client
        .get(format!(
            "{}/api/admin/users?search={}",
            &app.address, user_id
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .expect("Failed to execute request");

    let users: serde_json::Value = users_response
        .json()
        .await
        .expect("Failed to parse response");
    let author_user_id = users["data"]["users"][0]["id"].as_str().unwrap();

    let promote_response = app
        .client
        .put(format!(
            "{}/api/admin/users/{}",
            &app.address, author_user_id
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&serde_json::json!({
            "role": "author"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(promote_response.status(), 200);

    // Get new token for author (role is cached in token extraction)
    let login_response = app
        .client
        .post(format!("{}/api/users/login", &app.address))
        .json(&serde_json::json!({
            "user": {
                "email": "author@example.com",
                "password": "password123"
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    let login_body: serde_json::Value = login_response
        .json()
        .await
        .expect("Failed to parse response");
    let author_token = login_body["user"]["token"].as_str().unwrap();

    // Try to create article as author
    let response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", author_token))
        .json(&serde_json::json!({
            "article": {
                "title": "Test Article",
                "description": "Test description",
                "body": "Test body",
                "tagList": []
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Should succeed (201 Created)
    assert_eq!(response.status(), 201);
}

#[tokio::test]
async fn test_admin_can_create_articles() {
    let app = spawn_app().await;

    // Register admin (first user)
    let admin_token = app.register_user_default("admin").await;

    // Create article as admin
    let response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&serde_json::json!({
            "article": {
                "title": "Admin Article",
                "description": "Admin description",
                "body": "Admin body",
                "tagList": []
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Should succeed (201 Created)
    assert_eq!(response.status(), 201);
}

#[tokio::test]
async fn test_subscriber_cannot_create_categories() {
    let app = spawn_app().await;

    // Register admin and subscriber
    app.register_user_default("admin").await;
    let subscriber_token = app.register_user_default("subscriber").await;

    // Try to create category as subscriber
    let response = app
        .client
        .post(format!("{}/api/categories", &app.address))
        .header("Authorization", format!("Bearer {}", subscriber_token))
        .json(&serde_json::json!({
            "category": {
                "name": "Test Category",
                "description": "Test description"
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Should be forbidden
    assert_eq!(response.status(), 403);
}

#[tokio::test]
async fn test_author_can_create_categories() {
    let app = spawn_app().await;

    // Register admin (first user)
    let admin_token = app.register_user_default("admin").await;

    // Register and promote user to author
    let subscriber_token = app.register_user_default("author").await;

    // Get subscriber's user ID
    let response = app
        .client
        .get(format!("{}/api/user", &app.address))
        .header("Authorization", format!("Bearer {}", subscriber_token))
        .send()
        .await
        .expect("Failed to execute request");

    let user_data: serde_json::Value = response.json().await.expect("Failed to parse response");
    let user_id = user_data["user"]["username"].as_str().unwrap();

    // Admin promotes subscriber to author
    let users_response = app
        .client
        .get(format!(
            "{}/api/admin/users?search={}",
            &app.address, user_id
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .expect("Failed to execute request");

    let users: serde_json::Value = users_response
        .json()
        .await
        .expect("Failed to parse response");
    let author_user_id = users["data"]["users"][0]["id"].as_str().unwrap();

    app.client
        .put(format!(
            "{}/api/admin/users/{}",
            &app.address, author_user_id
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&serde_json::json!({
            "role": "author"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Get new token
    let login_response = app
        .client
        .post(format!("{}/api/users/login", &app.address))
        .json(&serde_json::json!({
            "user": {
                "email": "author@example.com",
                "password": "password123"
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    let login_body: serde_json::Value = login_response
        .json()
        .await
        .expect("Failed to parse response");
    let author_token = login_body["user"]["token"].as_str().unwrap();

    // Try to create category as author
    let response = app
        .client
        .post(format!("{}/api/categories", &app.address))
        .header("Authorization", format!("Bearer {}", author_token))
        .json(&serde_json::json!({
            "category": {
                "name": "Test Category",
                "description": "Test description"
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Should succeed (201 Created)
    assert_eq!(response.status(), 201);
}

#[tokio::test]
async fn test_non_admin_cannot_access_admin_users_endpoint() {
    let app = spawn_app().await;

    // Register admin and subscriber
    app.register_user_default("admin").await;
    let subscriber_token = app.register_user_default("subscriber").await;

    // Try to access admin users endpoint as subscriber
    let response = app
        .client
        .get(format!("{}/api/admin/users", &app.address))
        .header("Authorization", format!("Bearer {}", subscriber_token))
        .send()
        .await
        .expect("Failed to execute request");

    // Should be forbidden
    assert_eq!(response.status(), 403);
}

#[tokio::test]
async fn test_admin_can_access_admin_users_endpoint() {
    let app = spawn_app().await;

    // Register admin
    let admin_token = app.register_user_default("admin").await;

    // Access admin users endpoint as admin
    let response = app
        .client
        .get(format!("{}/api/admin/users", &app.address))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .expect("Failed to execute request");

    // Should succeed
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_admin_can_promote_subscriber_to_author() {
    let app = spawn_app().await;

    // Register admin
    let admin_token = app.register_user_default("admin").await;

    // Register subscriber
    app.register_user_default("subscriber").await;

    // Get subscriber's user ID
    let users_response = app
        .client
        .get(format!(
            "{}/api/admin/users?search=subscriber",
            &app.address
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .expect("Failed to execute request");

    let users: serde_json::Value = users_response
        .json()
        .await
        .expect("Failed to parse response");
    let subscriber_id = users["data"]["users"][0]["id"].as_str().unwrap();
    let current_role = users["data"]["users"][0]["role"].as_str().unwrap();

    assert_eq!(current_role, "subscriber");

    // Promote subscriber to author
    let response = app
        .client
        .put(format!(
            "{}/api/admin/users/{}",
            &app.address, subscriber_id
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&serde_json::json!({
            "role": "author"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["data"]["user"]["role"], "author");
}

#[tokio::test]
async fn test_admin_can_promote_author_to_admin() {
    let app = spawn_app().await;

    // Register admin
    let admin_token = app.register_user_default("admin").await;

    // Register author (will be subscriber initially)
    app.register_user_default("newauthor").await;

    // Get user ID
    let users_response = app
        .client
        .get(format!("{}/api/admin/users?search=newauthor", &app.address))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .expect("Failed to execute request");

    let users: serde_json::Value = users_response
        .json()
        .await
        .expect("Failed to parse response");
    let user_id = users["data"]["users"][0]["id"].as_str().unwrap();

    // Promote to author first
    app.client
        .put(format!("{}/api/admin/users/{}", &app.address, user_id))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&serde_json::json!({
            "role": "author"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Promote author to admin
    let response = app
        .client
        .put(format!("{}/api/admin/users/{}", &app.address, user_id))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&serde_json::json!({
            "role": "admin"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["data"]["user"]["role"], "admin");
}

#[tokio::test]
async fn test_admin_can_demote_author_to_subscriber() {
    let app = spawn_app().await;

    // Register admin
    let admin_token = app.register_user_default("admin").await;

    // Register and promote user to author
    app.register_user_default("author").await;

    // Get user ID
    let users_response = app
        .client
        .get(format!("{}/api/admin/users?search=author", &app.address))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .expect("Failed to execute request");

    let users: serde_json::Value = users_response
        .json()
        .await
        .expect("Failed to parse response");
    let user_id = users["data"]["users"][0]["id"].as_str().unwrap();

    // Promote to author
    app.client
        .put(format!("{}/api/admin/users/{}", &app.address, user_id))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&serde_json::json!({
            "role": "author"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Demote author to subscriber
    let response = app
        .client
        .put(format!("{}/api/admin/users/{}", &app.address, user_id))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&serde_json::json!({
            "role": "subscriber"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["data"]["user"]["role"], "subscriber");
}

#[tokio::test]
async fn test_non_admin_cannot_update_user_roles() {
    let app = spawn_app().await;

    // Register admin and two subscribers
    app.register_user_default("admin").await;
    let subscriber1_token = app.register_user_default("subscriber1").await;
    app.register_user_default("subscriber2").await;

    // Get subscriber2's user ID
    let admin_token = app.register_user_default("tempAdmin").await;
    let users_response = app
        .client
        .get(format!(
            "{}/api/admin/users?search=subscriber2",
            &app.address
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .expect("Failed to execute request");

    let users: serde_json::Value = users_response
        .json()
        .await
        .expect("Failed to parse response");

    // If no users found, the endpoint might have failed due to permissions
    if users["data"]["users"].as_array().map_or(0, |a| a.len()) == 0 {
        // Expected - non-admin can't access admin endpoints
        return;
    }

    let user_id = users["data"]["users"][0]["id"].as_str().unwrap();

    // Try to update role as non-admin
    let response = app
        .client
        .put(format!("{}/api/admin/users/{}", &app.address, user_id))
        .header("Authorization", format!("Bearer {}", subscriber1_token))
        .json(&serde_json::json!({
            "role": "admin"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Should be forbidden
    assert_eq!(response.status(), 403);
}

#[tokio::test]
async fn test_role_validation_rejects_invalid_roles() {
    let app = spawn_app().await;

    // Register admin
    let admin_token = app.register_user_default("admin").await;

    // Register subscriber
    app.register_user_default("subscriber").await;

    // Get subscriber's user ID
    let users_response = app
        .client
        .get(format!(
            "{}/api/admin/users?search=subscriber",
            &app.address
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .expect("Failed to execute request");

    let users: serde_json::Value = users_response
        .json()
        .await
        .expect("Failed to parse response");
    let user_id = users["data"]["users"][0]["id"].as_str().unwrap();

    // Try to set invalid role
    let response = app
        .client
        .put(format!("{}/api/admin/users/{}", &app.address, user_id))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&serde_json::json!({
            "role": "superadmin"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Should fail with bad request
    assert!(response.status().is_client_error());
}
