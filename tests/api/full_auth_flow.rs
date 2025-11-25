// tests/api/full_auth_flow.rs
// Test the complete authentication flow as a browser would experience it

use crate::helpers::{TestUserBuilder, spawn_app};

#[tokio::test]
async fn complete_browser_flow_admin_users_page() {
    let app = spawn_app().await;

    // Step 1: Register and get token using helper
    let token = app
        .register_user("testuser", "test@example.com", "password123")
        .await;
    let cookie = format!("authToken={}", token);

    // Step 2: Navigate to /admin/users with cookie (simulates browser navigation)
    let page_response = app
        .client
        .get(format!("{}/admin/users", &app.address))
        .header("Cookie", &cookie)
        .send()
        .await
        .expect("Failed to load page");

    assert_eq!(
        page_response.status(),
        200,
        "Page should load with cookie auth"
    );

    // Step 3: Simulate JavaScript API call that happens when page loads (using cookie)
    let api_response = app
        .client
        .get(format!("{}/api/admin/users", &app.address))
        .header("Cookie", &cookie)
        .send()
        .await
        .expect("Failed to call API with cookie");

    assert_eq!(
        api_response.status(),
        200,
        "API call should work with cookie auth, got status: {}",
        api_response.status()
    );

    // Step 4: Verify the API response is valid JSON
    let api_body: serde_json::Value = api_response
        .json()
        .await
        .expect("Failed to parse API response");

    assert!(
        api_body["data"]["users"].is_array(),
        "API should return users array"
    );
}

#[tokio::test]
async fn javascript_api_calls_work_with_bearer_token_from_cookie() {
    let app = spawn_app().await;

    // Register and get token using helper
    let token = app
        .register_user("testuser", "test@example.com", "password123")
        .await;

    // Simulate browser: Page loads with cookie, JavaScript extracts token from cookie
    // and sends it as Bearer token in API calls
    let api_response = app
        .client
        .get(format!("{}/api/admin/users", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to call API");

    assert_eq!(
        api_response.status(),
        200,
        "API call with Bearer token should work, got: {}",
        api_response.status()
    );
}

#[tokio::test]
async fn verify_cookie_is_readable_by_checking_both_auth_methods() {
    let app = spawn_app().await;

    // Register using helper
    let token = app
        .register_user("testuser", "test@example.com", "password123")
        .await;
    let cookie = format!("authToken={}", token);

    // Test 1: API call with Cookie header only (simulates server-side auth check)
    let cookie_response = app
        .client
        .get(format!("{}/api/admin/users", &app.address))
        .header("Cookie", &cookie)
        .send()
        .await
        .expect("Failed with cookie");

    println!("Cookie auth status: {}", cookie_response.status());
    assert_eq!(
        cookie_response.status(),
        200,
        "Cookie authentication should work for API calls"
    );

    // Test 2: API call with Authorization header (simulates JavaScript reading cookie and sending as Bearer)
    let bearer_response = app
        .client
        .get(format!("{}/api/admin/users", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed with bearer");

    println!("Bearer auth status: {}", bearer_response.status());
    assert_eq!(
        bearer_response.status(),
        200,
        "Bearer authentication should work for API calls"
    );
}
