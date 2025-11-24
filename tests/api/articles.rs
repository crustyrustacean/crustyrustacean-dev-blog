// tests/api/articles.rs

use crate::helpers::{TestArticleBuilder, TestFixture, TestUserBuilder, spawn_app};
use crate::{assert_status, bearer_request, parse_json};
use reqwest::StatusCode;
use serde_json::{Value, json};

#[tokio::test]
async fn test_create_article_happy_path() {
    // Arrange
    let app = spawn_app().await;
    let token = app
        .register_user("articleauthor", "author@example.com", "securepassword123")
        .await;

    let article_data = json!({
        "article": {
            "title": "How to Train Your Dragon",
            "description": "Ever wonder how?",
            "body": "You have to believe in yourself. That's the secret to life.",
            "tagList": ["dragons", "training"]
        }
    });

    // Act - Create article
    let response = bearer_request!(
        post & app,
        format!("{}/api/articles", &app.address),
        &token,
        article_data
    )
    .expect("Failed to execute request");

    // Assert
    let status = response.status();
    if status != StatusCode::CREATED {
        let error_body = response.text().await.expect("Failed to get error text");
        panic!("Expected 201 CREATED, got {}: {}", status, error_body);
    }

    let response_body: Value = parse_json!(response);

    // Verify response structure
    assert!(response_body["article"].is_object());
    assert_eq!(
        response_body["article"]["title"],
        "How to Train Your Dragon"
    );
    assert_eq!(response_body["article"]["description"], "Ever wonder how?");
    assert_eq!(
        response_body["article"]["body"],
        "You have to believe in yourself. That's the secret to life."
    );
    assert_eq!(response_body["article"]["slug"], "how-to-train-your-dragon");
    assert_eq!(
        response_body["article"]["tagList"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(response_body["article"]["favorited"], false);
    assert_eq!(response_body["article"]["favoritesCount"], 0);
    assert!(response_body["article"]["author"].is_object());
    assert_eq!(
        response_body["article"]["author"]["username"],
        "articleauthor"
    );
    assert!(response_body["article"]["createdAt"].is_string());
    assert!(response_body["article"]["updatedAt"].is_string());
}

#[tokio::test]
async fn test_create_article_without_auth() {
    // Arrange
    let app = spawn_app().await;

    let article_data = json!({
        "article": {
            "title": "Unauthorized Article",
            "description": "This should fail",
            "body": "No token provided"
        }
    });

    // Act
    let response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Content-Type", "application/json")
        .json(&article_data)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_article_with_invalid_data() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    let test_cases = vec![
        // Empty title
        json!({
            "article": {
                "title": "",
                "description": "Description",
                "body": "Body"
            }
        }),
        // Empty description
        json!({
            "article": {
                "title": "Title",
                "description": "",
                "body": "Body"
            }
        }),
        // Empty body
        json!({
            "article": {
                "title": "Title",
                "description": "Description",
                "body": ""
            }
        }),
        // Missing article wrapper
        json!({
            "title": "Title",
            "description": "Description",
            "body": "Body"
        }),
    ];

    for invalid_data in test_cases {
        // Act
        let response = bearer_request!(
            post & app,
            format!("{}/api/articles", &app.address),
            &token,
            invalid_data
        )
        .expect("Failed to execute request");

        // Assert
        assert_status!(response, StatusCode::BAD_REQUEST);
    }
}

#[tokio::test]
async fn test_get_article_happy_path() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("articlereader").await;

    // Create an article first
    let slug = app
        .create_article(
            &token,
            "Test Article for Reading",
            "This is a test article",
            "Content of the test article",
            vec!["test", "reading"],
        )
        .await;

    // Act - Get the article
    let response = app
        .client
        .get(format!("{}/api/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    assert!(response_body["article"].is_object());
    assert_eq!(
        response_body["article"]["title"],
        "Test Article for Reading"
    );
    assert_eq!(
        response_body["article"]["description"],
        "This is a test article"
    );
    assert_eq!(
        response_body["article"]["body"],
        "Content of the test article"
    );
    assert_eq!(response_body["article"]["slug"], "test-article-for-reading");
    assert_eq!(
        response_body["article"]["tagList"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(response_body["article"]["favorited"], false);
    assert_eq!(response_body["article"]["favoritesCount"], 0);
    assert!(response_body["article"]["author"].is_object());
    assert_eq!(
        response_body["article"]["author"]["username"],
        "articlereader"
    );
}

#[tokio::test]
async fn test_get_nonexistent_article() {
    // Arrange
    let app = spawn_app().await;

    // Act - Try to get a non-existent article
    let response = app
        .client
        .get(format!("{}/api/articles/non-existent-slug", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_article_page_route() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("pageuser").await;

    // Create an article first
    let slug = app
        .create_article(
            &token,
            "Test Article Page",
            "Testing the article page",
            "This is the article content for testing the page.",
            vec!["test", "page"],
        )
        .await;

    // Act - Access the article page (HTML route)
    let response = app
        .client
        .get(format!("{}/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should return HTML page (200 OK)
    assert_status!(response, StatusCode::OK);

    // Check that it returns HTML content
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Should be HTML, not JSON
    assert!(content_type.contains("text/html") || content_type.is_empty()); // Some test servers might not set content-type

    // Check that the response body contains HTML
    let body = response.text().await.expect("Failed to get response body");
    assert!(body.contains("<html") || body.contains("<!DOCTYPE html"));
    assert!(body.contains("Test Article Page")); // Should contain the article title
}

#[tokio::test]
async fn test_create_article_generates_unique_slugs() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("slugtester").await;

    // Create first article
    let article_data1 = json!({
        "article": {
            "title": "Duplicate Title",
            "description": "First article",
            "body": "This is the first article with duplicate title"
        }
    });

    let response1 = bearer_request!(
        post & app,
        format!("{}/api/articles", &app.address),
        &token,
        article_data1
    )
    .expect("Failed to create first article");

    assert_status!(response1, StatusCode::CREATED);

    let body1: Value = parse_json!(response1);
    let slug1 = body1["article"]["slug"].as_str().unwrap();

    // Create second article with same title
    let article_data2 = json!({
        "article": {
            "title": "Duplicate Title",
            "description": "Second article",
            "body": "This is the second article with duplicate title"
        }
    });

    let response2 = bearer_request!(
        post & app,
        format!("{}/api/articles", &app.address),
        &token,
        article_data2
    )
    .expect("Failed to create second article");

    assert_status!(response2, StatusCode::CREATED);

    let body2: Value = parse_json!(response2);
    let slug2 = body2["article"]["slug"].as_str().unwrap();

    // Assert - Slugs should be different
    assert_ne!(slug1, slug2);
    assert_eq!(slug1, "duplicate-title");
    // Second slug should have some suffix to make it unique
    assert!(slug2.starts_with("duplicate-title"));
    assert!(slug2.len() > "duplicate-title".len());
}

#[tokio::test]
async fn test_update_article_happy_path() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("updateuser").await;

    // Create an article first
    let slug = app
        .create_article(
            &token,
            "Original Title",
            "Original description",
            "Original body content",
            vec!["original"],
        )
        .await;

    // Prepare update data
    let update_data = json!({
        "article": {
            "title": "Updated Title",
            "description": "Updated description",
            "body": "Updated body content"
        }
    });

    // Act - Update the article
    let response = bearer_request!(
        put & app,
        format!("{}/api/articles/{}", &app.address, slug),
        &token,
        update_data
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    assert!(response_body["article"].is_object());
    assert_eq!(response_body["article"]["title"], "Updated Title");
    assert_eq!(
        response_body["article"]["description"],
        "Updated description"
    );
    assert_eq!(response_body["article"]["body"], "Updated body content");
    assert_eq!(response_body["article"]["slug"], slug); // Slug should remain the same
    assert!(response_body["article"]["author"].is_object());
    assert_eq!(response_body["article"]["author"]["username"], "updateuser");
}

#[tokio::test]
async fn test_update_article_unauthorized() {
    // Arrange - Use TestFixture for multi-user scenario
    let fixture = TestFixture::new()
        .await
        .with_user_default("author1")
        .await
        .with_user_default("author2")
        .await;

    let token1 = fixture.get_token("author1");
    let token2 = fixture.get_token("author2");

    // Create article with user1
    let slug = fixture
        .app
        .create_article_simple(&token1, "User1 Article")
        .await;

    // Try to update user1's article with user2's token
    let update_data = json!({
        "article": {
            "title": "Hacked Title"
        }
    });

    // Act
    let response = bearer_request!(
        put & fixture.app,
        format!("{}/api/articles/{}", &fixture.app.address, slug),
        &token2,
        update_data
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_delete_article_happy_path() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("deleteuser").await;

    // Create an article first
    let slug = app
        .create_article(
            &token,
            "Article to Delete",
            "This will be deleted",
            "Delete me",
            vec!["delete", "test"],
        )
        .await;

    // Act - Delete the article
    let response = bearer_request!(
        delete & app,
        format!("{}/api/articles/{}", &app.address, slug),
        &token
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::NO_CONTENT);

    // Verify article is actually deleted
    let get_response = app
        .client
        .get(format!("{}/api/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to execute get request");

    assert_status!(get_response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_article_unauthorized() {
    // Arrange - Use TestFixture for multi-user scenario
    let fixture = TestFixture::new()
        .await
        .with_user_default("owner")
        .await
        .with_user_default("hacker")
        .await;

    let token1 = fixture.get_token("owner");
    let token2 = fixture.get_token("hacker");

    // Create article
    let slug = fixture
        .app
        .create_article_simple(&token1, "Protected Article")
        .await;

    // Act - Try to delete with wrong user
    let response = bearer_request!(
        delete & fixture.app,
        format!("{}/api/articles/{}", &fixture.app.address, slug),
        &token2
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::FORBIDDEN);

    // Verify article still exists
    let get_response = fixture
        .app
        .client
        .get(format!("{}/api/articles/{}", &fixture.app.address, slug))
        .send()
        .await
        .expect("Failed to execute get request");

    assert_status!(get_response, StatusCode::OK);
}

#[tokio::test]
async fn test_list_articles_happy_path() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("listuser").await;

    // Create multiple articles
    let articles = vec![
        json!({
            "article": {
                "title": "First Article",
                "description": "First description",
                "body": "First body",
                "tagList": ["first", "test"]
            }
        }),
        json!({
            "article": {
                "title": "Second Article",
                "description": "Second description",
                "body": "Second body",
                "tagList": ["second"]
            }
        }),
    ];

    for article_data in &articles {
        bearer_request!(
            post & app,
            format!("{}/api/articles", &app.address),
            &token,
            article_data
        )
        .expect("Failed to create article");
    }

    // Act - List articles
    let response = app
        .client
        .get(format!("{}/api/articles", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    assert!(response_body["articles"].is_array());
    assert_eq!(response_body["articlesCount"], 2);

    let articles_array = response_body["articles"].as_array().unwrap();
    assert_eq!(articles_array.len(), 2);

    // Verify article structure
    for article in articles_array {
        assert!(article["title"].is_string());
        assert!(article["description"].is_string());
        assert!(article["body"].is_string());
        assert!(article["slug"].is_string());
        assert!(article["tagList"].is_array());
        assert!(article["author"].is_object());
        assert_eq!(article["author"]["username"], "listuser");
        assert!(article["createdAt"].is_string());
        assert!(article["updatedAt"].is_string());
    }
}

#[tokio::test]
async fn test_list_articles_with_filters() {
    // Arrange - Use TestFixture for multi-user scenario
    let fixture = TestFixture::new()
        .await
        .with_user_default("author1")
        .await
        .with_user_default("author2")
        .await;

    let token1 = fixture.get_token("author1");
    let token2 = fixture.get_token("author2");

    // Create articles with different tags and authors
    let article1 = json!({
        "article": {
            "title": "Rust Article",
            "description": "About Rust",
            "body": "Rust content",
            "tagList": ["rust", "programming"]
        }
    });

    let article2 = json!({
        "article": {
            "title": "Python Article",
            "description": "About Python",
            "body": "Python content",
            "tagList": ["python", "programming"]
        }
    });

    // Create articles by different authors
    bearer_request!(
        post & fixture.app,
        format!("{}/api/articles", &fixture.app.address),
        &token1,
        article1
    )
    .expect("Failed to create article1");

    bearer_request!(
        post & fixture.app,
        format!("{}/api/articles", &fixture.app.address),
        &token2,
        article2
    )
    .expect("Failed to create article2");

    // Act & Assert - Filter by author
    let response = fixture
        .app
        .client
        .get(format!(
            "{}/api/articles?author=author1",
            &fixture.app.address
        ))
        .send()
        .await
        .expect("Failed to execute request");

    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    assert_eq!(response_body["articlesCount"], 1);
    assert_eq!(
        response_body["articles"][0]["author"]["username"],
        "author1"
    );

    // Act & Assert - Filter by tag
    let response = fixture
        .app
        .client
        .get(format!("{}/api/articles?tag=rust", &fixture.app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    assert_eq!(response_body["articlesCount"], 1);
    let tag_list = response_body["articles"][0]["tagList"].as_array().unwrap();
    assert!(tag_list.contains(&json!("rust")));
}

#[tokio::test]
async fn test_update_nonexistent_article() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    let update_data = json!({
        "article": {
            "title": "Updated Title"
        }
    });

    // Act - Try to update non-existent article
    let response = bearer_request!(
        put & app,
        format!("{}/api/articles/non-existent-slug", &app.address),
        &token,
        update_data
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_nonexistent_article() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("testuser").await;

    // Act - Try to delete non-existent article
    let response = bearer_request!(
        delete & app,
        format!("{}/api/articles/non-existent-slug", &app.address),
        &token
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::NOT_FOUND);
}

// Phase 4: Tag Editing Tests

#[tokio::test]
async fn test_update_article_with_new_tags() {
    // Arrange
    let app = spawn_app().await;

    // Register user and get token
    let register_body = json!({
        "user": {
            "username": "tagupdateuser",
            "email": "tagupdate@example.com",
            "password": "password123"
        }
    });

    let register_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .json(&register_body)
        .send()
        .await
        .expect("Failed to execute request");

    let register_json: Value = register_response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    let token = register_json["user"]["token"]
        .as_str()
        .expect("Token not found in response");

    // Create an article with initial tags
    let article_body = json!({
        "article": {
            "title": "Article for Tag Update",
            "description": "Testing tag updates",
            "body": "Initial content",
            "tagList": ["tutorial", "beginner"]
        }
    });

    let create_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&article_body)
        .send()
        .await
        .expect("Failed to execute request");

    let create_json: Value = create_response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    let slug = create_json["article"]["slug"]
        .as_str()
        .expect("Slug not found in response");

    // Act - Update with different tags (remove "tutorial", add "advanced")
    let update_body = json!({
        "article": {
            "tagList": ["beginner", "advanced"]
        }
    });

    let response = app
        .client
        .put(format!("{}/api/articles/{}", &app.address, slug))
        .header("Authorization", format!("Bearer {}", token))
        .json(&update_body)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let response_json: Value = response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    let tag_list = response_json["article"]["tagList"]
        .as_array()
        .expect("tagList not found or not an array");

    assert_eq!(tag_list.len(), 2);
    assert!(tag_list.contains(&json!("beginner")));
    assert!(tag_list.contains(&json!("advanced")));
    assert!(!tag_list.contains(&json!("tutorial")));
}

#[tokio::test]
async fn test_update_article_removing_all_tags() {
    // Arrange
    let app = spawn_app().await;

    // Register user and get token
    let register_body = json!({
        "user": {
            "username": "tagremoveuser",
            "email": "tagremove@example.com",
            "password": "password123"
        }
    });

    let register_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .json(&register_body)
        .send()
        .await
        .expect("Failed to execute request");

    let register_json: Value = register_response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    let token = register_json["user"]["token"]
        .as_str()
        .expect("Token not found in response");

    // Create an article with tags
    let article_body = json!({
        "article": {
            "title": "Article with Tags to Remove",
            "description": "Testing tag removal",
            "body": "Initial content with tags",
            "tagList": ["tag1", "tag2", "tag3"]
        }
    });

    let create_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&article_body)
        .send()
        .await
        .expect("Failed to execute request");

    let create_json: Value = create_response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    let slug = create_json["article"]["slug"]
        .as_str()
        .expect("Slug not found in response");

    // Act - Update with empty tag list
    let update_body = json!({
        "article": {
            "tagList": []
        }
    });

    let response = app
        .client
        .put(format!("{}/api/articles/{}", &app.address, slug))
        .header("Authorization", format!("Bearer {}", token))
        .json(&update_body)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let response_json: Value = response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    let tag_list = response_json["article"]["tagList"]
        .as_array()
        .expect("tagList not found or not an array");

    assert_eq!(tag_list.len(), 0);
}

#[tokio::test]
async fn test_update_article_without_touching_tags() {
    // Arrange
    let app = spawn_app().await;

    // Register user and get token
    let register_body = json!({
        "user": {
            "username": "tagunchangeduser",
            "email": "tagunchanged@example.com",
            "password": "password123"
        }
    });

    let register_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .json(&register_body)
        .send()
        .await
        .expect("Failed to execute request");

    let register_json: Value = register_response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    let token = register_json["user"]["token"]
        .as_str()
        .expect("Token not found in response");

    // Create an article with tags
    let article_body = json!({
        "article": {
            "title": "Original Title",
            "description": "Original description",
            "body": "Original content",
            "tagList": ["persistent", "unchanged"]
        }
    });

    let create_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&article_body)
        .send()
        .await
        .expect("Failed to execute request");

    let create_json: Value = create_response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    let slug = create_json["article"]["slug"]
        .as_str()
        .expect("Slug not found in response");

    // Act - Update only the title, don't include tagList
    let update_body = json!({
        "article": {
            "title": "Updated Title Only"
        }
    });

    let response = app
        .client
        .put(format!("{}/api/articles/{}", &app.address, slug))
        .header("Authorization", format!("Bearer {}", token))
        .json(&update_body)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let response_json: Value = response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    // Title should be updated
    assert_eq!(response_json["article"]["title"], "Updated Title Only");

    // Tags should remain unchanged
    let tag_list = response_json["article"]["tagList"]
        .as_array()
        .expect("tagList not found or not an array");

    assert_eq!(tag_list.len(), 2);
    assert!(tag_list.contains(&json!("persistent")));
    assert!(tag_list.contains(&json!("unchanged")));
}

#[tokio::test]
async fn test_add_tags_to_tagless_article() {
    // Arrange
    let app = spawn_app().await;

    // Register user and get token
    let register_body = json!({
        "user": {
            "username": "tagadduser",
            "email": "tagadd@example.com",
            "password": "password123"
        }
    });

    let register_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .json(&register_body)
        .send()
        .await
        .expect("Failed to execute request");

    let register_json: Value = register_response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    let token = register_json["user"]["token"]
        .as_str()
        .expect("Token not found in response");

    // Create an article without tags
    let article_body = json!({
        "article": {
            "title": "Article Without Tags",
            "description": "No tags initially",
            "body": "Content without tags"
        }
    });

    let create_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&article_body)
        .send()
        .await
        .expect("Failed to execute request");

    let create_json: Value = create_response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    let slug = create_json["article"]["slug"]
        .as_str()
        .expect("Slug not found in response");

    // Verify article was created without tags
    let initial_tag_list = create_json["article"]["tagList"]
        .as_array()
        .expect("tagList not found or not an array");
    assert_eq!(initial_tag_list.len(), 0);

    // Act - Add tags to the tagless article
    let update_body = json!({
        "article": {
            "tagList": ["new", "added", "tags"]
        }
    });

    let response = app
        .client
        .put(format!("{}/api/articles/{}", &app.address, slug))
        .header("Authorization", format!("Bearer {}", token))
        .json(&update_body)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let response_json: Value = response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    let tag_list = response_json["article"]["tagList"]
        .as_array()
        .expect("tagList not found or not an array");

    assert_eq!(tag_list.len(), 3);
    assert!(tag_list.contains(&json!("new")));
    assert!(tag_list.contains(&json!("added")));
    assert!(tag_list.contains(&json!("tags")));
}

#[tokio::test]
async fn test_tag_editing_authorization() {
    // Arrange
    let app = spawn_app().await;

    // Register two users
    let user1_body = json!({
        "user": {
            "username": "tagowner",
            "email": "tagowner@example.com",
            "password": "password123"
        }
    });

    let user2_body = json!({
        "user": {
            "username": "tagthief",
            "email": "tagthief@example.com",
            "password": "password123"
        }
    });

    let reg1_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .json(&user1_body)
        .send()
        .await
        .expect("Failed to execute request");

    let reg2_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .json(&user2_body)
        .send()
        .await
        .expect("Failed to execute request");

    let reg1_json: Value = reg1_response
        .json()
        .await
        .expect("Failed to parse response body as JSON");
    let reg2_json: Value = reg2_response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    let token1 = reg1_json["user"]["token"]
        .as_str()
        .expect("Token not found in response");
    let token2 = reg2_json["user"]["token"]
        .as_str()
        .expect("Token not found in response");

    // Create article with user1
    let article_body = json!({
        "article": {
            "title": "Protected Article",
            "description": "Only owner can modify tags",
            "body": "Protected content",
            "tagList": ["protected", "original"]
        }
    });

    let create_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token1))
        .json(&article_body)
        .send()
        .await
        .expect("Failed to execute request");

    let create_json: Value = create_response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    let slug = create_json["article"]["slug"]
        .as_str()
        .expect("Slug not found in response");

    // Act - Try to modify tags with user2's token
    let update_body = json!({
        "article": {
            "tagList": ["hacked", "stolen"]
        }
    });

    let response = app
        .client
        .put(format!("{}/api/articles/{}", &app.address, slug))
        .header("Authorization", format!("Bearer {}", token2))
        .json(&update_body)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 403);

    // Verify original tags are unchanged by checking with a GET request
    let get_response = app
        .client
        .get(format!("{}/api/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(get_response.status().as_u16(), 200);

    let get_json: Value = get_response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    let tag_list = get_json["article"]["tagList"]
        .as_array()
        .expect("tagList not found or not an array");

    assert_eq!(tag_list.len(), 2);
    assert!(tag_list.contains(&json!("protected")));
    assert!(tag_list.contains(&json!("original")));
    assert!(!tag_list.contains(&json!("hacked")));
    assert!(!tag_list.contains(&json!("stolen")));
}

#[tokio::test]
async fn test_tag_updates_idempotency() {
    // Arrange
    let app = spawn_app().await;

    // Register user and get token
    let register_body = json!({
        "user": {
            "username": "idempotentuser",
            "email": "idempotent@example.com",
            "password": "password123"
        }
    });

    let register_response = app
        .client
        .post(format!("{}/api/users", &app.address))
        .json(&register_body)
        .send()
        .await
        .expect("Failed to execute request");

    let register_json: Value = register_response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    let token = register_json["user"]["token"]
        .as_str()
        .expect("Token not found in response");

    // Create an article with tags
    let article_body = json!({
        "article": {
            "title": "Idempotency Test Article",
            "description": "Testing idempotent tag updates",
            "body": "Content for idempotency test",
            "tagList": ["stable", "consistent"]
        }
    });

    let create_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&article_body)
        .send()
        .await
        .expect("Failed to execute request");

    let create_json: Value = create_response
        .json()
        .await
        .expect("Failed to parse response body as JSON");

    let slug = create_json["article"]["slug"]
        .as_str()
        .expect("Slug not found in response");

    // Update data with same tags
    let update_body = json!({
        "article": {
            "tagList": ["stable", "consistent"]
        }
    });

    // Act - Update article with same tags multiple times
    for i in 1..=3 {
        let response = app
            .client
            .put(format!("{}/api/articles/{}", &app.address, slug))
            .header("Authorization", format!("Bearer {}", token))
            .json(&update_body)
            .send()
            .await
            .unwrap_or_else(|_| panic!("Failed to execute request {}", i));

        // Assert - Each update should succeed
        assert_eq!(response.status().as_u16(), 200);

        let response_json: Value = response
            .json()
            .await
            .unwrap_or_else(|_| panic!("Failed to parse response body {}", i));

        // Tags should remain consistent
        let tag_list = response_json["article"]["tagList"]
            .as_array()
            .expect("tagList not found or not an array");

        assert_eq!(tag_list.len(), 2);
        assert!(tag_list.contains(&json!("stable")));
        assert!(tag_list.contains(&json!("consistent")));
    }
}

// Draft Status Tests

#[tokio::test]
async fn test_create_article_as_draft() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("draftauthor").await;

    let article_data = json!({
        "article": {
            "title": "Draft Article",
            "description": "This is a draft",
            "body": "Draft content that is not ready for publication",
            "draft": true,
            "tagList": ["draft", "wip"]
        }
    });

    // Act
    let response = bearer_request!(
        post & app,
        format!("{}/api/articles", &app.address),
        &token,
        article_data
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::CREATED);
    let response_body: Value = parse_json!(response);

    assert_eq!(response_body["article"]["title"], "Draft Article");
    assert_eq!(response_body["article"]["draft"], true);
}

#[tokio::test]
async fn test_create_article_as_published() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("publishedauthor").await;

    let article_data = json!({
        "article": {
            "title": "Published Article",
            "description": "This is published",
            "body": "Published content",
            "draft": false
        }
    });

    // Act
    let response = bearer_request!(
        post & app,
        format!("{}/api/articles", &app.address),
        &token,
        article_data
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::CREATED);
    let response_body: Value = parse_json!(response);

    assert_eq!(response_body["article"]["title"], "Published Article");
    assert_eq!(response_body["article"]["draft"], false);
}

#[tokio::test]
async fn test_create_article_without_draft_field_defaults_to_published() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("defaultauthor").await;

    let article_data = json!({
        "article": {
            "title": "Default Status Article",
            "description": "No draft field specified",
            "body": "Should default to published"
        }
    });

    // Act
    let response = bearer_request!(
        post & app,
        format!("{}/api/articles", &app.address),
        &token,
        article_data
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::CREATED);
    let response_body: Value = parse_json!(response);

    // Should default to published (draft = false)
    assert_eq!(response_body["article"]["draft"], false);
}

#[tokio::test]
async fn test_update_article_to_draft() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("statuschanger").await;

    // Create a published article
    let slug = app.create_article_simple(&token, "Published Article").await;

    // Act - Update to draft
    let update_data = json!({
        "article": {
            "draft": true
        }
    });

    let response = bearer_request!(
        put & app,
        format!("{}/api/articles/{}", &app.address, slug),
        &token,
        update_data
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);
    assert_eq!(response_body["article"]["draft"], true);
}

#[tokio::test]
async fn test_update_article_from_draft_to_published() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("publisher").await;

    // Create a draft article
    let draft_data = json!({
        "article": {
            "title": "Draft to Publish",
            "description": "Testing publication",
            "body": "Draft content",
            "draft": true
        }
    });

    let create_response = bearer_request!(
        post & app,
        format!("{}/api/articles", &app.address),
        &token,
        draft_data
    )
    .expect("Failed to create draft");

    let create_body: Value = parse_json!(create_response);
    let slug = create_body["article"]["slug"].as_str().unwrap();

    // Act - Publish the draft
    let update_data = json!({
        "article": {
            "draft": false
        }
    });

    let response = bearer_request!(
        put & app,
        format!("{}/api/articles/{}", &app.address, slug),
        &token,
        update_data
    )
    .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);
    assert_eq!(response_body["article"]["draft"], false);
}

#[tokio::test]
async fn test_list_articles_excludes_drafts_from_other_users() {
    // Arrange
    let fixture = TestFixture::new()
        .await
        .with_user_default("author1")
        .await
        .with_user_default("author2")
        .await;

    let token1 = fixture.get_token("author1");
    let token2 = fixture.get_token("author2");

    // Author1 creates a published article
    let published_data = json!({
        "article": {
            "title": "Published by Author1",
            "description": "Public article",
            "body": "Everyone can see this",
            "draft": false
        }
    });

    bearer_request!(
        post & fixture.app,
        format!("{}/api/articles", &fixture.app.address),
        &token1,
        published_data
    )
    .expect("Failed to create published article");

    // Author2 creates a draft article
    let draft_data = json!({
        "article": {
            "title": "Draft by Author2",
            "description": "Private draft",
            "body": "Only author2 should see this",
            "draft": true
        }
    });

    bearer_request!(
        post & fixture.app,
        format!("{}/api/articles", &fixture.app.address),
        &token2,
        draft_data
    )
    .expect("Failed to create draft article");

    // Act - List articles without authentication
    let response = fixture
        .app
        .client
        .get(format!("{}/api/articles", &fixture.app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    // Should only show the published article, not the draft
    assert_eq!(response_body["articlesCount"], 1);
    let articles = response_body["articles"].as_array().unwrap();
    assert_eq!(articles[0]["title"], "Published by Author1");
}

#[tokio::test]
async fn test_list_articles_excludes_own_drafts_by_default() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("mixedauthor").await;

    // Create a published article
    let published_data = json!({
        "article": {
            "title": "Published Article",
            "description": "Public",
            "body": "Published content",
            "draft": false
        }
    });

    bearer_request!(
        post & app,
        format!("{}/api/articles", &app.address),
        &token,
        published_data
    )
    .expect("Failed to create published article");

    // Create a draft article
    let draft_data = json!({
        "article": {
            "title": "Draft Article",
            "description": "Private",
            "body": "Draft content",
            "draft": true
        }
    });

    bearer_request!(
        post & app,
        format!("{}/api/articles", &app.address),
        &token,
        draft_data
    )
    .expect("Failed to create draft article");

    // Act - List articles (without includeDrafts parameter)
    let response = app
        .client
        .get(format!("{}/api/articles", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    // By default, should only show published articles
    assert_eq!(response_body["articlesCount"], 1);
    let articles = response_body["articles"].as_array().unwrap();
    assert_eq!(articles[0]["title"], "Published Article");
}

#[tokio::test]
async fn test_get_draft_article_as_author() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("draftowner").await;

    // Create a draft article
    let draft_data = json!({
        "article": {
            "title": "My Private Draft",
            "description": "Draft article",
            "body": "Draft content",
            "draft": true
        }
    });

    let create_response = bearer_request!(
        post & app,
        format!("{}/api/articles", &app.address),
        &token,
        draft_data
    )
    .expect("Failed to create draft");

    let create_body: Value = parse_json!(create_response);
    let slug = create_body["article"]["slug"].as_str().unwrap();

    // Act - Get the draft article as the author
    let response = app
        .client
        .get(format!("{}/api/articles/{}", &app.address, slug))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Author should be able to access their own draft
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);
    assert_eq!(response_body["article"]["title"], "My Private Draft");
    assert_eq!(response_body["article"]["draft"], true);
}

#[tokio::test]
async fn test_get_draft_article_as_non_author_fails() {
    // Arrange
    let fixture = TestFixture::new()
        .await
        .with_user_default("draftauthor")
        .await
        .with_user_default("otheruser")
        .await;

    let author_token = fixture.get_token("draftauthor");
    let other_token = fixture.get_token("otheruser");

    // Create a draft article
    let draft_data = json!({
        "article": {
            "title": "Private Draft",
            "description": "Should not be visible to others",
            "body": "Secret draft content",
            "draft": true
        }
    });

    let create_response = bearer_request!(
        post & fixture.app,
        format!("{}/api/articles", &fixture.app.address),
        &author_token,
        draft_data
    )
    .expect("Failed to create draft");

    let create_body: Value = parse_json!(create_response);
    let slug = create_body["article"]["slug"].as_str().unwrap();

    // Act - Try to get the draft article as a different user
    let response = fixture
        .app
        .client
        .get(format!("{}/api/articles/{}", &fixture.app.address, slug))
        .header("Authorization", format!("Bearer {}", other_token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Non-author should not be able to access the draft
    assert_status!(response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_draft_article_without_auth_fails() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("draftauthor").await;

    // Create a draft article
    let draft_data = json!({
        "article": {
            "title": "Private Draft",
            "description": "Should not be visible without auth",
            "body": "Secret content",
            "draft": true
        }
    });

    let create_response = bearer_request!(
        post & app,
        format!("{}/api/articles", &app.address),
        &token,
        draft_data
    )
    .expect("Failed to create draft");

    let create_body: Value = parse_json!(create_response);
    let slug = create_body["article"]["slug"].as_str().unwrap();

    // Act - Try to get the draft article without authentication
    let response = app
        .client
        .get(format!("{}/api/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Unauthenticated users should not be able to access drafts
    assert_status!(response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_articles_with_author_filter_excludes_their_drafts() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("filterauthor").await;

    // Create a published article
    let published_data = json!({
        "article": {
            "title": "Published by FilterAuthor",
            "description": "Public",
            "body": "Published content",
            "draft": false
        }
    });

    bearer_request!(
        post & app,
        format!("{}/api/articles", &app.address),
        &token,
        published_data
    )
    .expect("Failed to create published article");

    // Create a draft article
    let draft_data = json!({
        "article": {
            "title": "Draft by FilterAuthor",
            "description": "Private",
            "body": "Draft content",
            "draft": true
        }
    });

    bearer_request!(
        post & app,
        format!("{}/api/articles", &app.address),
        &token,
        draft_data
    )
    .expect("Failed to create draft article");

    // Act - List articles filtered by author
    let response = app
        .client
        .get(format!("{}/api/articles?author=filterauthor", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should only show published articles, not drafts
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    assert_eq!(response_body["articlesCount"], 1);
    let articles = response_body["articles"].as_array().unwrap();
    assert_eq!(articles[0]["title"], "Published by FilterAuthor");
}

#[tokio::test]
async fn test_update_draft_status_authorization() {
    // Arrange
    let fixture = TestFixture::new()
        .await
        .with_user_default("owner")
        .await
        .with_user_default("hacker")
        .await;

    let owner_token = fixture.get_token("owner");
    let hacker_token = fixture.get_token("hacker");

    // Owner creates a draft
    let draft_data = json!({
        "article": {
            "title": "Owner's Draft",
            "description": "Private draft",
            "body": "Draft content",
            "draft": true
        }
    });

    let create_response = bearer_request!(
        post & fixture.app,
        format!("{}/api/articles", &fixture.app.address),
        &owner_token,
        draft_data
    )
    .expect("Failed to create draft");

    let create_body: Value = parse_json!(create_response);
    let slug = create_body["article"]["slug"].as_str().unwrap();

    // Act - Try to publish the draft as a different user
    let update_data = json!({
        "article": {
            "draft": false
        }
    });

    let response = bearer_request!(
        put & fixture.app,
        format!("{}/api/articles/{}", &fixture.app.address, slug),
        &hacker_token,
        update_data
    )
    .expect("Failed to execute request");

    // Assert - Should be forbidden
    assert_status!(response, StatusCode::FORBIDDEN);

    // Verify the draft status hasn't changed by checking as the owner
    let get_response = fixture
        .app
        .client
        .get(format!("{}/api/articles/{}", &fixture.app.address, slug))
        .header("Authorization", format!("Bearer {}", owner_token))
        .send()
        .await
        .expect("Failed to execute request");

    assert_status!(get_response, StatusCode::OK);
    let get_body: Value = parse_json!(get_response);
    assert_eq!(get_body["article"]["draft"], true);
}

// ============================================================================
// Markdown File Upload Tests
// ============================================================================

#[tokio::test]
async fn test_upload_markdown_file_happy_path() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("mdauthor").await;

    // Create a markdown file content
    let markdown_content = r#"# My Amazing Article

This is the first paragraph that serves as a description.

## Introduction

Here is the main content of the article with more details.

- Point 1
- Point 2
- Point 3

## Conclusion

That's all folks!
"#;

    // Create a multipart form with the markdown file
    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(markdown_content.as_bytes().to_vec())
            .file_name("test-article.md")
            .mime_str("text/markdown")
            .unwrap(),
    );

    // Act
    let response = app
        .client
        .post(format!("{}/api/articles/upload-markdown", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);

    let response_body: Value = parse_json!(response);

    assert_eq!(response_body["success"], true);
    assert_eq!(response_body["title"], "My Amazing Article");
    assert_eq!(
        response_body["description"],
        "This is the first paragraph that serves as a description."
    );
    assert_eq!(response_body["body"], markdown_content);
    assert_eq!(response_body["filename"], "test-article.md");
}

#[tokio::test]
async fn test_upload_markdown_without_title() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("mdauthor2").await;

    // Markdown without H1 heading
    let markdown_content = r#"This is just content without a title heading.

It has multiple paragraphs and sections.

## Section 1

Some content here.
"#;

    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(markdown_content.as_bytes().to_vec())
            .file_name("no-title.md")
            .mime_str("text/markdown")
            .unwrap(),
    );

    // Act
    let response = app
        .client
        .post(format!("{}/api/articles/upload-markdown", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);

    let response_body: Value = parse_json!(response);

    assert_eq!(response_body["success"], true);
    // Should default to "Untitled Article" when no H1 found
    assert_eq!(response_body["title"], "Untitled Article");
    // Should use first 200 chars as description
    assert!(response_body["description"].as_str().unwrap().len() <= 200);
}

#[tokio::test]
async fn test_upload_markdown_without_authentication() {
    // Arrange
    let app = spawn_app().await;

    let markdown_content = "# Test\n\nContent";

    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(markdown_content.as_bytes().to_vec())
            .file_name("test.md")
            .mime_str("text/markdown")
            .unwrap(),
    );

    // Act
    let response = app
        .client
        .post(format!("{}/api/articles/upload-markdown", &app.address))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should require authentication
    assert_status!(response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_upload_invalid_file_type() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("mdauthor3").await;

    // Try to upload a non-markdown file
    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(b"not markdown".to_vec())
            .file_name("test.txt")
            .mime_str("text/plain")
            .unwrap(),
    );

    // Act
    let response = app
        .client
        .post(format!("{}/api/articles/upload-markdown", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should reject non-markdown files
    assert_status!(response, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_upload_markdown_file_too_large() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("mdauthor4").await;

    // Create a file larger than 1MB
    let large_content = "a".repeat(1024 * 1024 + 1); // 1MB + 1 byte

    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(large_content.as_bytes().to_vec())
            .file_name("large.md")
            .mime_str("text/markdown")
            .unwrap(),
    );

    // Act
    let response = app
        .client
        .post(format!("{}/api/articles/upload-markdown", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Should reject files over 1MB
    assert_status!(response, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_upload_markdown_with_complex_formatting() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("mdauthor5").await;

    // Markdown with complex formatting
    let markdown_content = r#"# Advanced Rust Patterns

Learn about advanced patterns in Rust programming.

## Ownership and Borrowing

Rust's ownership system is **unique** and *powerful*.

```rust
fn main() {
    let x = String::from("hello");
    println!("{}", x);
}
```

### Code Examples

- Ownership rules
- Borrowing rules
- Lifetimes

> This is a quote about Rust

[Link to docs](https://doc.rust-lang.org)
"#;

    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(markdown_content.as_bytes().to_vec())
            .file_name("rust-patterns.md")
            .mime_str("text/markdown")
            .unwrap(),
    );

    // Act
    let response = app
        .client
        .post(format!("{}/api/articles/upload-markdown", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);

    let response_body: Value = parse_json!(response);

    assert_eq!(response_body["success"], true);
    assert_eq!(response_body["title"], "Advanced Rust Patterns");
    assert_eq!(
        response_body["description"],
        "Learn about advanced patterns in Rust programming."
    );
    // Ensure the full content is preserved including code blocks
    assert!(response_body["body"].as_str().unwrap().contains("```rust"));
    assert!(
        response_body["body"]
            .as_str()
            .unwrap()
            .contains("fn main()")
    );
}

#[tokio::test]
async fn test_upload_markdown_no_file_provided() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("mdauthor6").await;

    // Create empty form
    let form = reqwest::multipart::Form::new();

    // Act
    let response = app
        .client
        .post(format!("{}/api/articles/upload-markdown", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::BAD_REQUEST);
}

// ===== Pagination Tests =====

#[tokio::test]
async fn test_articles_page_default_pagination() {
    // Arrange - Create 15 articles to test pagination
    let app = spawn_app().await;
    let token = app.register_user_default("paginationtest").await;

    // Create 15 articles
    for i in 1..=15 {
        app.create_article_simple(&token, &format!("Article {}", i))
            .await;
    }

    // Act - Request the articles page without pagination params (should default to 10 per page)
    let response = app
        .client
        .get(format!("{}/articles", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body = response.text().await.expect("Failed to get response body");

    // Should show pagination controls since we have more than 10 articles
    assert!(body.contains("pagination"));
    assert!(body.contains("Next"));
}

#[tokio::test]
async fn test_articles_pagination_with_limit_and_offset() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("paginationtest2").await;

    // Create 25 articles to test multiple pages
    for i in 1..=25 {
        app.create_article_simple(&token, &format!("Test Article {}", i))
            .await;
    }

    // Act - Request first page with limit=10
    let response = app
        .client
        .get(format!("{}/articles?limit=10&offset=0", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert first page
    assert_status!(response, StatusCode::OK);
    let body = response.text().await.expect("Failed to get response body");

    // Should show pagination controls
    assert!(body.contains("Page 1 of 3"));
    assert!(body.contains("Next"));

    // Act - Request second page with limit=10
    let response = app
        .client
        .get(format!("{}/articles?limit=10&offset=10", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert second page
    assert_status!(response, StatusCode::OK);
    let body = response.text().await.expect("Failed to get response body");

    // Should show page 2
    assert!(body.contains("Page 2 of 3"));
    assert!(body.contains("Previous"));
    assert!(body.contains("Next"));

    // Act - Request third page with limit=10
    let response = app
        .client
        .get(format!("{}/articles?limit=10&offset=20", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert third page
    assert_status!(response, StatusCode::OK);
    let body = response.text().await.expect("Failed to get response body");

    // Should show page 3 (last page)
    assert!(body.contains("Page 3 of 3"));
    assert!(body.contains("Previous"));
}

#[tokio::test]
async fn test_articles_pagination_api_endpoint() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("apipaginationtest").await;

    // Create 15 articles
    for i in 1..=15 {
        app.create_article_simple(&token, &format!("API Article {}", i))
            .await;
    }

    // Act - Request first page of articles via API
    let response = app
        .client
        .get(format!("{}/api/articles?limit=10&offset=0", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    // Should return 10 articles
    assert!(response_body["articles"].is_array());
    assert_eq!(response_body["articles"].as_array().unwrap().len(), 10);
    assert_eq!(response_body["articlesCount"], 10);

    // Act - Request second page
    let response = app
        .client
        .get(format!("{}/api/articles?limit=10&offset=10", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let response_body: Value = parse_json!(response);

    // Should return remaining 5 articles
    assert_eq!(response_body["articles"].as_array().unwrap().len(), 5);
    assert_eq!(response_body["articlesCount"], 5);
}

#[tokio::test]
async fn test_articles_no_pagination_controls_with_few_articles() {
    // Arrange - Create only 5 articles (less than default page size)
    let app = spawn_app().await;
    let token = app.register_user_default("fewartices").await;

    // Create 5 articles
    for i in 1..=5 {
        app.create_article_simple(&token, &format!("Few Article {}", i))
            .await;
    }

    // Act
    let response = app
        .client
        .get(format!("{}/articles", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body = response.text().await.expect("Failed to get response body");

    // Should NOT show pagination controls since we have less than 10 articles
    // The pagination section should not appear at all
    assert!(!body.contains("Page 1 of 1"));
}
