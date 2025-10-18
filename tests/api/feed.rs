// tests/api/feed.rs

use crate::helpers::spawn_app;
use reqwest::StatusCode;
use serde_json::{Value, json};

#[tokio::test]
async fn test_get_feed_happy_path() {
    // Arrange
    let app = spawn_app().await;

    // Register two users
    let user1_data = json!({
        "user": {
            "username": "follower",
            "email": "follower@example.com",
            "password": "securepassword123"
        }
    });

    let user2_data = json!({
        "user": {
            "username": "author",
            "email": "author@example.com",
            "password": "securepassword123"
        }
    });

    let reg1_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user1_data)
        .send()
        .await
        .expect("Failed to register follower");

    let reg2_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user2_data)
        .send()
        .await
        .expect("Failed to register author");

    let reg1_body: Value = reg1_response
        .json()
        .await
        .expect("Failed to parse reg1 response");
    let reg2_body: Value = reg2_response
        .json()
        .await
        .expect("Failed to parse reg2 response");

    let follower_token = reg1_body["user"]["token"].as_str().unwrap();
    let author_token = reg2_body["user"]["token"].as_str().unwrap();

    // Follower follows author
    app.client
        .post(format!("{}/api/profiles/author/follow", &app.address))
        .header("Authorization", format!("Bearer {}", follower_token))
        .send()
        .await
        .expect("Failed to follow author");

    // Author creates an article
    let article_data = json!({
        "article": {
            "title": "Feed Article",
            "description": "This should appear in feed",
            "body": "Content for the feed",
            "tagList": ["feed", "test"]
        }
    });

    app.client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", author_token))
        .json(&article_data)
        .send()
        .await
        .expect("Failed to create article");

    // Act - Follower gets feed
    let response = app
        .client
        .get(format!("{}/api/articles/feed", &app.address))
        .header("Authorization", format!("Bearer {}", follower_token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response body");

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
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_feed_empty_when_not_following_anyone() {
    // Arrange
    let app = spawn_app().await;

    // Register user
    let user_data = json!({
        "user": {
            "username": "loner",
            "email": "loner@example.com",
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

    // Act - Get feed when not following anyone
    let response = app
        .client
        .get(format!("{}/api/articles/feed", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response body");

    assert!(response_body["articles"].is_array());
    assert_eq!(response_body["articlesCount"], 0);
    assert_eq!(response_body["articles"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_get_feed_with_pagination() {
    // Arrange
    let app = spawn_app().await;

    // Register two users
    let follower_data = json!({
        "user": {
            "username": "feedreader",
            "email": "feedreader@example.com",
            "password": "securepassword123"
        }
    });

    let author_data = json!({
        "user": {
            "username": "prolificwriter",
            "email": "prolific@example.com",
            "password": "securepassword123"
        }
    });

    let follower_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&follower_data)
        .send()
        .await
        .expect("Failed to register follower");

    let author_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&author_data)
        .send()
        .await
        .expect("Failed to register author");

    let follower_body: Value = follower_response
        .json()
        .await
        .expect("Failed to parse follower response");
    let author_body: Value = author_response
        .json()
        .await
        .expect("Failed to parse author response");

    let follower_token = follower_body["user"]["token"].as_str().unwrap();
    let author_token = author_body["user"]["token"].as_str().unwrap();

    // Follower follows author
    app.client
        .post(format!(
            "{}/api/profiles/prolificwriter/follow",
            &app.address
        ))
        .header("Authorization", format!("Bearer {}", follower_token))
        .send()
        .await
        .expect("Failed to follow author");

    // Author creates multiple articles
    for i in 1..=3 {
        let article_data = json!({
            "article": {
                "title": format!("Feed Article {}", i),
                "description": format!("Description {}", i),
                "body": format!("Content {}", i),
                "tagList": ["feed"]
            }
        });

        app.client
            .post(format!("{}/api/articles", &app.address))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", author_token))
            .json(&article_data)
            .send()
            .await
            .expect("Failed to create article");
    }

    // Act - Get feed with limit
    let response = app
        .client
        .get(format!("{}/api/articles/feed?limit=2", &app.address))
        .header("Authorization", format!("Bearer {}", follower_token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response body");

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
    // Arrange
    let app = spawn_app().await;

    // Register three users
    let follower_data = json!({
        "user": {
            "username": "selectivereader",
            "email": "selective@example.com",
            "password": "securepassword123"
        }
    });

    let followed_author_data = json!({
        "user": {
            "username": "followedauthor",
            "email": "followed@example.com",
            "password": "securepassword123"
        }
    });

    let unfollowed_author_data = json!({
        "user": {
            "username": "unfollowedauthor",
            "email": "unfollowed@example.com",
            "password": "securepassword123"
        }
    });

    let follower_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&follower_data)
        .send()
        .await
        .expect("Failed to register follower");

    let followed_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&followed_author_data)
        .send()
        .await
        .expect("Failed to register followed author");

    let unfollowed_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&unfollowed_author_data)
        .send()
        .await
        .expect("Failed to register unfollowed author");

    let follower_body: Value = follower_response
        .json()
        .await
        .expect("Failed to parse follower response");
    let followed_body: Value = followed_response
        .json()
        .await
        .expect("Failed to parse followed response");
    let unfollowed_body: Value = unfollowed_response
        .json()
        .await
        .expect("Failed to parse unfollowed response");

    let follower_token = follower_body["user"]["token"].as_str().unwrap();
    let followed_token = followed_body["user"]["token"].as_str().unwrap();
    let unfollowed_token = unfollowed_body["user"]["token"].as_str().unwrap();

    // Follower follows only one author
    app.client
        .post(format!(
            "{}/api/profiles/followedauthor/follow",
            &app.address
        ))
        .header("Authorization", format!("Bearer {}", follower_token))
        .send()
        .await
        .expect("Failed to follow author");

    // Both authors create articles
    let followed_article = json!({
        "article": {
            "title": "Should Appear in Feed",
            "description": "From followed author",
            "body": "This should appear"
        }
    });

    let unfollowed_article = json!({
        "article": {
            "title": "Should NOT Appear in Feed",
            "description": "From unfollowed author",
            "body": "This should not appear"
        }
    });

    app.client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", followed_token))
        .json(&followed_article)
        .send()
        .await
        .expect("Failed to create followed article");

    app.client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", unfollowed_token))
        .json(&unfollowed_article)
        .send()
        .await
        .expect("Failed to create unfollowed article");

    // Act - Get feed
    let response = app
        .client
        .get(format!("{}/api/articles/feed", &app.address))
        .header("Authorization", format!("Bearer {}", follower_token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response body");

    assert!(response_body["articles"].is_array());
    assert_eq!(response_body["articlesCount"], 1); // Only one article from followed user

    let articles = response_body["articles"].as_array().unwrap();
    assert_eq!(articles[0]["title"], "Should Appear in Feed");
    assert_eq!(articles[0]["author"]["username"], "followedauthor");
}
