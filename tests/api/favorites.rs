// tests/api/favorites.rs

use crate::helpers::{spawn_app, TestArticleBuilder, TestFixture, TestUserBuilder};
use crate::{assert_status, bearer_request, parse_json};
use reqwest::StatusCode;
use serde_json::{json, Value};

#[tokio::test]
async fn test_favorite_article_happy_path() {
    // Arrange - Use TestFixture for multi-user scenario
    let fixture = TestFixture::new()
        .await
        .with_user_default("articleauthor")
        .await
        .with_user_default("favoriteuser")
        .await;

    let author_token = fixture.get_token("articleauthor");
    let user_token = fixture.get_token("favoriteuser");

    // Create an article
    let slug = fixture
        .app
        .create_article(
            &author_token,
            "Article to Favorite",
            "This article will be favorited",
            "Content of the article to be favorited",
            vec!["favorite", "test"],
        )
        .await;

    // Act - Favorite the article
    let response = bearer_request!(post &fixture.app,
        format!("{}/api/articles/{}/favorite", &fixture.app.address, slug),
        &user_token,
        json!({})
    )
    .await
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    // Verify response structure
    assert!(response_body["article"].is_object());
    assert_eq!(response_body["article"]["favorited"], true);
    assert_eq!(response_body["article"]["favoritesCount"], 1);
    assert_eq!(response_body["article"]["slug"], slug);
    assert_eq!(response_body["article"]["title"], "Article to Favorite");
}

