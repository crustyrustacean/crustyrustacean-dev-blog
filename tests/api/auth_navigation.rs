// tests/api/auth_navigation.rs
// Tests for authentication during browser navigation (using cookies)

use crate::helpers::{TestUserBuilder, spawn_app};

#[tokio::test]
async fn navigation_to_admin_users_with_cookie_auth() {
    let app = spawn_app().await;

    // Register user using helper (auto-verifies and returns token)
    let token = app
        .register_user("testuser", "test@example.com", "password123")
        .await;

    // Simulate browser navigation - set cookie and navigate without Authorization header
    let cookie = format!("authToken={}", token);

    let response = app
        .client
        .get(format!("{}/admin/users", &app.address))
        .header("Cookie", cookie)
        .send()
        .await
        .expect("Failed to execute request");

    // Should return 200 OK with HTML page, not 401
    assert_eq!(
        response.status(),
        200,
        "Expected 200 OK when navigating with cookie auth, got {}",
        response.status()
    );

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    assert!(
        content_type.contains("text/html"),
        "Expected HTML response, got content-type: {}",
        content_type
    );
}

#[tokio::test]
async fn navigation_to_account_settings_with_cookie_auth() {
    let app = spawn_app().await;

    // Register user using helper
    let token = app
        .register_user("testuser", "test@example.com", "password123")
        .await;

    // Navigate to account settings with cookie
    let cookie = format!("authToken={}", token);

    let response = app
        .client
        .get(format!("{}/account", &app.address))
        .header("Cookie", cookie)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(
        response.status(),
        200,
        "Expected 200 OK when navigating to account with cookie auth"
    );
}

#[tokio::test]
async fn navigation_to_admin_dashboard_with_cookie_auth() {
    let app = spawn_app().await;

    // Register user using helper
    let token = app
        .register_user("testuser", "test@example.com", "password123")
        .await;

    // Navigate to admin dashboard with cookie
    let cookie = format!("authToken={}", token);

    let response = app
        .client
        .get(format!("{}/admin", &app.address))
        .header("Cookie", cookie)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(
        response.status(),
        200,
        "Expected 200 OK when navigating to admin dashboard with cookie auth"
    );
}

#[tokio::test]
async fn navigation_without_auth_cookie_returns_401() {
    let app = spawn_app().await;

    // Try to access admin users without cookie
    let response = app
        .client
        .get(format!("{}/admin/users", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(
        response.status(),
        401,
        "Expected 401 Unauthorized when navigating without auth"
    );
}
