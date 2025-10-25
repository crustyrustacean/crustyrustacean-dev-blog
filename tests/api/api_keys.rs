// tests/api/api_keys.rs

use crate::helpers::{TestFixture, TestUserBuilder, spawn_app};
use crate::{assert_status, bearer_request, parse_json};
use reqwest::StatusCode;
use serde_json::{Value, json};

// ============================================================================
// Happy Path Tests
// ============================================================================

#[tokio::test]
async fn test_create_api_key_happy_path() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act - Create an API key
    let response = bearer_request!(
        post &app,
        format!("{}/api/keys", &app.address),
        &token,
        json!({
            "name": "My Test API Key"
        })
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body: Value = parse_json!(response);

    // Verify response structure
    assert!(body["id"].is_string(), "API key should have an ID");
    assert_eq!(body["name"], "My Test API Key");
    assert!(body["key"].is_string(), "API key should be returned");
    assert!(body["created_at"].is_string(), "Created timestamp should be present");

    // Verify the key format (should be a long string)
    let key = body["key"].as_str().unwrap();
    assert!(key.len() > 32, "API key should be sufficiently long");
}

#[tokio::test]
async fn test_list_api_keys_happy_path() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Create multiple API keys
    bearer_request!(
        post &app,
        format!("{}/api/keys", &app.address),
        &token,
        json!({"name": "Key 1"})
    )
    .expect("Failed to create key 1");

    bearer_request!(
        post &app,
        format!("{}/api/keys", &app.address),
        &token,
        json!({"name": "Key 2"})
    )
    .expect("Failed to create key 2");

    bearer_request!(
        post &app,
        format!("{}/api/keys", &app.address),
        &token,
        json!({"name": "Key 3"})
    )
    .expect("Failed to create key 3");

    // Act - List API keys
    let response = bearer_request!(
        get &app,
        format!("{}/api/keys", &app.address),
        &token
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body: Value = parse_json!(response);

    assert!(body["api_keys"].is_array(), "Response should contain api_keys array");
    let keys = body["api_keys"].as_array().unwrap();
    assert_eq!(keys.len(), 3, "Should have 3 API keys");

    // Verify structure of each key (key value should NOT be included)
    for key in keys {
        assert!(key["id"].is_string());
        assert!(key["name"].is_string());
        assert!(key["created_at"].is_string());
        assert!(key.get("key").is_none(), "Raw key should NOT be included in list");
    }

    // Verify keys are ordered by created_at DESC (most recent first)
    assert_eq!(keys[0]["name"], "Key 3");
    assert_eq!(keys[1]["name"], "Key 2");
    assert_eq!(keys[2]["name"], "Key 1");
}

#[tokio::test]
async fn test_delete_api_key_happy_path() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Create an API key
    let create_response = bearer_request!(
        post &app,
        format!("{}/api/keys", &app.address),
        &token,
        json!({"name": "Key to Delete"})
    )
    .expect("Failed to create key");
    let create_body: Value = parse_json!(create_response);
    let key_id = create_body["id"].as_str().unwrap();

    // Act - Delete the API key
    let response = bearer_request!(
        delete &app,
        format!("{}/api/keys/{}", &app.address, key_id),
        &token
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body: Value = parse_json!(response);
    assert_eq!(body["message"], "API key deleted");

    // Verify the key is actually deleted by listing
    let list_response = bearer_request!(
        get &app,
        format!("{}/api/keys", &app.address),
        &token
    )
    .expect("Failed to list keys");
    let list_body: Value = parse_json!(list_response);
    let keys = list_body["api_keys"].as_array().unwrap();
    assert_eq!(keys.len(), 0, "Key should be deleted");
}

#[tokio::test]
async fn test_multiple_api_keys_per_user() {
    // Arrange - Multiple users with their own API keys
    let fixture = TestFixture::new()
        .await
        .with_user_default("user1")
        .await
        .with_user_default("user2")
        .await;

    let token1 = fixture.get_token("user1");
    let token2 = fixture.get_token("user2");

    // Act - Each user creates API keys
    bearer_request!(
        post &fixture.app,
        format!("{}/api/keys", &fixture.app.address),
        &token1,
        json!({"name": "User1 Key"})
    )
    .expect("Failed to create user1 key");

    bearer_request!(
        post &fixture.app,
        format!("{}/api/keys", &fixture.app.address),
        &token2,
        json!({"name": "User2 Key"})
    )
    .expect("Failed to create user2 key");

    // Assert - Each user sees only their own keys
    let list1 = bearer_request!(
        get &fixture.app,
        format!("{}/api/keys", &fixture.app.address),
        &token1
    )
    .expect("Failed to list user1 keys");
    let body1: Value = parse_json!(list1);
    let keys1 = body1["api_keys"].as_array().unwrap();
    assert_eq!(keys1.len(), 1);
    assert_eq!(keys1[0]["name"], "User1 Key");

    let list2 = bearer_request!(
        get &fixture.app,
        format!("{}/api/keys", &fixture.app.address),
        &token2
    )
    .expect("Failed to list user2 keys");
    let body2: Value = parse_json!(list2);
    let keys2 = body2["api_keys"].as_array().unwrap();
    assert_eq!(keys2.len(), 1);
    assert_eq!(keys2[0]["name"], "User2 Key");
}

// ============================================================================
// Unhappy Path Tests
// ============================================================================

