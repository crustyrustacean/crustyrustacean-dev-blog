// tests/api/favorites.rs

use crate::helpers::spawn_app;
use reqwest::StatusCode;
use serde_json::{Value, json};

#[tokio::test]
async fn test_favorite_article_happy_path() {
    // Arrange
    let app = spawn_app().await;

    // Register first user (article author)
    let author_data = json!({
        "user": {
            "username": "articleauthor",
            "email": "author@example.com",
            "password": "securepassword123"
        }
    });

    let author_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&author_data)
        .send()
        .await
        .expect("Failed to register author");

    let author_body: Value = author_response
        .json()
        .await
        .expect("Failed to parse author response");

    let author_token = author_body["user"]["token"].as_str().unwrap();

    // Create an article
    let article_data = json!({
        "article": {
            "title": "Article to Favorite",
            "description": "This article will be favorited",
            "body": "Content of the article to be favorited",
            "tagList": ["favorite", "test"]
        }
    });

    let create_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", author_token))
        .json(&article_data)
        .send()
        .await
        .expect("Failed to create article");

    let create_body: Value = create_response
        .json()
        .await
        .expect("Failed to parse create response");

    let slug = create_body["article"]["slug"].as_str().unwrap();

    // Register second user (who will favorite the article)
    let user_data = json!({
        "user": {
            "username": "favoriteuser",
            "email": "favorite@example.com",
            "password": "securepassword123"
        }
    });

    let user_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user_data)
        .send()
        .await
        .expect("Failed to register user");

    let user_body: Value = user_response
        .json()
        .await
        .expect("Failed to parse user response");

    let user_token = user_body["user"]["token"].as_str().unwrap();

    // Act - Favorite the article
    let response = app
        .client
        .post(format!("{}/api/articles/{}/favorite", &app.address, slug))
        .header("Authorization", format!("Bearer {}", user_token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response body");

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
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_favorite_nonexistent_article() {
    // Arrange
    let app = spawn_app().await;

    // Register user
    let user_data = json!({
        "user": {
            "username": "testuser",
            "email": "test@example.com",
            "password": "securepassword123"
        }
    });

    let user_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user_data)
        .send()
        .await
        .expect("Failed to register user");

    let user_body: Value = user_response
        .json()
        .await
        .expect("Failed to parse user response");

    let token = user_body["user"]["token"].as_str().unwrap();

    // Act - Try to favorite non-existent article
    let response = app
        .client
        .post(format!(
            "{}/api/articles/non-existent-slug/favorite",
            &app.address
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_unfavorite_article_happy_path() {
    // Arrange
    let app = spawn_app().await;

    // Register and create article (similar to favorite test)
    let author_data = json!({
        "user": {
            "username": "author",
            "email": "author@example.com",
            "password": "securepassword123"
        }
    });

    let author_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&author_data)
        .send()
        .await
        .expect("Failed to register author");

    let author_body: Value = author_response
        .json()
        .await
        .expect("Failed to parse author response");

    let author_token = author_body["user"]["token"].as_str().unwrap();

    // Create article
    let article_data = json!({
        "article": {
            "title": "Article to Unfavorite",
            "description": "This article will be unfavorited",
            "body": "Content of the article",
            "tagList": ["unfavorite", "test"]
        }
    });

    let create_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", author_token))
        .json(&article_data)
        .send()
        .await
        .expect("Failed to create article");

    let create_body: Value = create_response
        .json()
        .await
        .expect("Failed to parse create response");

    let slug = create_body["article"]["slug"].as_str().unwrap();

    // Register user and favorite article first
    let user_data = json!({
        "user": {
            "username": "unfavoriteuser",
            "email": "unfavorite@example.com",
            "password": "securepassword123"
        }
    });

    let user_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user_data)
        .send()
        .await
        .expect("Failed to register user");

    let user_body: Value = user_response
        .json()
        .await
        .expect("Failed to parse user response");

    let user_token = user_body["user"]["token"].as_str().unwrap();

    // First favorite the article
    app.client
        .post(format!("{}/api/articles/{}/favorite", &app.address, slug))
        .header("Authorization", format!("Bearer {}", user_token))
        .send()
        .await
        .expect("Failed to favorite article");

    // Act - Unfavorite the article
    let response = app
        .client
        .delete(format!("{}/api/articles/{}/favorite", &app.address, slug))
        .header("Authorization", format!("Bearer {}", user_token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response body");

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
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_unfavorite_nonexistent_article() {
    // Arrange
    let app = spawn_app().await;

    // Register user
    let user_data = json!({
        "user": {
            "username": "testuser",
            "email": "test@example.com",
            "password": "securepassword123"
        }
    });

    let user_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user_data)
        .send()
        .await
        .expect("Failed to register user");

    let user_body: Value = user_response
        .json()
        .await
        .expect("Failed to parse user response");

    let token = user_body["user"]["token"].as_str().unwrap();

    // Act - Try to unfavorite non-existent article
    let response = app
        .client
        .delete(format!(
            "{}/api/articles/non-existent-slug/favorite",
            &app.address
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_multiple_users_favorite_same_article() {
    // Arrange
    let app = spawn_app().await;

    // Register author and create article
    let author_data = json!({
        "user": {
            "username": "author",
            "email": "author@example.com",
            "password": "securepassword123"
        }
    });

    let author_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&author_data)
        .send()
        .await
        .expect("Failed to register author");

    let author_body: Value = author_response
        .json()
        .await
        .expect("Failed to parse author response");

    let author_token = author_body["user"]["token"].as_str().unwrap();

    let article_data = json!({
        "article": {
            "title": "Popular Article",
            "description": "This article will have multiple favorites",
            "body": "Popular content",
            "tagList": ["popular"]
        }
    });

    let create_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", author_token))
        .json(&article_data)
        .send()
        .await
        .expect("Failed to create article");

    let create_body: Value = create_response
        .json()
        .await
        .expect("Failed to parse create response");

    let slug = create_body["article"]["slug"].as_str().unwrap();

    // Register two users
    let user1_data = json!({
        "user": {
            "username": "user1",
            "email": "user1@example.com",
            "password": "securepassword123"
        }
    });

    let user2_data = json!({
        "user": {
            "username": "user2",
            "email": "user2@example.com",
            "password": "securepassword123"
        }
    });

    let user1_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user1_data)
        .send()
        .await
        .expect("Failed to register user1");

    let user2_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user2_data)
        .send()
        .await
        .expect("Failed to register user2");

    let user1_body: Value = user1_response
        .json()
        .await
        .expect("Failed to parse user1 response");

    let user2_body: Value = user2_response
        .json()
        .await
        .expect("Failed to parse user2 response");

    let user1_token = user1_body["user"]["token"].as_str().unwrap();
    let user2_token = user2_body["user"]["token"].as_str().unwrap();

    // Act - Both users favorite the article
    let response1 = app
        .client
        .post(format!("{}/api/articles/{}/favorite", &app.address, slug))
        .header("Authorization", format!("Bearer {}", user1_token))
        .send()
        .await
        .expect("Failed to favorite by user1");

    let response2 = app
        .client
        .post(format!("{}/api/articles/{}/favorite", &app.address, slug))
        .header("Authorization", format!("Bearer {}", user2_token))
        .send()
        .await
        .expect("Failed to favorite by user2");

    // Assert
    assert_eq!(response1.status(), StatusCode::OK);
    assert_eq!(response2.status(), StatusCode::OK);

    let body1: Value = response1.json().await.expect("Failed to parse response1");

    let body2: Value = response2.json().await.expect("Failed to parse response2");

    // Both should show favorited=true for their respective requests
    assert_eq!(body1["article"]["favorited"], true);
    assert_eq!(body1["article"]["favoritesCount"], 1);

    assert_eq!(body2["article"]["favorited"], true);
    assert_eq!(body2["article"]["favoritesCount"], 2);

    // Verify by getting article without auth (should show total count)
    let get_response = app
        .client
        .get(format!("{}/api/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to get article");

    let get_body: Value = get_response
        .json()
        .await
        .expect("Failed to parse get response");

    assert_eq!(get_body["article"]["favorited"], false); // No auth, so false
    assert_eq!(get_body["article"]["favoritesCount"], 2);
}

#[tokio::test]
async fn test_favorite_already_favorited_article() {
    // Arrange
    let app = spawn_app().await;

    // Register author and create article
    let author_data = json!({
        "user": {
            "username": "author",
            "email": "author@example.com",
            "password": "securepassword123"
        }
    });

    let author_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&author_data)
        .send()
        .await
        .expect("Failed to register author");

    let author_body: Value = author_response
        .json()
        .await
        .expect("Failed to parse author response");

    let author_token = author_body["user"]["token"].as_str().unwrap();

    let article_data = json!({
        "article": {
            "title": "Already Favorited Article",
            "description": "This article will be favorited twice",
            "body": "Content",
            "tagList": ["duplicate"]
        }
    });

    let create_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", author_token))
        .json(&article_data)
        .send()
        .await
        .expect("Failed to create article");

    let create_body: Value = create_response
        .json()
        .await
        .expect("Failed to parse create response");

    let slug = create_body["article"]["slug"].as_str().unwrap();

    // Register user
    let user_data = json!({
        "user": {
            "username": "duplicateuser",
            "email": "duplicate@example.com",
            "password": "securepassword123"
        }
    });

    let user_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user_data)
        .send()
        .await
        .expect("Failed to register user");

    let user_body: Value = user_response
        .json()
        .await
        .expect("Failed to parse user response");

    let user_token = user_body["user"]["token"].as_str().unwrap();

    // First favorite
    app.client
        .post(format!("{}/api/articles/{}/favorite", &app.address, slug))
        .header("Authorization", format!("Bearer {}", user_token))
        .send()
        .await
        .expect("Failed to favorite article first time");

    // Act - Try to favorite again
    let response = app
        .client
        .post(format!("{}/api/articles/{}/favorite", &app.address, slug))
        .header("Authorization", format!("Bearer {}", user_token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should still work (idempotent) but count should remain 1
    assert_eq!(response.status(), StatusCode::OK);

    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response body");

    assert_eq!(response_body["article"]["favorited"], true);
    assert_eq!(response_body["article"]["favoritesCount"], 1); // Should not increase
}

#[tokio::test]
async fn test_unfavorite_not_favorited_article() {
    // Arrange
    let app = spawn_app().await;

    // Register author and create article
    let author_data = json!({
        "user": {
            "username": "author",
            "email": "author@example.com",
            "password": "securepassword123"
        }
    });

    let author_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&author_data)
        .send()
        .await
        .expect("Failed to register author");

    let author_body: Value = author_response
        .json()
        .await
        .expect("Failed to parse author response");

    let author_token = author_body["user"]["token"].as_str().unwrap();

    let article_data = json!({
        "article": {
            "title": "Not Favorited Article",
            "description": "This article is not favorited yet",
            "body": "Content",
            "tagList": ["notfav"]
        }
    });

    let create_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", author_token))
        .json(&article_data)
        .send()
        .await
        .expect("Failed to create article");

    let create_body: Value = create_response
        .json()
        .await
        .expect("Failed to parse create response");

    let slug = create_body["article"]["slug"].as_str().unwrap();

    // Register user
    let user_data = json!({
        "user": {
            "username": "notfavuser",
            "email": "notfav@example.com",
            "password": "securepassword123"
        }
    });

    let user_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user_data)
        .send()
        .await
        .expect("Failed to register user");

    let user_body: Value = user_response
        .json()
        .await
        .expect("Failed to parse user response");

    let user_token = user_body["user"]["token"].as_str().unwrap();

    // Act - Try to unfavorite without favoriting first
    let response = app
        .client
        .delete(format!("{}/api/articles/{}/favorite", &app.address, slug))
        .header("Authorization", format!("Bearer {}", user_token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should still work (idempotent)
    assert_eq!(response.status(), StatusCode::OK);

    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response body");

    assert_eq!(response_body["article"]["favorited"], false);
    assert_eq!(response_body["article"]["favoritesCount"], 0);
}

#[tokio::test]
async fn test_get_articles_favorited_by_user() {
    // Arrange
    let app = spawn_app().await;

    // Register author and create articles
    let author_data = json!({
        "user": {
            "username": "author",
            "email": "author@example.com",
            "password": "securepassword123"
        }
    });

    let author_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&author_data)
        .send()
        .await
        .expect("Failed to register author");

    let author_body: Value = author_response
        .json()
        .await
        .expect("Failed to parse author response");

    let author_token = author_body["user"]["token"].as_str().unwrap();

    // Create two articles
    let article1_data = json!({
        "article": {
            "title": "First Article",
            "description": "First article description",
            "body": "First article content",
            "tagList": ["first"]
        }
    });

    let article2_data = json!({
        "article": {
            "title": "Second Article",
            "description": "Second article description",
            "body": "Second article content",
            "tagList": ["second"]
        }
    });

    let create1_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", author_token))
        .json(&article1_data)
        .send()
        .await
        .expect("Failed to create first article");

    let create2_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", author_token))
        .json(&article2_data)
        .send()
        .await
        .expect("Failed to create second article");

    let create1_body: Value = create1_response
        .json()
        .await
        .expect("Failed to parse create1 response");

    let create2_body: Value = create2_response
        .json()
        .await
        .expect("Failed to parse create2 response");

    let slug1 = create1_body["article"]["slug"].as_str().unwrap();
    let _slug2 = create2_body["article"]["slug"].as_str().unwrap();

    // Register user who will favorite
    let user_data = json!({
        "user": {
            "username": "favoriter",
            "email": "favoriter@example.com",
            "password": "securepassword123"
        }
    });

    let user_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .header("Content-Type", "application/json")
        .json(&user_data)
        .send()
        .await
        .expect("Failed to register user");

    let user_body: Value = user_response
        .json()
        .await
        .expect("Failed to parse user response");

    let user_token = user_body["user"]["token"].as_str().unwrap();

    // Favorite only the first article
    app.client
        .post(format!("{}/api/articles/{}/favorite", &app.address, slug1))
        .header("Authorization", format!("Bearer {}", user_token))
        .send()
        .await
        .expect("Failed to favorite first article");

    // Act - Get articles favorited by the user
    let response = app
        .client
        .get(format!("{}/api/articles?favorited=favoriter", &app.address))
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
    assert_eq!(response.status(), StatusCode::OK);

    let response_body: Value = response
        .json()
        .await
        .expect("Failed to parse response body");

    assert!(response_body["articles"].is_array());
    assert_eq!(response_body["articlesCount"], 0);
    assert_eq!(response_body["articles"].as_array().unwrap().len(), 0);
}
