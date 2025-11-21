// tests/api/tags.rs

use crate::helpers::{TestArticleBuilder, TestFixture, TestUserBuilder, spawn_app};
use crate::{assert_status, bearer_request, parse_json};
use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn get_tags_returns_empty_list_when_no_tags_exist() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .client
        .get(format!("{}/api/tags", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body = parse_json!(response);
    assert_eq!(body["tags"], json!([]));
}

#[tokio::test]
async fn get_tags_returns_all_unique_tags() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Create article with tags: rust, webdev
    app.create_article(
        &token,
        "How to learn Rust",
        "Ever wonder how?",
        "It takes time",
        vec!["rust", "webdev"],
    )
    .await;

    // Create another article with tags: rust, programming
    app.create_article(
        &token,
        "Rust patterns",
        "Common patterns",
        "Here are some patterns",
        vec!["rust", "programming"],
    )
    .await;

    // Act
    let response = app
        .client
        .get(format!("{}/api/tags", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body = parse_json!(response);
    let tags = body["tags"].as_array().expect("tags should be an array");

    // Should have 3 unique tags
    assert_eq!(tags.len(), 3);

    let tag_strings: Vec<String> = tags
        .iter()
        .map(|t| t.as_str().unwrap().to_string())
        .collect();

    assert!(tag_strings.contains(&"rust".to_string()));
    assert!(tag_strings.contains(&"webdev".to_string()));
    assert!(tag_strings.contains(&"programming".to_string()));
}

#[tokio::test]
async fn get_tags_returns_tags_in_alphabetical_order() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Create article with tags in non-alphabetical order
    app.create_article(
        &token,
        "Test article",
        "Testing tags",
        "Body content",
        vec!["zebra", "apple", "mango", "banana"],
    )
    .await;

    // Act
    let response = app
        .client
        .get(format!("{}/api/tags", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body = parse_json!(response);
    let tags = body["tags"].as_array().expect("tags should be an array");
    let tag_strings: Vec<String> = tags
        .iter()
        .map(|t| t.as_str().unwrap().to_string())
        .collect();

    // Should be in alphabetical order
    assert_eq!(tag_strings, vec!["apple", "banana", "mango", "zebra"]);
}

#[tokio::test]
async fn update_tag_happy_path() {
    // Arrange
    let fixture = TestFixture::new()
        .await
        .with_user_default("tagupdater")
        .await
        .promote_to_author("tagupdater")
        .await;

    let token = fixture.get_token("tagupdater");

    // Create article with a tag
    fixture.app.create_article(
        &token,
        "Test article",
        "Testing tag update",
        "Body content",
        vec!["oldtag"],
    )
    .await;

    // Act - Update the tag
    let update_body = json!({
        "tag": {
            "name": "newtag"
        }
    });

    let response = bearer_request!(
        put & fixture.app,
        format!("{}/api/tags/oldtag", &fixture.app.address),
        &token,
        update_body
    )
    .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body = parse_json!(response);
    assert_eq!(body["tag"], "newtag");

    // Verify the old tag no longer exists and new tag exists
    let tags_response = fixture.app
        .client
        .get(format!("{}/api/tags", &fixture.app.address))
        .send()
        .await
        .expect("Failed to get tags");

    let tags_body = parse_json!(tags_response);
    let tag_strings: Vec<String> = tags_body["tags"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.as_str().unwrap().to_string())
        .collect();

    assert!(!tag_strings.contains(&"oldtag".to_string()));
    assert!(tag_strings.contains(&"newtag".to_string()));
}

#[tokio::test]
async fn update_tag_requires_authentication() {
    // Arrange
    let app = spawn_app().await;

    let update_body = json!({
        "tag": {
            "name": "newtag"
        }
    });

    // Act - Try to update without authentication
    let response = app
        .client
        .put(format!("{}/api/tags/oldtag", &app.address))
        .json(&update_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn update_nonexistent_tag() {
    // Arrange
    let fixture = TestFixture::new()
        .await
        .with_user_default("testuser")
        .await
        .promote_to_author("testuser")
        .await;

    let token = fixture.get_token("testuser");

    // Act - Try to update a tag that doesn't exist
    let update_body = json!({
        "tag": {
            "name": "newtag"
        }
    });

    let response = bearer_request!(
        put & fixture.app,
        format!("{}/api/tags/nonexistent", &fixture.app.address),
        &token,
        update_body
    )
    .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_tag_with_duplicate_name() {
    // Arrange
    let fixture = TestFixture::new()
        .await
        .with_user_default("testuser")
        .await
        .promote_to_author("testuser")
        .await;

    let token = fixture.get_token("testuser");

    // Create article with two tags
    fixture.app.create_article(
        &token,
        "Test article",
        "Testing",
        "Body content",
        vec!["tag1", "tag2"],
    )
    .await;

    // Act - Try to rename tag1 to tag2 (which already exists)
    let update_body = json!({
        "tag": {
            "name": "tag2"
        }
    });

    let response = bearer_request!(
        put & fixture.app,
        format!("{}/api/tags/tag1", &fixture.app.address),
        &token,
        update_body
    )
    .expect("Failed to execute request.");

    // Assert - Should return conflict
    assert_status!(response, StatusCode::CONFLICT);
}

#[tokio::test]
async fn update_tag_with_empty_name() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Create article with a tag
    app.create_article(
        &token,
        "Test article",
        "Testing",
        "Body content",
        vec!["testtag"],
    )
    .await;

    // Act - Try to update with empty name
    let update_body = json!({
        "tag": {
            "name": ""
        }
    });

    let response = bearer_request!(
        put & app,
        format!("{}/api/tags/testtag", &app.address),
        &token,
        update_body
    )
    .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn update_tag_with_invalid_payload() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act - Try to update with missing "tag" wrapper
    let update_body = json!({
        "name": "newtag"
    });

    let response = bearer_request!(
        put & app,
        format!("{}/api/tags/sometag", &app.address),
        &token,
        update_body
    )
    .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn delete_tag_happy_path() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("tagdeleter").await;

    // Create article with tags
    app.create_article(
        &token,
        "Test article",
        "Testing tag deletion",
        "Body content",
        vec!["tagtokeep", "tagtodelete"],
    )
    .await;

    // Act - Delete one tag
    let response = bearer_request!(
        delete & app,
        format!("{}/api/tags/tagtodelete", &app.address),
        &token
    )
    .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::NO_CONTENT);

    // Verify the tag no longer exists
    let tags_response = app
        .client
        .get(format!("{}/api/tags", &app.address))
        .send()
        .await
        .expect("Failed to get tags");

    let tags_body = parse_json!(tags_response);
    let tag_strings: Vec<String> = tags_body["tags"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.as_str().unwrap().to_string())
        .collect();

    assert!(!tag_strings.contains(&"tagtodelete".to_string()));
    assert!(tag_strings.contains(&"tagtokeep".to_string()));
}

#[tokio::test]
async fn delete_tag_requires_authentication() {
    // Arrange
    let app = spawn_app().await;

    // Act - Try to delete without authentication
    let response = app
        .client
        .delete(format!("{}/api/tags/sometag", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn delete_nonexistent_tag() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act - Try to delete a tag that doesn't exist
    let response = bearer_request!(
        delete & app,
        format!("{}/api/tags/nonexistent", &app.address),
        &token
    )
    .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_tag_removes_from_articles() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Create article with tags
    let slug = app
        .create_article(
            &token,
            "Article with tags",
            "Testing",
            "Content",
            vec!["keep", "remove"],
        )
        .await;

    // Delete the tag
    let delete_response = bearer_request!(
        delete & app,
        format!("{}/api/tags/remove", &app.address),
        &token
    )
    .expect("Failed to execute request.");

    assert_status!(delete_response, StatusCode::NO_CONTENT);

    // Verify the article no longer has the deleted tag
    let article_response = app
        .client
        .get(format!("{}/api/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to get article");

    let article_json = parse_json!(article_response);
    let tag_list = article_json["article"]["tagList"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.as_str().unwrap().to_string())
        .collect::<Vec<String>>();

    assert!(!tag_list.contains(&"remove".to_string()));
    assert!(tag_list.contains(&"keep".to_string()));
}

#[tokio::test]
async fn update_tag_same_name_is_idempotent() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Create article with a tag
    app.create_article(
        &token,
        "Test article",
        "Testing",
        "Body content",
        vec!["sametag"],
    )
    .await;

    // Act - Update tag to the same name
    let update_body = json!({
        "tag": {
            "name": "sametag"
        }
    });

    let response = bearer_request!(
        put & app,
        format!("{}/api/tags/sametag", &app.address),
        &token,
        update_body
    )
    .expect("Failed to execute request.");

    // Assert - Should succeed (idempotent operation)
    assert_status!(response, StatusCode::OK);
    let body = parse_json!(response);
    assert_eq!(body["tag"], "sametag");
}
