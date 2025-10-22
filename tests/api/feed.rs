// tests/api/feed.rs

use crate::helpers::{TestArticleBuilder, TestFixture, TestUserBuilder, spawn_app};
use crate::{assert_status, bearer_request, parse_json};
use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn test_get_feed_happy_path() {
    // Arrange - Use TestFixture for multi-user scenario
    let fixture = TestFixture::new()
        .await
        .with_user_default("follower")
        .await
        .with_user_default("author")
        .await;

    let follower_token = fixture.get_token("follower");
    let author_token = fixture.get_token("author");

    // Follower follows author
    fixture
        .app
        .client
        .post(format!(
            "{}/api/profiles/author/follow",
            &fixture.app.address
        ))
        .header("Authorization", format!("Bearer {}", follower_token))
        .send()
        .await
        .expect("Failed to follow author");

    // Author creates an article
    fixture
        .app
        .create_article(
            &author_token,
            "Feed Article",
            "This should appear in feed",
            "Content for the feed",
            vec!["feed", "test"],
        )
        .await;

    // Act - Follower gets feed
    let response = bearer_request!(
        get & fixture.app,
        format!("{}/api/articles/feed", &fixture.app.address),
        &follower_token
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    assert!(response_body["articles"].is_array());
    assert_eq!(response_body["articlesCount"], 1);

    let articles = response_body["articles"].as_array().unwrap();
    assert_eq!(articles[0]["title"], "Feed Article");
    assert_eq!(articles[0]["author"]["username"], "author");
}

#[tokio::test]
async fn test_get_feed_requires_authentication() {
    // Arrange
    let app = spawn_app().await;

    // Act - Try to get feed without authentication
    let response = app
        .client
        .get(format!("{}/api/articles/feed", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_feed_empty_when_not_following_anyone() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("loner").await;

    // Act - Get feed when not following anyone
    let response = bearer_request!(
        get & app,
        format!("{}/api/articles/feed", &app.address),
        &token
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    assert!(response_body["articles"].is_array());
    assert_eq!(response_body["articlesCount"], 0);
    assert_eq!(response_body["articles"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_get_feed_with_pagination() {
    // Arrange - Use TestFixture for multi-user scenario
    let fixture = TestFixture::new()
        .await
        .with_user_default("feedreader")
        .await
        .with_user_default("prolificwriter")
        .await;

    let follower_token = fixture.get_token("feedreader");
    let author_token = fixture.get_token("prolificwriter");

    // Follower follows author
    fixture
        .app
        .client
        .post(format!(
            "{}/api/profiles/prolificwriter/follow",
            &fixture.app.address
        ))
        .header("Authorization", format!("Bearer {}", follower_token))
        .send()
        .await
        .expect("Failed to follow author");

    // Author creates multiple articles
    for i in 1..=3 {
        fixture
            .app
            .create_article(
                &author_token,
                &format!("Feed Article {}", i),
                &format!("Description {}", i),
                &format!("Content {}", i),
                vec!["feed"],
            )
            .await;
    }

    // Act - Get feed with limit
    let response = bearer_request!(
        get & fixture.app,
        format!("{}/api/articles/feed?limit=2", &fixture.app.address),
        &follower_token
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    assert!(response_body["articles"].is_array());
    assert_eq!(response_body["articlesCount"], 2); // Limited to 2

    let articles = response_body["articles"].as_array().unwrap();
    assert_eq!(articles.len(), 2);

    // Should be in reverse chronological order (newest first)
    assert_eq!(articles[0]["title"], "Feed Article 3");
    assert_eq!(articles[1]["title"], "Feed Article 2");
}

#[tokio::test]
async fn test_get_feed_excludes_unfollowed_authors() {
    // Arrange - Use TestFixture for multi-user scenario
    let fixture = TestFixture::new()
        .await
        .with_user_default("selectivereader")
        .await
        .with_user_default("followedauthor")
        .await
        .with_user_default("unfollowedauthor")
        .await;

    let follower_token = fixture.get_token("selectivereader");
    let followed_token = fixture.get_token("followedauthor");
    let unfollowed_token = fixture.get_token("unfollowedauthor");

    // Follower follows only one author
    fixture
        .app
        .client
        .post(format!(
            "{}/api/profiles/followedauthor/follow",
            &fixture.app.address
        ))
        .header("Authorization", format!("Bearer {}", follower_token))
        .send()
        .await
        .expect("Failed to follow author");

    // Both authors create articles
    fixture
        .app
        .create_article(
            &followed_token,
            "Should Appear in Feed",
            "From followed author",
            "This should appear",
            vec![],
        )
        .await;

    fixture
        .app
        .create_article(
            &unfollowed_token,
            "Should NOT Appear in Feed",
            "From unfollowed author",
            "This should not appear",
            vec![],
        )
        .await;

    // Act - Get feed
    let response = bearer_request!(
        get & fixture.app,
        format!("{}/api/articles/feed", &fixture.app.address),
        &follower_token
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    assert!(response_body["articles"].is_array());
    assert_eq!(response_body["articlesCount"], 1); // Only one article from followed user

    let articles = response_body["articles"].as_array().unwrap();
    assert_eq!(articles[0]["title"], "Should Appear in Feed");
    assert_eq!(articles[0]["author"]["username"], "followedauthor");
}
