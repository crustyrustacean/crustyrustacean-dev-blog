// tests/api/categories.rs

use crate::helpers::{TestArticleBuilder, TestUserBuilder, spawn_app};
use crate::{assert_status, bearer_request, parse_json};
use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn get_categories_returns_empty_list_when_no_categories_exist() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .client
        .get(format!("{}/api/categories", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body = parse_json!(response);
    assert_eq!(body["categories"], json!([]));
}

#[tokio::test]
async fn create_category_happy_path() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("admin").await;

    // Act
    let create_body = json!({
        "category": {
            "name": "Technology",
            "description": "Tech-related articles"
        }
    });

    let response = bearer_request!(
        post & app,
        format!("{}/api/categories", &app.address),
        &token,
        create_body
    )
    .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::CREATED);
    let body = parse_json!(response);
    assert_eq!(body["category"]["name"], "Technology");
    assert_eq!(body["category"]["slug"], "technology");
    assert_eq!(body["category"]["description"], "Tech-related articles");
}

#[tokio::test]
async fn create_category_requires_authentication() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let create_body = json!({
        "category": {
            "name": "Technology"
        }
    });

    let response = app
        .client
        .post(format!("{}/api/categories", &app.address))
        .json(&create_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn create_category_with_duplicate_name() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("admin").await;

    // Create first category
    let create_body = json!({
        "category": {
            "name": "Technology"
        }
    });

    bearer_request!(
        post & app,
        format!("{}/api/categories", &app.address),
        &token,
        create_body.clone()
    )
    .expect("Failed to execute request.");

    // Act - Try to create duplicate
    let response = bearer_request!(
        post & app,
        format!("{}/api/categories", &app.address),
        &token,
        create_body
    )
    .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::CONFLICT);
}

#[tokio::test]
async fn create_category_generates_unique_slug_from_name() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("admin").await;

    // Act
    let create_body = json!({
        "category": {
            "name": "Web Development"
        }
    });

    let response = bearer_request!(
        post & app,
        format!("{}/api/categories", &app.address),
        &token,
        create_body
    )
    .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::CREATED);
    let body = parse_json!(response);
    assert_eq!(body["category"]["slug"], "web-development");
}