#[tokio::test]
async fn test_create_api_key_without_authentication() {
    // Arrange
    let app = spawn_app().await;

    // Act - Try to create API key without auth
    let response = app
        .client
        .post(format!("{}/api/keys", &app.address))
        .header("Content-Type", "application/json")
        .json(&json!({"name": "Unauthorized Key"}))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_api_key_with_empty_name() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act - Try to create API key with empty name
    let response = bearer_request!(
        post &app,
        format!("{}/api/keys", &app.address),
        &token,
        json!({"name": ""})
    )
    .expect("Failed to execute request");

    // Assert - Should fail validation
    assert_status!(response, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_api_key_with_name_too_long() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act - Try to create API key with name exceeding 100 characters
    let long_name = "a".repeat(101);
    let response = bearer_request!(
        post &app,
        format!("{}/api/keys", &app.address),
        &token,
        json!({"name": long_name})
    )
    .expect("Failed to execute request");

    // Assert - Should fail validation
    assert_status!(response, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_api_key_with_invalid_payload() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act - Try to create API key with missing name field
    let response = bearer_request!(
        post &app,
        format!("{}/api/keys", &app.address),
        &token,
        json!({})
    )
    .expect("Failed to execute request");

    // Assert - Should fail validation
    assert!(
        response.status() == StatusCode::BAD_REQUEST || 
        response.status() == StatusCode::UNPROCESSABLE_ENTITY,
        "Expected 400 or 422 status code"
    );
}

#[tokio::test]
async fn test_list_api_keys_without_authentication() {
    // Arrange
    let app = spawn_app().await;

    // Act - Try to list API keys without auth
    let response = app
        .client
        .get(format!("{}/api/keys", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_list_api_keys_empty_list() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act - List API keys when none exist
    let response = bearer_request!(
        get &app,
        format!("{}/api/keys", &app.address),
        &token
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body: Value = parse_json!(response);
    assert!(body["api_keys"].is_array());
    let keys = body["api_keys"].as_array().unwrap();
    assert_eq!(keys.len(), 0, "Should have no API keys");
}

#[tokio::test]
async fn test_delete_api_key_without_authentication() {
    // Arrange
    let app = spawn_app().await;
    let fake_key_id = "550e8400-e29b-41d4-a716-446655440000";

    // Act - Try to delete API key without auth
    let response = app
        .client
        .delete(format!("{}/api/keys/{}", &app.address, fake_key_id))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_delete_nonexistent_api_key() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;
    let fake_key_id = "550e8400-e29b-41d4-a716-446655440000";

    // Act - Try to delete non-existent API key
    let response = bearer_request!(
        delete &app,
        format!("{}/api/keys/{}", &app.address, fake_key_id),
        &token
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_another_users_api_key() {
    // Arrange - Create two users with their own keys
    let fixture = TestFixture::new()
        .await
        .with_user_default("user1")
        .await
        .with_user_default("user2")
        .await;

    let token1 = fixture.get_token("user1");
    let token2 = fixture.get_token("user2");

    // User1 creates an API key
    let create_response = bearer_request!(
        post &fixture.app,
        format!("{}/api/keys", &fixture.app.address),
        &token1,
        json!({"name": "User1 Key"})
    )
    .expect("Failed to create key");
    let create_body: Value = parse_json!(create_response);
    let user1_key_id = create_body["id"].as_str().unwrap();

    // Act - User2 tries to delete User1's key
    let response = bearer_request!(
        delete &fixture.app,
        format!("{}/api/keys/{}", &fixture.app.address, user1_key_id),
        &token2
    )
    .expect("Failed to execute request");

    // Assert - Should return NOT_FOUND (not FORBIDDEN, for security reasons)
    assert_status!(response, StatusCode::NOT_FOUND);

    // Verify User1's key still exists
    let list_response = bearer_request!(
        get &fixture.app,
        format!("{}/api/keys", &fixture.app.address),
        &token1
    )
    .expect("Failed to list keys");
    let list_body: Value = parse_json!(list_response);
    let keys = list_body["api_keys"].as_array().unwrap();
    assert_eq!(keys.len(), 1, "User1's key should still exist");
}

#[tokio::test]
async fn test_delete_api_key_with_invalid_uuid() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act - Try to delete with invalid UUID format
    let response = bearer_request!(
        delete &app,
        format!("{}/api/keys/not-a-valid-uuid", &app.address),
        &token
    )
    .expect("Failed to execute request");

    // Assert - Should return bad request or not found
    assert!(
        response.status() == StatusCode::BAD_REQUEST ||
        response.status() == StatusCode::NOT_FOUND,
        "Expected 400 or 404 status code for invalid UUID"
    );
}

#[tokio::test]
async fn test_api_key_idempotent_deletion() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Create an API key
    let create_response = bearer_request!(
        post &app,
        format!("{}/api/keys", &app.address),
        &token,
        json!({"name": "Key to Delete Twice"})
    )
    .expect("Failed to create key");
    let create_body: Value = parse_json!(create_response);
    let key_id = create_body["id"].as_str().unwrap();

    // Delete once (should succeed)
    let response1 = bearer_request!(
        delete &app,
        format!("{}/api/keys/{}", &app.address, key_id),
        &token
    )
    .expect("Failed to delete first time");
    assert_status!(response1, StatusCode::OK);

    // Act - Try to delete again
    let response2 = bearer_request!(
        delete &app,
        format!("{}/api/keys/{}", &app.address, key_id),
        &token
    )
    .expect("Failed to execute request");

    // Assert - Second deletion should fail (key doesn't exist)
    assert_status!(response2, StatusCode::NOT_FOUND);
}
