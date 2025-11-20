// tests/api/cookie_persistence.rs
// Tests to verify cookie persistence across navigations

use crate::helpers::spawn_app;

#[tokio::test]
async fn cookie_persists_across_multiple_page_navigations() {
    let app = spawn_app().await;

    // Step 1: Register and get token
    let register_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .body(r#"{"user": {"username": "testuser", "email": "test@example.com", "password": "password123"}}"#)
        .send()
        .await
        .expect("Failed to register");

    let body: serde_json::Value = register_response
        .json()
        .await
        .expect("Failed to parse JSON");
    let token = body["user"]["token"].as_str().expect("No token");

    // Step 2: Navigate to admin dashboard with cookie
    let cookie = format!("authToken={}", token);

    let admin_response = app
        .client
        .get(format!("{}/admin", &app.address))
        .header("Cookie", &cookie)
        .send()
        .await
        .expect("Failed to navigate to admin");

    assert_eq!(
        admin_response.status(),
        200,
        "Failed to access admin dashboard with cookie"
    );

    // Step 3: Navigate to user management with the SAME cookie
    let users_response = app
        .client
        .get(format!("{}/admin/users", &app.address))
        .header("Cookie", &cookie)
        .send()
        .await
        .expect("Failed to navigate to users");

    assert_eq!(
        users_response.status(),
        200,
        "Failed to access user management with same cookie"
    );

    // Step 4: Navigate to account settings with the SAME cookie
    let account_response = app
        .client
        .get(format!("{}/account", &app.address))
        .header("Cookie", &cookie)
        .send()
        .await
        .expect("Failed to navigate to account");

    assert_eq!(
        account_response.status(),
        200,
        "Failed to access account settings with same cookie"
    );

    // Step 5: Make an API call with the SAME cookie
    let api_response = app
        .client
        .get(format!("{}/api/admin/users", &app.address))
        .header("Cookie", &cookie)
        .send()
        .await
        .expect("Failed to call API");

    assert_eq!(
        api_response.status(),
        200,
        "Failed to call API with same cookie"
    );
}

#[tokio::test]
async fn verify_cookie_format_matches_javascript_expectations() {
    let app = spawn_app().await;

    // Register
    let response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .body(r#"{"user": {"username": "test", "email": "test@example.com", "password": "password123"}}"#)
        .send()
        .await
        .expect("Failed to register");

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    let token = body["user"]["token"].as_str().expect("No token");

    // Test with plain cookie (no encoding issues)
    let cookie_plain = format!("authToken={}", token);

    let response1 = app
        .client
        .get(format!("{}/admin/users", &app.address))
        .header("Cookie", &cookie_plain)
        .send()
        .await
        .expect("Failed with plain cookie");

    assert_eq!(response1.status(), 200, "Plain cookie should work");
}
