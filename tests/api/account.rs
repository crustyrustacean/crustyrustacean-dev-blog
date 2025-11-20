// tests/api/account.rs

use crate::helpers::{TestUserBuilder, spawn_app};
use serde_json::json;

#[tokio::test]
async fn account_page_requires_authentication() {
    // Arrange
    let app = spawn_app().await;

    // Act - Try to access account page without authentication
    let response = app
        .client
        .get(format!("{}/account", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn account_page_loads_for_authenticated_user() {
    // Arrange
    let app = spawn_app().await;
    let email = "test@example.com";
    let username = "testuser";

    let token = app.register_user(username, email, "password123").await;

    // Act - Access account page with authentication
    let response = app
        .client
        .get(format!("{}/account", &app.address))
        .header("Cookie", format!("authToken={}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body = response.text().await.expect("Failed to get response body");
    assert!(body.contains("Account Settings"));
    assert!(body.contains(username));
    assert!(body.contains(email));
}

#[tokio::test]
async fn account_page_shows_profile_information() {
    // Arrange
    let app = spawn_app().await;
    let email = "user@example.com";
    let username = "profileuser";
    let bio = "This is my bio";

    let token = app.register_user(username, email, "password123").await;

    // Update profile with bio
    app.client
        .put(format!("{}/api/user", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "user": {
                "bio": bio
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Act - Access account page
    let response = app
        .client
        .get(format!("{}/account", &app.address))
        .header("Cookie", format!("authToken={}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body = response.text().await.expect("Failed to get response body");
    assert!(body.contains(bio));
}

#[tokio::test]
async fn update_profile_via_api_succeeds() {
    // Arrange
    let app = spawn_app().await;
    let email = "original@example.com";
    let username = "originaluser";

    let token = app.register_user(username, email, "password123").await;

    let new_username = "updateduser";
    let new_email = "updated@example.com";
    let new_bio = "Updated bio text";

    // Act - Update profile
    let response = app
        .client
        .put(format!("{}/api/user", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "user": {
                "username": new_username,
                "email": new_email,
                "bio": new_bio
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to deserialize response");

    assert_eq!(body["user"]["username"], new_username);
    assert_eq!(body["user"]["email"], new_email);
    assert_eq!(body["user"]["bio"], new_bio);
}

#[tokio::test]
async fn update_profile_with_invalid_email_fails() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act - Try to update with invalid email
    let response = app
        .client
        .put(format!("{}/api/user", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "user": {
                "email": "not-an-email"
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn update_profile_with_short_username_fails() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act - Try to update with username that's too short
    let response = app
        .client
        .put(format!("{}/api/user", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "user": {
                "username": "ab"
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn account_page_includes_change_password_form() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act - Access account page
    let response = app
        .client
        .get(format!("{}/account", &app.address))
        .header("Cookie", format!("authToken={}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body = response.text().await.expect("Failed to get response body");
    assert!(body.contains("Change Password"));
    assert!(body.contains("currentPassword"));
    assert!(body.contains("newPassword"));
    assert!(body.contains("confirmPassword"));
}

#[tokio::test]
async fn account_page_includes_password_reset_link() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act - Access account page
    let response = app
        .client
        .get(format!("{}/account", &app.address))
        .header("Cookie", format!("authToken={}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body = response.text().await.expect("Failed to get response body");
    assert!(body.contains("Danger Zone"));
    assert!(body.contains("/password-reset/request"));
}

#[tokio::test]
async fn update_profile_image_url_succeeds() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;
    let image_url = "https://example.com/avatar.jpg";

    // Act - Update profile image
    let response = app
        .client
        .put(format!("{}/api/user", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "user": {
                "image": image_url
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to deserialize response");

    assert_eq!(body["user"]["image"], image_url);
}