#[tokio::test]
async fn get_categories_returns_all_categories() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("admin").await;

    // Create multiple categories
    let categories = vec!["Technology", "Lifestyle", "Programming"];
    for name in categories {
        let create_body = json!({
            "category": {
                "name": name
            }
        });

        bearer_request!(
            post & app,
            format!("{}/api/categories", &app.address),
            &token,
            create_body
        )
        .expect("Failed to execute request.");
    }

    // Act
    let response = app
        .client
        .get(format!("{}/api/categories", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body = parse_json!(response);
    let categories = body["categories"].as_array().expect("categories should be an array");
    assert_eq!(categories.len(), 3);
}

#[tokio::test]
async fn get_categories_returns_categories_in_alphabetical_order() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("admin").await;

    // Create categories in non-alphabetical order
    for name in &["Zebra", "Apple", "Mango"] {
        let create_body = json!({
            "category": {
                "name": name
            }
        });

        bearer_request!(
            post & app,
            format!("{}/api/categories", &app.address),
            &token,
            create_body
        )
        .expect("Failed to execute request.");
    }

    // Act
    let response = app
        .client
        .get(format!("{}/api/categories", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body = parse_json!(response);
    let categories = body["categories"].as_array().unwrap();
    let names: Vec<String> = categories
        .iter()
        .map(|c| c["name"].as_str().unwrap().to_string())
        .collect();

    assert_eq!(names, vec!["Apple", "Mango", "Zebra"]);
}

#[tokio::test]
async fn get_category_by_slug() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("admin").await;

    // Create category
    let create_body = json!({
        "category": {
            "name": "Technology",
            "description": "Tech articles"
        }
    });

    bearer_request!(
        post & app,
        format!("{}/api/categories", &app.address),
        &token,
        create_body
    )
    .expect("Failed to execute request.");

    // Act
    let response = app
        .client
        .get(format!("{}/api/categories/technology", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body = parse_json!(response);
    assert_eq!(body["category"]["name"], "Technology");
    assert_eq!(body["category"]["slug"], "technology");
    assert_eq!(body["category"]["description"], "Tech articles");
}

#[tokio::test]
async fn get_nonexistent_category() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .client
        .get(format!("{}/api/categories/nonexistent", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_category_happy_path() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("admin").await;

    // Create category
    let create_body = json!({
        "category": {
            "name": "Technology"
        }
    });

    bearer_request!(
        post & app,
        format!("{}/api/categories", &app.address),
        &token,
        create_body
    )
    .expect("Failed to execute request.");

    // Act - Update category
    let update_body = json!({
        "category": {
            "name": "Tech & Innovation",
            "description": "Technology and innovation articles"
        }
    });

    let response = bearer_request!(
        put & app,
        format!("{}/api/categories/technology", &app.address),
        &token,
        update_body
    )
    .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body = parse_json!(response);
    assert_eq!(body["category"]["name"], "Tech & Innovation");
    assert_eq!(body["category"]["slug"], "tech-innovation");
}

#[tokio::test]
async fn update_category_requires_authentication() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let update_body = json!({
        "category": {
            "name": "New Name"
        }
    });

    let response = app
        .client
        .put(format!("{}/api/categories/something", &app.address))
        .json(&update_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn delete_category_happy_path() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("admin").await;

    // Create category
    let create_body = json!({
        "category": {
            "name": "Temporary"
        }
    });

    bearer_request!(
        post & app,
        format!("{}/api/categories", &app.address),
        &token,
        create_body
    )
    .expect("Failed to execute request.");

    // Act - Delete category
    let response = bearer_request!(
        delete & app,
        format!("{}/api/categories/temporary", &app.address),
        &token
    )
    .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::NO_CONTENT);

    // Verify category no longer exists
    let get_response = app
        .client
        .get(format!("{}/api/categories/temporary", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_status!(get_response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_category_requires_authentication() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .client
        .delete(format!("{}/api/categories/something", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn delete_nonexistent_category() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("admin").await;

    // Act
    let response = bearer_request!(
        delete & app,
        format!("{}/api/categories/nonexistent", &app.address),
        &token
    )
    .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_article_with_category() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("author").await;

    // Create a category first
    let category_body = json!({
        "category": {
            "name": "Technology"
        }
    });

    bearer_request!(
        post & app,
        format!("{}/api/categories", &app.address),
        &token,
        category_body
    )
    .expect("Failed to create category.");

    // Act - Create article with category
    let article_body = json!({
        "article": {
            "title": "Rust is awesome",
            "description": "Why Rust rocks",
            "body": "Rust provides memory safety without garbage collection.",
            "category": "technology"
        }
    });

    let response = bearer_request!(
        post & app,
        format!("{}/api/articles", &app.address),
        &token,
        article_body
    )
    .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::CREATED);
    let body = parse_json!(response);
    assert_eq!(body["article"]["category"], "technology");
}

#[tokio::test]
async fn create_article_with_nonexistent_category() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("author").await;

    // Act - Try to create article with nonexistent category
    let article_body = json!({
        "article": {
            "title": "Test Article",
            "description": "Description",
            "body": "Body content",
            "category": "nonexistent"
        }
    });

    let response = bearer_request!(
        post & app,
        format!("{}/api/articles", &app.address),
        &token,
        article_body
    )
    .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_article_without_category() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("author").await;

    // Act - Create article without category (should be optional)
    let article_body = json!({
        "article": {
            "title": "Uncategorized Article",
            "description": "No category",
            "body": "Body content"
        }
    });

    let response = bearer_request!(
        post & app,
        format!("{}/api/articles", &app.address),
        &token,
        article_body
    )
    .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::CREATED);
    let body = parse_json!(response);
    assert!(body["article"]["category"].is_null());
}

#[tokio::test]
async fn update_article_category() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("author").await;

    // Create categories
    for name in &["Technology", "Lifestyle"] {
        let category_body = json!({
            "category": {
                "name": name
            }
        });

        bearer_request!(
            post & app,
            format!("{}/api/categories", &app.address),
            &token,
            category_body
        )
        .expect("Failed to create category.");
    }

    // Create article with initial category
    let slug = app
        .create_article(
            &token,
            "Test Article",
            "Description",
            "Body",
            vec!["rust"],
        )
        .await;

    // Set initial category
    let update_body = json!({
        "article": {
            "category": "technology"
        }
    });

    bearer_request!(
        put & app,
        format!("{}/api/articles/{}", &app.address, slug),
        &token,
        update_body
    )
    .expect("Failed to update article.");

    // Act - Update to different category
    let update_body = json!({
        "article": {
            "category": "lifestyle"
        }
    });

    let response = bearer_request!(
        put & app,
        format!("{}/api/articles/{}", &app.address, slug),
        &token,
        update_body
    )
    .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body = parse_json!(response);
    assert_eq!(body["article"]["category"], "lifestyle");
}

#[tokio::test]
async fn filter_articles_by_category() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("author").await;

    // Create categories
    for name in &["Technology", "Lifestyle"] {
        let category_body = json!({
            "category": {
                "name": name
            }
        });

        bearer_request!(
            post & app,
            format!("{}/api/categories", &app.address),
            &token,
            category_body
        )
        .expect("Failed to create category.");
    }

    // Create articles with different categories
    for (title, category) in &[
        ("Tech Article 1", "technology"),
        ("Tech Article 2", "technology"),
        ("Life Article 1", "lifestyle"),
    ] {
        let article_body = json!({
            "article": {
                "title": title,
                "description": "Description",
                "body": "Body content",
                "category": category
            }
        });

        bearer_request!(
            post & app,
            format!("{}/api/articles", &app.address),
            &token,
            article_body
        )
        .expect("Failed to create article.");
    }

    // Act - Filter by technology category
    let response = app
        .client
        .get(format!("{}/api/articles?category=technology", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body = parse_json!(response);
    let articles = body["articles"].as_array().unwrap();
    assert_eq!(articles.len(), 2);

    for article in articles {
        assert_eq!(article["category"], "technology");
    }
}

#[tokio::test]
async fn delete_category_removes_from_articles() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("author").await;

    // Create category
    let category_body = json!({
        "category": {
            "name": "Technology"
        }
    });

    bearer_request!(
        post & app,
        format!("{}/api/categories", &app.address),
        &token,
        category_body
    )
    .expect("Failed to create category.");

    // Create article with category
    let article_body = json!({
        "article": {
            "title": "Tech Article",
            "description": "Description",
            "body": "Body content",
            "category": "technology"
        }
    });

    let create_response = bearer_request!(
        post & app,
        format!("{}/api/articles", &app.address),
        &token,
        article_body
    )
    .expect("Failed to create article.");

    let create_body = parse_json!(create_response);
    let slug = create_body["article"]["slug"].as_str().unwrap();

    // Act - Delete the category
    let delete_response = bearer_request!(
        delete & app,
        format!("{}/api/categories/technology", &app.address),
        &token
    )
    .expect("Failed to execute request.");

    assert_status!(delete_response, StatusCode::NO_CONTENT);

    // Assert - Article should now have null category
    let article_response = app
        .client
        .get(format!("{}/api/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to get article");

    let article_body = parse_json!(article_response);
    assert!(article_body["article"]["category"].is_null());
}

#[tokio::test]
async fn get_category_includes_article_count() {
    // Arrange
    let app = spawn_app().await;
    let token = app.register_user_default("author").await;

    // Create category
    let category_body = json!({
        "category": {
            "name": "Technology"
        }
    });

    bearer_request!(
        post & app,
        format!("{}/api/categories", &app.address),
        &token,
        category_body
    )
    .expect("Failed to create category.");

    // Create articles with category
    for i in 1..=3 {
        let article_body = json!({
            "article": {
                "title": format!("Article {}", i),
                "description": "Description",
                "body": "Body content",
                "category": "technology"
            }
        });

        bearer_request!(
            post & app,
            format!("{}/api/articles", &app.address),
            &token,
            article_body
        )
        .expect("Failed to create article.");
    }

    // Act
    let response = app
        .client
        .get(format!("{}/api/categories/technology", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body = parse_json!(response);
    assert_eq!(body["category"]["articleCount"], 3);
}
