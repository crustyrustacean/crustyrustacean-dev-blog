// tests/api/newsletters.rs

use crate::helpers::{spawn_app, TestUserBuilder};
use reqwest::StatusCode;
use serde_json::{json, Value};

#[tokio::test]
async fn test_subscribe_to_newsletter_success() {
    // Arrange
    let app = spawn_app().await;

    let subscribe_data = json!({
        "email": "subscriber@example.com",
        "name": "Test Subscriber"
    });

    // Act
    let response = app
        .client
        .post(format!("{}/api/newsletters/subscribe", &app.address))
        .header("Content-Type", "application/json")
        .json(&subscribe_data)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let response_body: Value = response.json().await.expect("Failed to parse response");

    assert_eq!(response_body["data"]["message"], "Please check your email to confirm your subscription.");
    assert_eq!(response_body["data"]["email"], "subscriber@example.com");
}

#[tokio::test]
async fn test_subscribe_with_invalid_email_fails() {
    // Arrange
    let app = spawn_app().await;

    let subscribe_data = json!({
        "email": "not-an-email",
        "name": "Test Subscriber"
    });

    // Act
    let response = app
        .client
        .post(format!("{}/api/newsletters/subscribe", &app.address))
        .header("Content-Type", "application/json")
        .json(&subscribe_data)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_subscribe_duplicate_email_returns_success() {
    // Arrange
    let app = spawn_app().await;

    let subscribe_data = json!({
        "email": "duplicate@example.com",
        "name": "Test Subscriber"
    });

    // First subscription
    let _ = app
        .client
        .post(format!("{}/api/newsletters/subscribe", &app.address))
        .header("Content-Type", "application/json")
        .json(&subscribe_data)
        .send()
        .await
        .expect("Failed to execute request");

    // Act - Second subscription with same email
    let response = app
        .client
        .post(format!("{}/api/newsletters/subscribe", &app.address))
        .header("Content-Type", "application/json")
        .json(&subscribe_data)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let response_body: Value = response.json().await.expect("Failed to parse response");

    // Should receive confirmation message
    assert!(response_body["data"]["message"].as_str().unwrap().contains("confirmation"));
}

#[tokio::test]
async fn test_confirm_subscription_success() {
    // Arrange
    let app = spawn_app().await;

    // First, subscribe to get a confirmation token
    let subscribe_data = json!({
        "email": "confirm@example.com",
        "name": "Test Subscriber"
    });

    let _ = app
        .client
        .post(format!("{}/api/newsletters/subscribe", &app.address))
        .header("Content-Type", "application/json")
        .json(&subscribe_data)
        .send()
        .await
        .expect("Failed to execute request");

    // Get the confirmation token from the database
    let token = {
        let conn = app.db.connect().expect("Failed to connect to database");
        let mut rows = conn
            .query(
                "SELECT confirmation_token FROM newsletter_subscribers WHERE email = 'confirm@example.com'",
                libsql::params![],
            )
            .await
            .expect("Failed to query database");

        let row = rows.next().await.expect("Failed to get row").expect("No subscriber found");
        row.get::<String>(0).expect("Failed to get token")
    }; // Connection is dropped here

    // Act - Confirm subscription
    let response = app
        .client
        .post(format!("{}/api/newsletters/confirm/{}", &app.address, token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let response_body: Value = response.json().await.expect("Failed to parse response");

    assert_eq!(response_body["data"]["message"], "Thank you! Your subscription has been confirmed.");
    assert_eq!(response_body["data"]["email"], "confirm@example.com");
}

#[tokio::test]
async fn test_confirm_subscription_with_invalid_token_fails() {
    // Arrange
    let app = spawn_app().await;
    let invalid_token = "invalid-token-12345";

    // Act
    let response = app
        .client
        .post(format!("{}/api/newsletters/confirm/{}", &app.address, invalid_token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_confirm_already_confirmed_subscription() {
    // Arrange
    let app = spawn_app().await;

    let subscribe_data = json!({
        "email": "already-confirmed@example.com",
        "name": "Test Subscriber"
    });

    let _ = app
        .client
        .post(format!("{}/api/newsletters/subscribe", &app.address))
        .header("Content-Type", "application/json")
        .json(&subscribe_data)
        .send()
        .await
        .expect("Failed to execute request");

    // Get confirmation token
    let token = {
        let conn = app.db.connect().expect("Failed to connect to database");
        let mut rows = conn
            .query(
                "SELECT confirmation_token FROM newsletter_subscribers WHERE email = 'already-confirmed@example.com'",
                libsql::params![],
            )
            .await
            .expect("Failed to query database");

        let row = rows.next().await.expect("Failed to get row").expect("No subscriber found");
        row.get::<String>(0).expect("Failed to get token")
    }; // Connection is dropped here

    // First confirmation
    let _ = app
        .client
        .post(format!("{}/api/newsletters/confirm/{}", &app.address, token))
        .send()
        .await
        .expect("Failed to execute request");

    // Act - Second confirmation
    let response = app
        .client
        .post(format!("{}/api/newsletters/confirm/{}", &app.address, token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let response_body: Value = response.json().await.expect("Failed to parse response");

    assert_eq!(response_body["data"]["message"], "Your subscription is already confirmed!");
}

#[tokio::test]
async fn test_unsubscribe_success() {
    // Arrange
    let app = spawn_app().await;

    let subscribe_data = json!({
        "email": "unsubscribe@example.com",
        "name": "Test Subscriber"
    });

    let _ = app
        .client
        .post(format!("{}/api/newsletters/subscribe", &app.address))
        .header("Content-Type", "application/json")
        .json(&subscribe_data)
        .send()
        .await
        .expect("Failed to execute request");

    // Get unsubscribe token
    let token = {
        let conn = app.db.connect().expect("Failed to connect to database");
        let mut rows = conn
            .query(
                "SELECT unsubscribe_token FROM newsletter_subscribers WHERE email = 'unsubscribe@example.com'",
                libsql::params![],
            )
            .await
            .expect("Failed to query database");

        let row = rows.next().await.expect("Failed to get row").expect("No subscriber found");
        row.get::<String>(0).expect("Failed to get token")
    }; // Connection is dropped here

    // Act - Unsubscribe
    let response = app
        .client
        .post(format!("{}/api/newsletters/unsubscribe/{}", &app.address, token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let response_body: Value = response.json().await.expect("Failed to parse response");

    assert_eq!(response_body["data"]["message"], "You have been successfully unsubscribed from our newsletter.");
}

#[tokio::test]
async fn test_unsubscribe_with_invalid_token_fails() {
    // Arrange
    let app = spawn_app().await;
    let invalid_token = "invalid-unsubscribe-token";

    // Act
    let response = app
        .client
        .post(format!("{}/api/newsletters/unsubscribe/{}", &app.address, invalid_token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_newsletter_requires_authentication() {
    // Arrange
    let app = spawn_app().await;

    let newsletter_data = json!({
        "title": "Test Newsletter",
        "subject": "Test Subject",
        "body": "Test body content"
    });

    // Act - Try to create without authentication
    let response = app
        .client
        .post(format!("{}/api/admin/newsletters", &app.address))
        .header("Content-Type", "application/json")
        .json(&newsletter_data)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_newsletter_success() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("author").await;

    let newsletter_data = json!({
        "title": "Test Newsletter",
        "subject": "Test Subject",
        "body": "This is the newsletter body content."
    });

    // Act
    let response = app
        .client
        .post(format!("{}/api/admin/newsletters", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .json(&newsletter_data)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::CREATED);
    let response_body: Value = response.json().await.expect("Failed to parse response");

    assert_eq!(response_body["data"]["message"], "Newsletter created successfully");
    assert!(response_body["data"]["id"].is_string());
}

#[tokio::test]
async fn test_list_newsletters_requires_authentication() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .client
        .get(format!("{}/api/admin/newsletters", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_list_newsletters_success() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("author").await;

    // Create a newsletter first
    let newsletter_data = json!({
        "title": "Test Newsletter",
        "subject": "Test Subject",
        "body": "Test body"
    });

    let _ = app
        .client
        .post(format!("{}/api/admin/newsletters", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .json(&newsletter_data)
        .send()
        .await
        .expect("Failed to execute request");

    // Act
    let response = app
        .client
        .get(format!("{}/api/admin/newsletters", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let response_body: Value = response.json().await.expect("Failed to parse response");

    assert!(response_body["data"]["newsletters"].is_array());
    let newsletters = response_body["data"]["newsletters"].as_array().unwrap();
    assert_eq!(newsletters.len(), 1);
    assert_eq!(newsletters[0]["title"], "Test Newsletter");
}

#[tokio::test]
async fn test_get_newsletter_stats() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("author").await;

    // Subscribe some users
    let subscribe_data = json!({
        "email": "stats1@example.com",
        "name": "Subscriber 1"
    });

    let _ = app
        .client
        .post(format!("{}/api/newsletters/subscribe", &app.address))
        .header("Content-Type", "application/json")
        .json(&subscribe_data)
        .send()
        .await
        .expect("Failed to execute request");

    // Act
    let response = app
        .client
        .get(format!("{}/api/admin/newsletters/stats", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let response_body: Value = response.json().await.expect("Failed to parse response");

    assert!(response_body["data"]["stats"].is_object());
    let stats = &response_body["data"]["stats"];
    assert_eq!(stats["total_subscribers"], 1);
    assert_eq!(stats["confirmed_subscribers"], 0); // Not confirmed yet
}

#[tokio::test]
async fn test_send_newsletter_success() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("author").await;

    // Create a subscriber and confirm
    let subscribe_data = json!({
        "email": "sendtest@example.com",
        "name": "Send Test"
    });

    let _ = app
        .client
        .post(format!("{}/api/newsletters/subscribe", &app.address))
        .header("Content-Type", "application/json")
        .json(&subscribe_data)
        .send()
        .await
        .expect("Failed to execute request");

    // Confirm the subscription
    let confirm_token = {
        let conn = app.db.connect().expect("Failed to connect to database");
        let mut rows = conn
            .query(
                "SELECT confirmation_token FROM newsletter_subscribers WHERE email = 'sendtest@example.com'",
                libsql::params![],
            )
            .await
            .expect("Failed to query database");

        let row = rows.next().await.expect("Failed to get row").expect("No subscriber found");
        row.get::<String>(0).expect("Failed to get token")
    }; // Connection is dropped here

    let _ = app
        .client
        .post(format!("{}/api/newsletters/confirm/{}", &app.address, confirm_token))
        .send()
        .await
        .expect("Failed to execute request");

    // Create a newsletter
    let newsletter_data = json!({
        "title": "Send Test Newsletter",
        "subject": "Send Test Subject",
        "body": "Send test body"
    });

    let create_response = app
        .client
        .post(format!("{}/api/admin/newsletters", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .json(&newsletter_data)
        .send()
        .await
        .expect("Failed to execute request");

    let create_body: Value = create_response.json().await.expect("Failed to parse response");
    let newsletter_id = create_body["data"]["id"].as_str().unwrap();

    // Act - Send newsletter
    let response = app
        .client
        .post(format!("{}/api/admin/newsletters/{}/send", &app.address, newsletter_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let response_body: Value = response.json().await.expect("Failed to parse response");

    assert!(response_body["data"]["message"].as_str().unwrap().contains("sent to 1 subscribers"));
    assert_eq!(response_body["data"]["recipient_count"], 1);
}

#[tokio::test]
async fn test_newsletter_page_accessible() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .client
        .get(format!("{}/newsletter", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("Failed to get response body");
    assert!(body.contains("Newsletter"));
}

#[tokio::test]
async fn test_admin_newsletters_page_requires_authentication() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .client
        .get(format!("{}/admin/newsletters", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
