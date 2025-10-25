// tests/api/media.rs

use crate::helpers::{TestUserBuilder, spawn_app};
use crate::{assert_status, bearer_request, parse_json};
use reqwest::{StatusCode, multipart::{Form, Part}};
use serde_json::{Value, json};

// ============================================================================
// Upload Tests
// ============================================================================

#[tokio::test]
async fn upload_media_succeeds_with_valid_image() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user("testuser", "test@example.com", "password123").await;

    // Create a simple 1x1 PNG image (smallest possible PNG)
    let png_bytes = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let part = Part::bytes(png_bytes)
        .file_name("test.png")
        .mime_str("image/png")
        .expect("Failed to create mime type");

    let form = Form::new().part("file", part);

    // Act
    let response = app
        .client
        .post(format!("{}/api/media", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body: Value = parse_json!(response);

    // Verify response structure
    assert!(body["media"].is_object());
    assert!(body["media"]["id"].is_string());
    assert_eq!(body["media"]["filename"], "test.png");
    assert_eq!(body["media"]["mime_type"], "image/png");
    assert!(body["media"]["file_size"].is_number());
    assert!(body["media"]["download_url"].is_string());
}

#[tokio::test]
async fn upload_media_with_metadata() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user("metauser", "meta@example.com", "password123").await;

    let png_bytes = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let part = Part::bytes(png_bytes)
        .file_name("test_meta.png")
        .mime_str("image/png")
        .expect("Failed to create mime type");

    let form = Form::new()
        .part("file", part)
        .text("title", "Test Image")
        .text("alt_text", "A test image for testing")
        .text("caption", "Test caption")
        .text("description", "This is a test image upload with metadata");

    // Act
    let response = app
        .client
        .post(format!("{}/api/media", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body: Value = parse_json!(response);

    assert_eq!(body["media"]["title"], "Test Image");
    assert_eq!(body["media"]["alt_text"], "A test image for testing");
    assert_eq!(body["media"]["caption"], "Test caption");
    assert_eq!(body["media"]["description"], "This is a test image upload with metadata");
}

#[tokio::test]
async fn upload_media_fails_with_non_image_file() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user("textuser", "text@example.com", "password123").await;

    let part = Part::text("not an image")
        .file_name("test.txt")
        .mime_str("text/plain")
        .expect("Failed to create mime type");

    let form = Form::new().part("file", part);

    // Act
    let response = app
        .client
        .post(format!("{}/api/media", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - should reject non-image files
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn upload_media_fails_without_authentication() {
    // Arrange
    let app = spawn_app().await;

    let png_bytes = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let part = Part::bytes(png_bytes)
        .file_name("no_auth.png")
        .mime_str("image/png")
        .expect("Failed to create mime type");

    let form = Form::new().part("file", part);

    // Act
    let response = app
        .client
        .post(format!("{}/api/media", &app.address))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - should require authentication
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn upload_media_fails_with_no_file() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user("nofile", "nofile@example.com", "password123").await;

    let form = Form::new();

    // Act
    let response = app
        .client
        .post(format!("{}/api/media", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// List Tests
// ============================================================================

#[tokio::test]
async fn list_media_returns_users_media() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user("listuser", "list@example.com", "password123").await;

    // Upload a test image first
    let png_bytes = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let part = Part::bytes(png_bytes)
        .file_name("list_test.png")
        .mime_str("image/png")
        .expect("Failed to create mime type");

    let form = Form::new().part("file", part);

    let _ = app
        .client
        .post(format!("{}/api/media", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .expect("Failed to upload media");

    // Act
    let response = bearer_request!(get & app, format!("{}/api/media", &app.address), &token)
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body: Value = parse_json!(response);

    assert!(body["media"].is_array());
    assert!(body["mediaCount"].is_number());
    let media_count = body["mediaCount"].as_i64().unwrap();
    assert!(media_count >= 1, "Should have at least one media item");
}

#[tokio::test]
async fn list_media_fails_without_authentication() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .client
        .get(format!("{}/api/media", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_media_with_pagination() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user("pageuser", "page@example.com", "password123").await;

    // Act
    let response = app
        .client
        .get(format!("{}/api/media?limit=10&offset=0", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body: Value = parse_json!(response);

    assert!(body["media"].is_array());
    let media = body["media"].as_array().unwrap();
    assert!(media.len() <= 10, "Should respect limit parameter");
}

// ============================================================================
// Get Metadata Tests
// ============================================================================

#[tokio::test]
async fn get_media_metadata_succeeds_for_owner() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user("metaowner", "metaowner@example.com", "password123").await;

    // Upload media
    let png_bytes = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let part = Part::bytes(png_bytes)
        .file_name("metadata_test.png")
        .mime_str("image/png")
        .expect("Failed to create mime type");

    let form = Form::new().part("file", part);

    let upload_response = app
        .client
        .post(format!("{}/api/media", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .expect("Failed to upload media");

    let upload_body: Value = parse_json!(upload_response);
    let media_id = upload_body["media"]["id"].as_str().unwrap();

    // Act
    let response = bearer_request!(
        get & app,
        format!("{}/api/media/{}", &app.address, media_id),
        &token
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body: Value = parse_json!(response);

    assert_eq!(body["media"]["id"], media_id);
    assert_eq!(body["media"]["filename"], "metadata_test.png");
    assert!(body["media"]["file_size"].is_number());
}

#[tokio::test]
async fn get_media_metadata_fails_for_non_owner() {
    // Arrange
    let app = spawn_app().await;
    let owner_token = app.register_user("owner", "owner@example.com", "password123").await;
    let other_token = app.register_user("other", "other@example.com", "password123").await;

    // Owner uploads media
    let png_bytes = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let part = Part::bytes(png_bytes)
        .file_name("ownership_test.png")
        .mime_str("image/png")
        .expect("Failed to create mime type");

    let form = Form::new().part("file", part);

    let upload_response = app
        .client
        .post(format!("{}/api/media", &app.address))
        .header("Authorization", format!("Bearer {}", owner_token))
        .multipart(form)
        .send()
        .await
        .expect("Failed to upload media");

    let upload_body: Value = parse_json!(upload_response);
    let media_id = upload_body["media"]["id"].as_str().unwrap();

    // Act - other user tries to access
    let response = bearer_request!(
        get & app,
        format!("{}/api/media/{}", &app.address, media_id),
        &other_token
    )
    .expect("Failed to execute request");

    // Assert - should be forbidden
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn get_media_metadata_fails_for_nonexistent_media() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user("notfound", "notfound@example.com", "password123").await;
    let fake_id = "nonexistent-media-id";

    // Act
    let response = bearer_request!(
        get & app,
        format!("{}/api/media/{}", &app.address, fake_id),
        &token
    )
    .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Update Metadata Tests
// ============================================================================

#[tokio::test]
async fn update_media_metadata_succeeds() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user("updater", "updater@example.com", "password123").await;

    // Upload media
    let png_bytes = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let part = Part::bytes(png_bytes)
        .file_name("update_test.png")
        .mime_str("image/png")
        .expect("Failed to create mime type");

    let form = Form::new().part("file", part);

    let upload_response = app
        .client
        .post(format!("{}/api/media", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .expect("Failed to upload media");

    let upload_body: Value = parse_json!(upload_response);
    let media_id = upload_body["media"]["id"].as_str().unwrap();

    // Act - update metadata
    let update_data = json!({
        "title": "Updated Title",
        "alt_text": "Updated alt text",
        "caption": "Updated caption",
        "description": "Updated description"
    });

    let response = bearer_request!(
        put & app,
        format!("{}/api/media/{}", &app.address, media_id),
        &token,
        update_data
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body: Value = parse_json!(response);

    assert_eq!(body["media"]["title"], "Updated Title");
    assert_eq!(body["media"]["alt_text"], "Updated alt text");
    assert_eq!(body["media"]["caption"], "Updated caption");
    assert_eq!(body["media"]["description"], "Updated description");
}

#[tokio::test]
async fn update_media_metadata_fails_for_non_owner() {
    // Arrange
    let app = spawn_app().await;
    let owner_token = app.register_user("metaowner2", "metaowner2@example.com", "password123").await;
    let other_token = app.register_user("metaother", "metaother@example.com", "password123").await;

    // Owner uploads media
    let png_bytes = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let part = Part::bytes(png_bytes)
        .file_name("update_ownership.png")
        .mime_str("image/png")
        .expect("Failed to create mime type");

    let form = Form::new().part("file", part);

    let upload_response = app
        .client
        .post(format!("{}/api/media", &app.address))
        .header("Authorization", format!("Bearer {}", owner_token))
        .multipart(form)
        .send()
        .await
        .expect("Failed to upload media");

    let upload_body: Value = parse_json!(upload_response);
    let media_id = upload_body["media"]["id"].as_str().unwrap();

    // Act - other user tries to update
    let update_data = json!({
        "title": "Hacked Title"
    });

    let response = bearer_request!(
        put & app,
        format!("{}/api/media/{}", &app.address, media_id),
        &other_token,
        update_data
    )
    .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

// ============================================================================
// Delete Tests
// ============================================================================

#[tokio::test]
async fn delete_media_succeeds_for_owner() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user("deleter", "deleter@example.com", "password123").await;

    // Upload media
    let png_bytes = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let part = Part::bytes(png_bytes)
        .file_name("delete_test.png")
        .mime_str("image/png")
        .expect("Failed to create mime type");

    let form = Form::new().part("file", part);

    let upload_response = app
        .client
        .post(format!("{}/api/media", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .expect("Failed to upload media");

    let upload_body: Value = parse_json!(upload_response);
    let media_id = upload_body["media"]["id"].as_str().unwrap();

    // Act
    let response = bearer_request!(
        delete & app,
        format!("{}/api/media/{}", &app.address, media_id),
        &token
    )
    .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify media is gone
    let get_response = bearer_request!(
        get & app,
        format!("{}/api/media/{}", &app.address, media_id),
        &token
    )
    .expect("Failed to execute get request");

    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_media_fails_for_non_owner() {
    // Arrange
    let app = spawn_app().await;
    let owner_token = app.register_user("delowner", "delowner@example.com", "password123").await;
    let other_token = app.register_user("delother", "delother@example.com", "password123").await;

    // Owner uploads media
    let png_bytes = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let part = Part::bytes(png_bytes)
        .file_name("delete_ownership.png")
        .mime_str("image/png")
        .expect("Failed to create mime type");

    let form = Form::new().part("file", part);

    let upload_response = app
        .client
        .post(format!("{}/api/media", &app.address))
        .header("Authorization", format!("Bearer {}", owner_token))
        .multipart(form)
        .send()
        .await
        .expect("Failed to upload media");

    let upload_body: Value = parse_json!(upload_response);
    let media_id = upload_body["media"]["id"].as_str().unwrap();

    // Act - other user tries to delete
    let response = bearer_request!(
        delete & app,
        format!("{}/api/media/{}", &app.address, media_id),
        &other_token
    )
    .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn delete_media_fails_for_nonexistent_media() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user("delfake", "delfake@example.com", "password123").await;
    let fake_id = "nonexistent-media-id";

    // Act
    let response = bearer_request!(
        delete & app,
        format!("{}/api/media/{}", &app.address, fake_id),
        &token
    )
    .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Download Tests
// ============================================================================

#[tokio::test]
async fn download_media_succeeds() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user("downloader", "downloader@example.com", "password123").await;

    let png_bytes = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let part = Part::bytes(png_bytes.clone())
        .file_name("download_test.png")
        .mime_str("image/png")
        .expect("Failed to create mime type");

    let form = Form::new().part("file", part);

    let upload_response = app
        .client
        .post(format!("{}/api/media", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .expect("Failed to upload media");

    let upload_body: Value = parse_json!(upload_response);
    let media_id = upload_body["media"]["id"].as_str().unwrap();

    // Act
    let response = app
        .client
        .get(format!("{}/api/media/{}/download", &app.address, media_id))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "image/png"
    );

    let downloaded_bytes = response.bytes().await.expect("Failed to get bytes");
    assert_eq!(downloaded_bytes.to_vec(), png_bytes);
}

#[tokio::test]
async fn download_media_fails_for_nonexistent_media() {
    // Arrange
    let app = spawn_app().await;
    let fake_id = "nonexistent-media-id";

    // Act
    let response = app
        .client
        .get(format!("{}/api/media/{}/download", &app.address, fake_id))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Complete Workflow Tests
// ============================================================================

#[tokio::test]
async fn complete_media_lifecycle_workflow() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user("lifecycle", "lifecycle@example.com", "password123").await;

    let png_bytes = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    // Step 1: Upload
    let part = Part::bytes(png_bytes.clone())
        .file_name("lifecycle.png")
        .mime_str("image/png")
        .expect("Failed to create mime type");

    let form = Form::new()
        .part("file", part)
        .text("title", "Lifecycle Test");

    let upload_response = app
        .client
        .post(format!("{}/api/media", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .expect("Failed to upload");

    assert_status!(upload_response, StatusCode::OK);
    let upload_body: Value = parse_json!(upload_response);
    let media_id = upload_body["media"]["id"].as_str().unwrap();

    // Step 2: List and verify it's there
    let list_response = bearer_request!(get & app, format!("{}/api/media", &app.address), &token)
        .expect("Failed to list");

    assert_status!(list_response, StatusCode::OK);
    let list_body: Value = parse_json!(list_response);
    assert!(list_body["media"].as_array().unwrap().len() >= 1);

    // Step 3: Get metadata
    let meta_response = bearer_request!(
        get & app,
        format!("{}/api/media/{}", &app.address, media_id),
        &token
    )
    .expect("Failed to get metadata");

    assert_status!(meta_response, StatusCode::OK);

    // Step 4: Update metadata
    let update_data = json!({
        "title": "Updated Lifecycle Test",
        "description": "A complete lifecycle test"
    });

    let update_response = bearer_request!(
        put & app,
        format!("{}/api/media/{}", &app.address, media_id),
        &token,
        update_data
    )
    .expect("Failed to update");

    assert_status!(update_response, StatusCode::OK);
    let update_body: Value = parse_json!(update_response);
    assert_eq!(update_body["media"]["title"], "Updated Lifecycle Test");

    // Step 5: Download
    let download_response = app
        .client
        .get(format!("{}/api/media/{}/download", &app.address, media_id))
        .send()
        .await
        .expect("Failed to download");

    assert_status!(download_response, StatusCode::OK);
    let downloaded_bytes = download_response.bytes().await.expect("Failed to get bytes");
    assert_eq!(downloaded_bytes.to_vec(), png_bytes);

    // Step 6: Delete
    let delete_response = bearer_request!(
        delete & app,
        format!("{}/api/media/{}", &app.address, media_id),
        &token
    )
    .expect("Failed to delete");

    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    // Step 7: Verify it's gone
    let verify_response = bearer_request!(
        get & app,
        format!("{}/api/media/{}", &app.address, media_id),
        &token
    )
    .expect("Failed to verify deletion");

    assert_eq!(verify_response.status(), StatusCode::NOT_FOUND);
}