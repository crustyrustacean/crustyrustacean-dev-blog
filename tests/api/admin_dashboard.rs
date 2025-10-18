// tests/api/admin_dashboard.rs

use crate::helpers::spawn_app;
use reqwest::StatusCode;
use serde_json::{Value, json};

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
    let user_data = json!({
        "user": {
            "username": "adminuser",
            "email": "admin@example.com",
            "password": "securepassword123"
        }
    });
    
    let registration_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user_data)
        .send()
        .await
        .expect("Failed to register user");
    
    let registration_body: Value = registration_response
        .json()
        .await
        .expect("Failed to parse registration response");
    
    let token = registration_body["user"]["token"].as_str().unwrap();
    
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
    
    // Check that it returns HTML content
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    // Should be HTML, not JSON
    assert!(content_type.contains("text/html") || content_type.is_empty());
    
    // Check that the response body contains expected HTML
    let body = response.text().await.expect("Failed to get response body");
    assert!(body.contains("<html") || body.contains("<!DOCTYPE html"));
    assert!(body.contains("Admin Dashboard"));
}

#[tokio::test]
async fn test_admin_dashboard_with_articles() {
    // Arrange
    let app = spawn_app().await;
    
    // Register user and get token
    let user_data = json!({
        "user": {
            "username": "dashboarduser",
            "email": "dashboard@example.com",
            "password": "securepassword123"
        }
    });
    
    let registration_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user_data)
        .send()
        .await
        .expect("Failed to register user");
    
    let registration_body: Value = registration_response
        .json()
        .await
        .expect("Failed to parse registration response");
    
    let token = registration_body["user"]["token"].as_str().unwrap();
    
    // Create an article to show in dashboard
    let article_data = json!({
        "article": {
            "title": "Dashboard Test Article",
            "description": "This article should appear in the dashboard",
            "body": "This is content for the dashboard test article.",
            "tagList": ["test", "dashboard"]
        }
    });
    
    let create_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .json(&article_data)
        .send()
        .await
        .expect("Failed to create article");
    
    assert_eq!(create_response.status(), StatusCode::OK);
    
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
    
    let body = response.text().await.expect("Failed to get response body");
    
    // Should contain the dashboard structure
    assert!(body.contains("Admin Dashboard"));
    assert!(body.contains("Dashboard Test Article")); // Article title should appear
    assert!(body.contains("dashboarduser")); // Author name should appear
    
    // Should contain expected dashboard elements
    assert!(body.contains("New Article")); // New article button
    assert!(body.contains("Actions")); // Actions column header
}

#[tokio::test]
async fn test_admin_dashboard_template_renders_with_special_characters() {
    // Arrange
    let app = spawn_app().await;
    
    // Register user with special characters in username
    let user_data = json!({
        "user": {
            "username": "test-user_123",
            "email": "special@example.com",
            "password": "securepassword123"
        }
    });
    
    let registration_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user_data)
        .send()
        .await
        .expect("Failed to register user");
    
    let registration_body: Value = registration_response
        .json()
        .await
        .expect("Failed to parse registration response");
    
    let token = registration_body["user"]["token"].as_str().unwrap();
    
    // Create an article with special characters in title (potential template issue)
    let article_data = json!({
        "article": {
            "title": "Test Article with 'Quotes' and \"Double Quotes\"",
            "description": "Description with <HTML> & special chars",
            "body": "Body with special characters: & < > ' \"",
            "tagList": ["special-chars", "test"]
        }
    });
    
    let create_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .json(&article_data)
        .send()
        .await
        .expect("Failed to create article");
    
    assert_eq!(create_response.status(), StatusCode::OK);
    
    // Act - Access admin dashboard (this should not fail due to special characters)
    let response = app
        .client
        .get(format!("{}/admin", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");
    
    // Assert - Should handle special characters properly and return successful response
    let status = response.status();
    if status != StatusCode::OK {
        let error_body = response.text().await.unwrap_or_default();
        panic!("Expected 200 OK, got {}: {}", status, error_body);
    }
    
    let body = response.text().await.expect("Failed to get response body");
    
    // Should contain escaped/safe versions of special characters
    assert!(body.contains("Admin Dashboard"));
    // The title should be properly escaped in HTML
    assert!(body.contains("Test Article with")); // Part of the title should be there
}