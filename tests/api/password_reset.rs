// tests/api/password_reset.rs

use crate::helpers::{TestUserBuilder, spawn_app};
use serde_json::json;

#[tokio::test]
async fn password_reset_request_with_valid_email_returns_success() {
    // Arrange
    let app = spawn_app().await;
    let email = "test@example.com";

    // Register user first
    app.register_user("testuser", email, "password123").await;

    // Act - Request password reset
    let response = app
        .client
        .post(format!("{}/api/password-reset/request", &app.address))
        .json(&json!({ "email": email }))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to deserialize response");

    assert!(
        body["data"]["message"]
            .as_str()
            .unwrap()
            .contains("password reset instructions")
    );
}

#[tokio::test]
async fn password_reset_request_with_nonexistent_email_returns_success() {
    // Arrange
    let app = spawn_app().await;

    // Act - Request password reset for non-existent email
    let response = app
        .client
        .post(format!("{}/api/password-reset/request", &app.address))
        .json(&json!({ "email": "nonexistent@example.com" }))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should still return success to prevent email enumeration
    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to deserialize response");

    assert!(
        body["data"]["message"]
            .as_str()
            .unwrap()
            .contains("password reset instructions")
    );
}

#[tokio::test]
async fn password_reset_request_with_invalid_email_returns_error() {
    // Arrange
    let app = spawn_app().await;

    // Act - Request password reset with invalid email format
    let response = app
        .client
        .post(format!("{}/api/password-reset/request", &app.address))
        .json(&json!({ "email": "not-an-email" }))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn password_reset_page_with_valid_token_loads_successfully() {
    // Arrange
    let app = spawn_app().await;
    let email = "test@example.com";

    // Register user and request password reset
    app.register_user("testuser", email, "password123").await;
    app.client
        .post(format!("{}/api/password-reset/request", &app.address))
        .json(&json!({ "email": email }))
        .send()
        .await
        .expect("Failed to execute request");

    // Get the token from database
    let token = {
        let conn = app.db.connect().expect("Failed to connect to database");
        let mut rows = conn
            .query(
                "SELECT token FROM password_reset_tokens ORDER BY created_at DESC LIMIT 1",
                (),
            )
            .await
            .expect("Failed to query token");

        let token: String = rows
            .next()
            .await
            .expect("Failed to get row")
            .expect("No token found")
            .get(0)
            .expect("Failed to get token");

        // Drop rows and conn before continuing
        drop(rows);
        drop(conn);
        token
    };

    // Act - Load password reset page
    let response = app
        .client
        .get(format!("{}/password-reset/{}", &app.address, token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);
    let body = response.text().await.expect("Failed to get response body");
    assert!(body.contains("Set New Password"));
}

#[tokio::test]
async fn password_reset_page_with_invalid_token_returns_error() {
    // Arrange
    let app = spawn_app().await;

    // Act - Try to load password reset page with invalid token
    let response = app
        .client
        .get(format!(
            "{}/password-reset/invalid-token-12345",
            &app.address
        ))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn complete_password_reset_with_valid_token_succeeds() {
    // Arrange
    let app = spawn_app().await;
    let email = "test@example.com";
    let old_password = "password123";
    let new_password = "newpassword456";

    // Register user and request password reset
    app.register_user("testuser", email, old_password).await;
    app.client
        .post(format!("{}/api/password-reset/request", &app.address))
        .json(&json!({ "email": email }))
        .send()
        .await
        .expect("Failed to execute request");

    // Get the token from database
    let token = {
        let conn = app.db.connect().expect("Failed to connect to database");
        let mut rows = conn
            .query(
                "SELECT token FROM password_reset_tokens ORDER BY created_at DESC LIMIT 1",
                (),
            )
            .await
            .expect("Failed to query token");

        let token: String = rows
            .next()
            .await
            .expect("Failed to get row")
            .expect("No token found")
            .get(0)
            .expect("Failed to get token");

        // Drop rows and conn before continuing
        drop(rows);
        drop(conn);
        token
    };

    // Act - Complete password reset
    let response = app
        .client
        .post(format!("{}/api/password-reset/{}", &app.address, token))
        .json(&json!({ "password": new_password }))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    // Verify old password no longer works
    let login_response = app
        .client
        .post(format!("{}/api/users/login", &app.address))
        .json(&json!({
            "user": {
                "email": email,
                "password": old_password
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(login_response.status().as_u16(), 401);

    // Verify new password works
    let login_response = app
        .client
        .post(format!("{}/api/users/login", &app.address))
        .json(&json!({
            "user": {
                "email": email,
                "password": new_password
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(login_response.status().as_u16(), 200);
}

#[tokio::test]
async fn password_reset_token_can_only_be_used_once() {
    // Arrange
    let app = spawn_app().await;
    let email = "test@example.com";

    // Register user and request password reset
    app.register_user("testuser", email, "password123").await;
    app.client
        .post(format!("{}/api/password-reset/request", &app.address))
        .json(&json!({ "email": email }))
        .send()
        .await
        .expect("Failed to execute request");

    // Get the token from database
    let token = {
        let conn = app.db.connect().expect("Failed to connect to database");
        let mut rows = conn
            .query(
                "SELECT token FROM password_reset_tokens ORDER BY created_at DESC LIMIT 1",
                (),
            )
            .await
            .expect("Failed to query token");

        let token: String = rows
            .next()
            .await
            .expect("Failed to get row")
            .expect("No token found")
            .get(0)
            .expect("Failed to get token");

        // Drop rows and conn before continuing
        drop(rows);
        drop(conn);
        token
    };

    // Act - Use the token once
    let response = app
        .client
        .post(format!("{}/api/password-reset/{}", &app.address, token))
        .json(&json!({ "password": "newpassword456" }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 200);

    // Try to use the same token again
    let response = app
        .client
        .post(format!("{}/api/password-reset/{}", &app.address, token))
        .json(&json!({ "password": "anotherpassword789" }))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should fail with 400
    assert_eq!(response.status().as_u16(), 400);
    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to deserialize response");
    assert_eq!(body["success"], false);
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("already been used")
    );
}

#[tokio::test]
async fn password_reset_with_too_short_password_fails() {
    // Arrange
    let app = spawn_app().await;
    let email = "test@example.com";

    // Register user and request password reset
    app.register_user("testuser", email, "password123").await;
    app.client
        .post(format!("{}/api/password-reset/request", &app.address))
        .json(&json!({ "email": email }))
        .send()
        .await
        .expect("Failed to execute request");

    // Get the token from database
    let token = {
        let conn = app.db.connect().expect("Failed to connect to database");
        let mut rows = conn
            .query(
                "SELECT token FROM password_reset_tokens ORDER BY created_at DESC LIMIT 1",
                (),
            )
            .await
            .expect("Failed to query token");

        let token: String = rows
            .next()
            .await
            .expect("Failed to get row")
            .expect("No token found")
            .get(0)
            .expect("Failed to get token");

        // Drop rows and conn before continuing
        drop(rows);
        drop(conn);
        token
    };

    // Act - Try to reset with too short password
    let response = app
        .client
        .post(format!("{}/api/password-reset/{}", &app.address, token))
        .json(&json!({ "password": "short" }))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn change_password_with_correct_current_password_succeeds() {
    // Arrange
    let app = spawn_app().await;
    let email = "test@example.com";
    let old_password = "oldpassword123";
    let new_password = "newpassword456";

    let token = app.register_user("testuser", email, old_password).await;

    // Act - Change password
    let response = app
        .client
        .post(format!("{}/api/account/password", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "current_password": old_password,
            "new_password": new_password
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    // Verify old password no longer works
    let login_response = app
        .client
        .post(format!("{}/api/users/login", &app.address))
        .json(&json!({
            "user": {
                "email": email,
                "password": old_password
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(login_response.status().as_u16(), 401);

    // Verify new password works
    let login_response = app
        .client
        .post(format!("{}/api/users/login", &app.address))
        .json(&json!({
            "user": {
                "email": email,
                "password": new_password
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(login_response.status().as_u16(), 200);
}

#[tokio::test]
async fn change_password_with_incorrect_current_password_fails() {
    // Arrange
    let app = spawn_app().await;
    let email = "test@example.com";
    let password = "password123";

    let token = app.register_user("testuser", email, password).await;

    // Act - Try to change password with wrong current password
    let response = app
        .client
        .post(format!("{}/api/account/password", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "current_password": "wrongpassword",
            "new_password": "newpassword456"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn change_password_requires_authentication() {
    // Arrange
    let app = spawn_app().await;

    // Act - Try to change password without authentication
    let response = app
        .client
        .post(format!("{}/api/account/password", &app.address))
        .json(&json!({
            "current_password": "password123",
            "new_password": "newpassword456"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn password_reset_invalidates_existing_tokens() {
    // Arrange
    let app = spawn_app().await;
    let email = "test@example.com";
    let old_password = "password123";
    let new_password = "newpassword456";

    // Register user and get their token
    let old_token = app.register_user("testuser", email, old_password).await;

    // Verify the old token works
    let response = app
        .client
        .get(format!("{}/api/user", &app.address))
        .header("Authorization", format!("Bearer {}", old_token))
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(response.status().as_u16(), 200);

    // Request password reset
    app.client
        .post(format!("{}/api/password-reset/request", &app.address))
        .json(&json!({ "email": email }))
        .send()
        .await
        .expect("Failed to execute request");

    // Get the reset token from database
    let reset_token = {
        let conn = app.db.connect().expect("Failed to connect to database");
        let mut rows = conn
            .query(
                "SELECT token FROM password_reset_tokens ORDER BY created_at DESC LIMIT 1",
                (),
            )
            .await
            .expect("Failed to query token");

        let token: String = rows
            .next()
            .await
            .expect("Failed to get row")
            .expect("No token found")
            .get(0)
            .expect("Failed to get token");

        drop(rows);
        drop(conn);
        token
    };

    // Complete password reset
    let response = app
        .client
        .post(format!("{}/api/password-reset/{}", &app.address, reset_token))
        .json(&json!({ "password": new_password }))
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(response.status().as_u16(), 200);

    // Act - Try to use the old token
    let response = app
        .client
        .get(format!("{}/api/user", &app.address))
        .header("Authorization", format!("Bearer {}", old_token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Old token should be invalid (401)
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn change_password_invalidates_existing_tokens() {
    // Arrange
    let app = spawn_app().await;
    let email = "test@example.com";
    let old_password = "password123";
    let new_password = "newpassword456";

    // Register user and get their token
    let old_token = app.register_user("testuser", email, old_password).await;

    // Verify the old token works
    let response = app
        .client
        .get(format!("{}/api/user", &app.address))
        .header("Authorization", format!("Bearer {}", old_token))
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(response.status().as_u16(), 200);

    // Change password using the old token
    let response = app
        .client
        .post(format!("{}/api/account/password", &app.address))
        .header("Authorization", format!("Bearer {}", old_token))
        .json(&json!({
            "current_password": old_password,
            "new_password": new_password
        }))
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(response.status().as_u16(), 200);

    // Act - Try to use the old token again
    let response = app
        .client
        .get(format!("{}/api/user", &app.address))
        .header("Authorization", format!("Bearer {}", old_token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Old token should be invalid (401)
    assert_eq!(response.status().as_u16(), 401);
}