#[tokio::test]
async fn test_favorite_article_without_auth() {
    // Arrange
    let app = spawn_app().await;

    // Act - Try to favorite without authentication
    let response = app
        .client
        .post(format!("{}/api/articles/some-slug/favorite", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_favorite_nonexistent_article() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act - Try to favorite non-existent article
    let response = bearer_request!(post &app,
        format!("{}/api/articles/non-existent-slug/favorite", &app.address),
        &token,
        json!({})
    )
    .await
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_unfavorite_article_happy_path() {
    // Arrange - Use TestFixture for multi-user scenario
    let fixture = TestFixture::new()
        .await
        .with_user_default("author")
        .await
        .with_user_default("unfavoriteuser")
        .await;

    let author_token = fixture.get_token("author");
    let user_token = fixture.get_token("unfavoriteuser");

    // Create article
    let slug = fixture
        .app
        .create_article(
            &author_token,
            "Article to Unfavorite",
            "This article will be unfavorited",
            "Content of the article",
            vec!["unfavorite", "test"],
        )
        .await;

    // First favorite the article
    bearer_request!(post &fixture.app,
        format!("{}/api/articles/{}/favorite", &fixture.app.address, slug),
        &user_token,
        json!({})
    )
    .await
    .expect("Failed to favorite article");

    // Act - Unfavorite the article
    let response = bearer_request!(delete &fixture.app,
        format!("{}/api/articles/{}/favorite", &fixture.app.address, slug),
        &user_token
    )
    .await
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    // Verify response structure
    assert!(response_body["article"].is_object());
    assert_eq!(response_body["article"]["favorited"], false);
    assert_eq!(response_body["article"]["favoritesCount"], 0);
    assert_eq!(response_body["article"]["slug"], slug);
}

#[tokio::test]
async fn test_unfavorite_article_without_auth() {
    // Arrange
    let app = spawn_app().await;

    // Act - Try to unfavorite without authentication
    let response = app
        .client
        .delete(format!("{}/api/articles/some-slug/favorite", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_unfavorite_nonexistent_article() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act - Try to unfavorite non-existent article
    let response = bearer_request!(delete &app,
        format!("{}/api/articles/non-existent-slug/favorite", &app.address),
        &token
    )
    .await
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_multiple_users_favorite_same_article() {
    // Arrange - Use TestFixture for multi-user scenario
    let fixture = TestFixture::new()
        .await
        .with_user_default("author")
        .await
        .with_user_default("user1")
        .await
        .with_user_default("user2")
        .await;

    let author_token = fixture.get_token("author");
    let user1_token = fixture.get_token("user1");
    let user2_token = fixture.get_token("user2");

    // Create article
    let slug = fixture
        .app
        .create_article(
            &author_token,
            "Popular Article",
            "This article will have multiple favorites",
            "Popular content",
            vec!["popular"],
        )
        .await;

    // Act - Both users favorite the article
    let response1 = bearer_request!(post &fixture.app,
        format!("{}/api/articles/{}/favorite", &fixture.app.address, slug),
        &user1_token,
        json!({})
    )
    .await
    .expect("Failed to favorite by user1");

    let response2 = bearer_request!(post &fixture.app,
        format!("{}/api/articles/{}/favorite", &fixture.app.address, slug),
        &user2_token,
        json!({})
    )
    .await
    .expect("Failed to favorite by user2");

    // Assert
    assert_status!(response1, StatusCode::OK);
    assert_status!(response2, StatusCode::OK);

    let body1: Value = parse_json!(response1);
    let body2: Value = parse_json!(response2);

    // Both should show favorited=true for their respective requests
    assert_eq!(body1["article"]["favorited"], true);
    assert_eq!(body1["article"]["favoritesCount"], 1);

    assert_eq!(body2["article"]["favorited"], true);
    assert_eq!(body2["article"]["favoritesCount"], 2);

    // Verify by getting article without auth (should show total count)
    let get_response = fixture
        .app
        .client
        .get(format!("{}/api/articles/{}", &fixture.app.address, slug))
        .send()
        .await
        .expect("Failed to get article");

    let get_body: Value = parse_json!(get_response);

    assert_eq!(get_body["article"]["favorited"], false); // No auth, so false
    assert_eq!(get_body["article"]["favoritesCount"], 2);
}

#[tokio::test]
async fn test_favorite_already_favorited_article() {
    // Arrange - Use TestFixture for multi-user scenario
    let fixture = TestFixture::new()
        .await
        .with_user_default("author")
        .await
        .with_user_default("duplicateuser")
        .await;

    let author_token = fixture.get_token("author");
    let user_token = fixture.get_token("duplicateuser");

    // Create article
    let slug = fixture
        .app
        .create_article(
            &author_token,
            "Already Favorited Article",
            "This article will be favorited twice",
            "Content",
            vec!["duplicate"],
        )
        .await;

    // First favorite
    bearer_request!(post &fixture.app,
        format!("{}/api/articles/{}/favorite", &fixture.app.address, slug),
        &user_token,
        json!({})
    )
    .await
    .expect("Failed to favorite article first time");

    // Act - Try to favorite again
    let response = bearer_request!(post &fixture.app,
        format!("{}/api/articles/{}/favorite", &fixture.app.address, slug),
        &user_token,
        json!({})
    )
    .await
    .expect("Failed to execute request");

    // Assert - Should still work (idempotent) but count should remain 1
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    assert_eq!(response_body["article"]["favorited"], true);
    assert_eq!(response_body["article"]["favoritesCount"], 1); // Should not increase
}

#[tokio::test]
async fn test_unfavorite_not_favorited_article() {
    // Arrange - Use TestFixture for multi-user scenario
    let fixture = TestFixture::new()
        .await
        .with_user_default("author")
        .await
        .with_user_default("notfavuser")
        .await;

    let author_token = fixture.get_token("author");
    let user_token = fixture.get_token("notfavuser");

    // Create article
    let slug = fixture
        .app
        .create_article(
            &author_token,
            "Not Favorited Article",
            "This article is not favorited yet",
            "Content",
            vec!["notfav"],
        )
        .await;

    // Act - Try to unfavorite without favoriting first
    let response = bearer_request!(delete &fixture.app,
        format!("{}/api/articles/{}/favorite", &fixture.app.address, slug),
        &user_token
    )
    .await
    .expect("Failed to execute request");

    // Assert - Should still work (idempotent)
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    assert_eq!(response_body["article"]["favorited"], false);
    assert_eq!(response_body["article"]["favoritesCount"], 0);
}

#[tokio::test]
async fn test_get_articles_favorited_by_user() {
    // Arrange - Use TestFixture for multi-user scenario
    let fixture = TestFixture::new()
        .await
        .with_user_default("author")
        .await
        .with_user_default("favoriter")
        .await;

    let author_token = fixture.get_token("author");
    let user_token = fixture.get_token("favoriter");

    // Create two articles
    let slug1 = fixture
        .app
        .create_article(
            &author_token,
            "First Article",
            "First article description",
            "First article content",
            vec!["first"],
        )
        .await;

    let _slug2 = fixture
        .app
        .create_article(
            &author_token,
            "Second Article",
            "Second article description",
            "Second article content",
            vec!["second"],
        )
        .await;

    // Favorite only the first article
    bearer_request!(post &fixture.app,
        format!("{}/api/articles/{}/favorite", &fixture.app.address, slug1),
        &user_token,
        json!({})
    )
    .await
    .expect("Failed to favorite first article");

    // Act - Get articles favorited by the user
    let response = fixture
        .app
        .client
        .get(format!("{}/api/articles?favorited=favoriter", &fixture.app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    assert!(response_body["articles"].is_array());
    assert_eq!(response_body["articlesCount"], 1);

    let articles = response_body["articles"].as_array().unwrap();
    assert_eq!(articles.len(), 1);
    assert_eq!(articles[0]["slug"], slug1);
    assert_eq!(articles[0]["title"], "First Article");
}

#[tokio::test]
async fn test_get_articles_favorited_by_nonexistent_user() {
    // Arrange
    let app = spawn_app().await;

    // Act - Try to get articles favorited by non-existent user
    let response = app
        .client
        .get(format!(
            "{}/api/articles?favorited=nonexistentuser",
            &app.address
        ))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should return empty list, not error
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    assert!(response_body["articles"].is_array());
    assert_eq!(response_body["articlesCount"], 0);
    assert_eq!(response_body["articles"].as_array().unwrap().len(), 0);
}
