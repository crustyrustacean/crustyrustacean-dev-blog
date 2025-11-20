// tests/api/admin_users.rs

use crate::helpers::{HtmlResponseValidator, TestUserBuilder, assert_body_contains, spawn_app};

#[tokio::test]
async fn admin_users_page_requires_authentication() {
    let app = spawn_app().await;

    let response = app
        .client
        .get(format!("{}/admin/users", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 401, "Expected 401 Unauthorized");
}

#[tokio::test]
async fn admin_users_page_loads_for_authenticated_user() {
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    let response = app
        .client
        .get(format!("{}/admin/users", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    let body = response.assert_html_response().await;
    assert_body_contains(&body, &["User Management", "Total Users"]);
}

#[tokio::test]
async fn list_users_api_requires_authentication() {
    let app = spawn_app().await;

    let response = app
        .client
        .get(format!("{}/api/admin/users", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 401, "Expected 401 Unauthorized");
}

#[tokio::test]
async fn list_users_api_returns_all_users() {
    let app = spawn_app().await;

    // Create multiple users
    let token1 = app.register_user_default("alice").await;
    app.register_user_default("bob").await;
    app.register_user_default("charlie").await;

    let response = app
        .client
        .get(format!("{}/api/admin/users", &app.address))
        .header("Authorization", format!("Bearer {}", token1))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    let users = body["data"]["users"]
        .as_array()
        .expect("Expected users array");

    assert_eq!(users.len(), 3, "Expected 3 users");

    // Verify user data structure
    let alice = users
        .iter()
        .find(|u| u["username"] == "alice")
        .expect("Alice not found");
    assert_eq!(alice["email"], "alice@example.com");
    assert_eq!(alice["disabled"], false);
    assert!(alice["articleCount"].is_number());
}

#[tokio::test]
async fn list_users_api_filters_by_search() {
    let app = spawn_app().await;

    let token = app.register_user_default("alice").await;
    app.register_user_default("bob").await;
    app.register_user_default("charlie").await;

    let response = app
        .client
        .get(format!("{}/api/admin/users?search=ali", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    let users = body["data"]["users"]
        .as_array()
        .expect("Expected users array");

    assert_eq!(users.len(), 1, "Expected 1 user matching search");
    assert_eq!(users[0]["username"], "alice");
}

#[tokio::test]
async fn list_users_api_filters_by_status() {
    let app = spawn_app().await;

    let token = app.register_user_default("alice").await;
    app.register_user_default("bob").await;

    // Get all users to find Bob's ID
    let list_response = app
        .client
        .get(format!("{}/api/admin/users", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to list users");

    let list_body: serde_json::Value = list_response.json().await.expect("Failed to parse JSON");
    let users = list_body["data"]["users"]
        .as_array()
        .expect("Expected users array");
    let bob = users
        .iter()
        .find(|u| u["username"] == "bob")
        .expect("Bob not found");
    let bob_id = bob["id"].as_str().expect("Expected bob ID");

    // Disable Bob
    app.client
        .put(format!("{}/api/admin/users/{}", &app.address, bob_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(r#"{"disabled": true}"#)
        .send()
        .await
        .expect("Failed to disable user");

    // Filter for active users only
    let response = app
        .client
        .get(format!("{}/api/admin/users?status=active", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    let users = body["data"]["users"]
        .as_array()
        .expect("Expected users array");

    assert_eq!(users.len(), 1, "Expected 1 active user");
    assert_eq!(users[0]["username"], "alice");

    // Filter for disabled users only
    let response2 = app
        .client
        .get(format!("{}/api/admin/users?status=disabled", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    let body2: serde_json::Value = response2.json().await.expect("Failed to parse JSON");
    let users2 = body2["data"]["users"]
        .as_array()
        .expect("Expected users array");

    assert_eq!(users2.len(), 1, "Expected 1 disabled user");
    assert_eq!(users2[0]["username"], "bob");
}

#[tokio::test]
async fn get_user_admin_returns_user_details() {
    let app = spawn_app().await;

    let token = app.register_user_default("alice").await;

    // Get Alice's user ID from list
    let list_response = app
        .client
        .get(format!("{}/api/admin/users", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to list users");

    let list_body: serde_json::Value = list_response.json().await.expect("Failed to parse JSON");
    let users = list_body["data"]["users"]
        .as_array()
        .expect("Expected users array");
    let alice = users
        .iter()
        .find(|u| u["username"] == "alice")
        .expect("Alice not found");
    let alice_id = alice["id"].as_str().expect("Expected alice ID");

    let response = app
        .client
        .get(format!("{}/api/admin/users/{}", &app.address, alice_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    let user = &body["data"]["user"];

    assert_eq!(user["username"], "alice");
    assert_eq!(user["email"], "alice@example.com");
    assert_eq!(user["disabled"], false);
}

#[tokio::test]
async fn update_user_admin_succeeds() {
    let app = spawn_app().await;

    let token = app.register_user_default("alice").await;

    // Get Alice's user ID
    let list_response = app
        .client
        .get(format!("{}/api/admin/users", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to list users");

    let list_body: serde_json::Value = list_response.json().await.expect("Failed to parse JSON");
    let users = list_body["data"]["users"]
        .as_array()
        .expect("Expected users array");
    let alice = users
        .iter()
        .find(|u| u["username"] == "alice")
        .expect("Alice not found");
    let alice_id = alice["id"].as_str().expect("Expected alice ID");

    let response = app
        .client
        .put(format!("{}/api/admin/users/{}", &app.address, alice_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(r#"{"username": "alice_updated", "bio": "Updated bio"}"#)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    let user = &body["data"]["user"];

    assert_eq!(user["username"], "alice_updated");
    assert_eq!(user["bio"], "Updated bio");
}

#[tokio::test]
async fn update_user_admin_can_disable_user() {
    let app = spawn_app().await;

    let token = app.register_user_default("alice").await;
    app.register_user_default("bob").await;

    // Get Bob's user ID
    let list_response = app
        .client
        .get(format!("{}/api/admin/users", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to list users");

    let list_body: serde_json::Value = list_response.json().await.expect("Failed to parse JSON");
    let users = list_body["data"]["users"]
        .as_array()
        .expect("Expected users array");
    let bob = users
        .iter()
        .find(|u| u["username"] == "bob")
        .expect("Bob not found");
    let bob_id = bob["id"].as_str().expect("Expected bob ID");

    let response = app
        .client
        .put(format!("{}/api/admin/users/{}", &app.address, bob_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(r#"{"disabled": true}"#)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    let user = &body["data"]["user"];

    assert_eq!(user["disabled"], true);
}

#[tokio::test]
async fn delete_user_admin_succeeds() {
    let app = spawn_app().await;

    let token = app.register_user_default("alice").await;
    app.register_user_default("bob").await;

    // Get Bob's user ID
    let list_response = app
        .client
        .get(format!("{}/api/admin/users", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to list users");

    let list_body: serde_json::Value = list_response.json().await.expect("Failed to parse JSON");
    let users = list_body["data"]["users"]
        .as_array()
        .expect("Expected users array");
    let bob = users
        .iter()
        .find(|u| u["username"] == "bob")
        .expect("Bob not found");
    let bob_id = bob["id"].as_str().expect("Expected bob ID");

    let response = app
        .client
        .delete(format!("{}/api/admin/users/{}", &app.address, bob_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    // Verify user is deleted
    let list_response2 = app
        .client
        .get(format!("{}/api/admin/users", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to list users");

    let list_body2: serde_json::Value = list_response2.json().await.expect("Failed to parse JSON");
    let users2 = list_body2["data"]["users"]
        .as_array()
        .expect("Expected users array");

    assert_eq!(users2.len(), 1, "Expected 1 user remaining");
    assert_eq!(users2[0]["username"], "alice");
}

#[tokio::test]
async fn delete_user_admin_prevents_self_deletion() {
    let app = spawn_app().await;

    let token = app.register_user_default("alice").await;

    // Get Alice's user ID
    let list_response = app
        .client
        .get(format!("{}/api/admin/users", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to list users");

    let list_body: serde_json::Value = list_response.json().await.expect("Failed to parse JSON");
    let users = list_body["data"]["users"]
        .as_array()
        .expect("Expected users array");
    let alice = users
        .iter()
        .find(|u| u["username"] == "alice")
        .expect("Alice not found");
    let alice_id = alice["id"].as_str().expect("Expected alice ID");

    let response = app
        .client
        .delete(format!("{}/api/admin/users/{}", &app.address, alice_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 400, "Expected 400 Bad Request");

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(
        body["errors"]["body"][0]
            .as_str()
            .unwrap()
            .contains("Cannot delete your own account")
    );
}

#[tokio::test]
async fn get_nonexistent_user_returns_404() {
    let app = spawn_app().await;

    let token = app.register_user_default("alice").await;

    let response = app
        .client
        .get(format!(
            "{}/api/admin/users/nonexistent-id-12345",
            &app.address
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 404, "Expected 404 Not Found");
}

#[tokio::test]
async fn update_user_validates_email_format() {
    let app = spawn_app().await;

    let token = app.register_user_default("alice").await;

    // Get Alice's user ID
    let list_response = app
        .client
        .get(format!("{}/api/admin/users", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to list users");

    let list_body: serde_json::Value = list_response.json().await.expect("Failed to parse JSON");
    let users = list_body["data"]["users"]
        .as_array()
        .expect("Expected users array");
    let alice = users
        .iter()
        .find(|u| u["username"] == "alice")
        .expect("Alice not found");
    let alice_id = alice["id"].as_str().expect("Expected alice ID");

    let response = app
        .client
        .put(format!("{}/api/admin/users/{}", &app.address, alice_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(r#"{"email": "invalid-email"}"#)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 422, "Expected 422 Unprocessable Entity");
}

#[tokio::test]
async fn update_user_validates_username_length() {
    let app = spawn_app().await;

    let token = app.register_user_default("alice").await;

    // Get Alice's user ID
    let list_response = app
        .client
        .get(format!("{}/api/admin/users", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to list users");

    let list_body: serde_json::Value = list_response.json().await.expect("Failed to parse JSON");
    let users = list_body["data"]["users"]
        .as_array()
        .expect("Expected users array");
    let alice = users
        .iter()
        .find(|u| u["username"] == "alice")
        .expect("Alice not found");
    let alice_id = alice["id"].as_str().expect("Expected alice ID");

    let response = app
        .client
        .put(format!("{}/api/admin/users/{}", &app.address, alice_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(r#"{"username": "ab"}"#)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 422, "Expected 422 Unprocessable Entity");
}
