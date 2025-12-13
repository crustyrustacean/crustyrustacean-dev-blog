// tests/api/themes.rs
//
// Integration tests for the theming system

use crate::helpers::{spawn_app, TestUserBuilder};
use serde_json::json;

// ============================================================================
// Theme API Tests
// ============================================================================

#[tokio::test]
async fn list_themes_returns_available_themes() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .client
        .get(format!("{}/api/themes", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to deserialize response");

    // Should return success envelope
    assert_eq!(body["success"], true);

    // Should contain themes data
    let themes = body["data"].as_array().expect("data should be an array");

    // Should have at least the default themes
    assert!(!themes.is_empty(), "Should have at least one theme");

    // Each theme should have required fields
    for theme in themes {
        assert!(theme["id"].is_string(), "theme should have id");
        assert!(theme["name"].is_string(), "theme should have name");
        assert!(theme["color_scheme"].is_string(), "theme should have color_scheme");
    }
}

#[tokio::test]
async fn list_themes_contains_default_dark_and_high_contrast() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .client
        .get(format!("{}/api/themes", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to deserialize response");

    let themes = body["data"].as_array().expect("data should be an array");
    let theme_ids: Vec<&str> = themes
        .iter()
        .filter_map(|t| t["id"].as_str())
        .collect();

    assert!(theme_ids.contains(&"default"), "Should contain default theme");
    assert!(theme_ids.contains(&"dark"), "Should contain dark theme");
    assert!(theme_ids.contains(&"high-contrast"), "Should contain high-contrast theme");
}

#[tokio::test]
async fn get_theme_preference_requires_authentication() {
    // Arrange
    let app = spawn_app().await;

    // Act - Try to get theme preference without authentication
    let response = app
        .client
        .get(format!("{}/api/user/theme", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn get_theme_preference_returns_auto_for_new_user() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act
    let response = app
        .client
        .get(format!("{}/api/user/theme", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to deserialize response");

    assert_eq!(body["success"], true);
    // New users have 'auto' as default theme preference (follows system)
    assert_eq!(
        body["data"]["theme"], "auto",
        "New user should have 'auto' theme preference"
    );
}

#[tokio::test]
async fn update_theme_preference_requires_authentication() {
    // Arrange
    let app = spawn_app().await;

    // Act - Try to update theme preference without authentication
    let response = app
        .client
        .put(format!("{}/api/user/theme", &app.address))
        .json(&json!({ "theme": "dark" }))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn update_theme_preference_succeeds_with_valid_theme() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act - Update theme preference to dark
    let response = app
        .client
        .put(format!("{}/api/user/theme", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({ "theme": "dark" }))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to deserialize response");

    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["theme"], "dark");
}

#[tokio::test]
async fn update_theme_preference_persists_across_requests() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act - Update theme preference
    app.client
        .put(format!("{}/api/user/theme", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({ "theme": "high-contrast" }))
        .send()
        .await
        .expect("Failed to execute request");

    // Get theme preference
    let response = app
        .client
        .get(format!("{}/api/user/theme", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to deserialize response");

    assert_eq!(body["data"]["theme"], "high-contrast");
}

#[tokio::test]
async fn update_theme_preference_with_auto_succeeds() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act - Set theme to auto (follow system)
    let response = app
        .client
        .put(format!("{}/api/user/theme", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({ "theme": "auto" }))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to deserialize response");

    assert_eq!(body["data"]["theme"], "auto");
}

#[tokio::test]
async fn update_theme_preference_with_invalid_theme_fails() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act - Try to set an invalid theme
    let response = app
        .client
        .put(format!("{}/api/user/theme", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({ "theme": "nonexistent-theme" }))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should reject invalid themes
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn update_theme_preference_with_empty_theme_fails() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act - Try to set empty theme
    let response = app
        .client
        .put(format!("{}/api/user/theme", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({ "theme": "" }))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

// ============================================================================
// Theme Template Rendering Tests
// ============================================================================

#[tokio::test]
async fn homepage_includes_theme_css_variables() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .client
        .get(format!("{}/", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body = response.text().await.expect("Failed to get response body");

    // Should include theme variables style block
    assert!(body.contains("id=\"theme-variables\""), "Should have theme-variables style block");

    // Should include CSS variables for themes
    assert!(body.contains("--theme-"), "Should include --theme- CSS variables");
}

#[tokio::test]
async fn homepage_includes_data_theme_attribute() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .client
        .get(format!("{}/", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body = response.text().await.expect("Failed to get response body");

    // Should have data-theme attribute on html element
    assert!(body.contains("data-theme="), "Should have data-theme attribute");
}

#[tokio::test]
async fn homepage_includes_theme_switcher_script() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .client
        .get(format!("{}/", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body = response.text().await.expect("Failed to get response body");

    // Should include theme-switcher.js
    assert!(body.contains("theme-switcher.js"), "Should include theme-switcher.js");
}

#[tokio::test]
async fn account_settings_includes_theme_picker() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act
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

    // Should include theme picker section
    assert!(body.contains("Theme Preferences"), "Should include theme preferences section");
    assert!(body.contains("themePicker"), "Should include theme picker element");

    // Should include theme options
    assert!(body.contains("data-theme=\"auto\""), "Should include auto theme option");
    assert!(body.contains("data-theme=\"default\""), "Should include light theme option");
    assert!(body.contains("data-theme=\"dark\""), "Should include dark theme option");
    assert!(body.contains("data-theme=\"high-contrast\""), "Should include high-contrast theme option");
}

#[tokio::test]
async fn all_themes_css_includes_multiple_theme_selectors() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .client
        .get(format!("{}/", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let body = response.text().await.expect("Failed to get response body");

    // Should include CSS selectors for different themes
    assert!(
        body.contains("[data-theme=\"default\"]") || body.contains("[data-theme=\\\"default\\\"]"),
        "Should include default theme CSS selector"
    );
    assert!(
        body.contains("[data-theme=\"dark\"]") || body.contains("[data-theme=\\\"dark\\\"]"),
        "Should include dark theme CSS selector"
    );
    assert!(
        body.contains("[data-theme=\"high-contrast\"]") || body.contains("[data-theme=\\\"high-contrast\\\"]"),
        "Should include high-contrast theme CSS selector"
    );
}
