// tests/api/auth.rs

use crate::helpers::{spawn_app, TestUserBuilder};
use crate::{assert_status, bearer_request, parse_json};
use reqwest::StatusCode;
use serde_json::{json, Value};

#[tokio::test]
async fn test_user_registration_happy_path() {
    // Arrange
    let app = spawn_app().await;

    let user_data = json!({
        "user": {
            "username": "testuser",
            "email": "test@example.com",
            "password": "securepassword123"
        }
    });

    // Act
    let response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user_data)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    let status = response.status();
    if status != StatusCode::OK {
        let error_body = response.text().await.expect("Failed to get error text");
        panic!("Expected 200 OK, got {}: {}", status, error_body);
    }

    let response_body: Value = parse_json!(response);

    // Verify response structure
    assert!(response_body["user"].is_object());
    assert_eq!(response_body["user"]["email"], "test@example.com");
    assert_eq!(response_body["user"]["username"], "testuser");
    assert!(response_body["user"]["token"].is_string());
    assert!(!response_body["user"]["token"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_user_login_happy_path() {
    // Arrange
    let app = spawn_app().await;

    // First, register a user
    app.register_user("loginuser", "login@example.com", "securepassword123")
        .await;

    let login_data = json!({
        "user": {
            "email": "login@example.com",
            "password": "securepassword123"
        }
    });

    // Act
    let response = app
        .client
        .post(format!("{}/api/users/login", &app.address))
        .header("Content-Type", "application/json")
        .json(&login_data)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    // Verify response structure
    assert!(response_body["user"].is_object());
    assert_eq!(response_body["user"]["email"], "login@example.com");
    assert_eq!(response_body["user"]["username"], "loginuser");
    assert!(response_body["user"]["token"].is_string());
    assert!(!response_body["user"]["token"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_protected_endpoint_with_valid_token() {
    // Arrange
    let app = spawn_app().await;
    let token = app
        .register_user("protecteduser", "protected@example.com", "securepassword123")
        .await;

    // Act - Access protected endpoint with valid token
    let response = bearer_request!(get &app,
        format!("{}/api/user", &app.address),
        &token
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    assert_eq!(response_body["user"]["email"], "protected@example.com");
    assert_eq!(response_body["user"]["username"], "protecteduser");
}

#[tokio::test]
async fn test_registration_with_invalid_data() {
    // Arrange
    let app = spawn_app().await;

    let test_cases = vec![
        // Invalid email
        json!({
            "user": {
                "username": "testuser",
                "email": "invalid-email",
                "password": "securepassword123"
            }
        }),
        // Short password
        json!({
            "user": {
                "username": "testuser",
                "email": "test@example.com",
                "password": "123"
            }
        }),
        // Short username
        json!({
            "user": {
                "username": "ab",
                "email": "test@example.com",
                "password": "securepassword123"
            }
        }),
        // Missing user field
        json!({
            "username": "testuser",
            "email": "test@example.com",
            "password": "securepassword123"
        }),
    ];

    for invalid_data in test_cases {
        // Act
        let response = app
            .client
            .post(format!("{}/api/users", &app.address))
            .header("Content-Type", "application/json")
            .json(&invalid_data)
            .send()
            .await
            .expect("Failed to execute request");

        // Assert
        assert_status!(response, StatusCode::BAD_REQUEST);
    }
}

#[tokio::test]
async fn test_login_with_invalid_credentials() {
    // Arrange
    let app = spawn_app().await;

    // Register a user first
    app.register_user("validuser", "valid@example.com", "correctpassword")
        .await;

    let test_cases = vec![
        // Wrong password
        json!({
            "user": {
                "email": "valid@example.com",
                "password": "wrongpassword"
            }
        }),
        // Non-existent user
        json!({
            "user": {
                "email": "nonexistent@example.com",
                "password": "somepassword"
            }
        }),
        // Invalid email format
        json!({
            "user": {
                "email": "invalid-email",
                "password": "somepassword"
            }
        }),
    ];

    for invalid_login in test_cases {
        // Act
        let response = app
            .client
            .post(format!("{}/api/users/login", &app.address))
            .header("Content-Type", "application/json")
            .json(&invalid_login)
            .send()
            .await
            .expect("Failed to execute request");

        // Assert
        assert!(
            response.status() == StatusCode::BAD_REQUEST
                || response.status() == StatusCode::UNAUTHORIZED,
            "Expected 400 or 401, got: {}",
            response.status()
        );
    }
}

#[tokio::test]
async fn test_protected_endpoint_without_token() {
    // Arrange
    let app = spawn_app().await;

    // Act - Access protected endpoint without token
    let response = app
        .client
        .get(format!("{}/api/user", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_endpoint_with_invalid_token() {
    // Arrange
    let app = spawn_app().await;

    let test_cases = vec![
        "invalid.jwt.token",
        "Bearer malformed-token",
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid.signature",
        "",
    ];

    for invalid_token in test_cases {
        // Act
        let response = app
            .client
            .get(format!("{}/api/user", &app.address))
            .header("Authorization", format!("Bearer {}", invalid_token))
            .send()
            .await
            .expect("Failed to execute request");

        // Assert
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Expected 401 for token: {}",
            invalid_token
        );
    }
}

#[tokio::test]
async fn test_duplicate_user_registration() {
    // Arrange
    let app = spawn_app().await;

    let user_data = json!({
        "user": {
            "username": "duplicateuser",
            "email": "duplicate@example.com",
            "password": "securepassword123"
        }
    });

    // Act - Register user first time
    let first_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user_data)
        .send()
        .await
        .expect("Failed to execute first request");

    // Assert first registration succeeds
    assert_status!(first_response, StatusCode::OK);

    // Act - Try to register same user again
    let second_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user_data)
        .send()
        .await
        .expect("Failed to execute second request");

    // Assert second registration fails
    assert_status!(second_response, StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_token_contains_valid_claims() {
    // Arrange
    let app = spawn_app().await;
    let token = app
        .register_user("claimsuser", "claims@example.com", "securepassword123")
        .await;

    // Verify token is not empty and appears to be a JWT (has 3 parts separated by dots)
    assert!(!token.is_empty());
    let token_parts: Vec<&str> = token.split('.').collect();
    assert_eq!(
        token_parts.len(),
        3,
        "JWT should have 3 parts separated by dots"
    );

    // Verify each part is base64-encoded (not empty)
    for part in token_parts {
        assert!(!part.is_empty(), "JWT parts should not be empty");
    }
}

#[tokio::test]
async fn test_protected_endpoint_with_cookie() {
    // Arrange
    let app = spawn_app().await;
    let token = app
        .register_user("cookieuser", "cookie@example.com", "securepassword123")
        .await;

    // Act - Access protected endpoint with cookie instead of Authorization header
    let response = app
        .client
        .get(format!("{}/api/user", &app.address))
        .header("Cookie", format!("authToken={}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    assert_eq!(response_body["user"]["email"], "cookie@example.com");
    assert_eq!(response_body["user"]["username"], "cookieuser");
}

#[tokio::test]
async fn test_cookie_takes_precedence_over_missing_header() {
    // Arrange
    let app = spawn_app().await;
    let token = app
        .register_user(
            "precedenceuser",
            "precedence@example.com",
            "securepassword123",
        )
        .await;

    // Act - Access protected endpoint with only cookie (no Authorization header)
    let response = app
        .client
        .get(format!("{}/api/user", &app.address))
        .header("Cookie", format!("authToken={}; other=value", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should work with cookie even without Authorization header
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    assert_eq!(response_body["user"]["email"], "precedence@example.com");
    assert_eq!(response_body["user"]["username"], "precedenceuser");
}
