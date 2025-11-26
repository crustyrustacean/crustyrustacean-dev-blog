use crate::helpers::{TestUserBuilder, spawn_app};
use serde_json::json;

#[tokio::test]
async fn test_create_article_with_markdown() {
    // Arrange
    let app = spawn_app().await;

    // Register a test user
    let token = app
        .register_user("testuser", "test@example.com", "password123")
        .await;

    // Create an article with markdown content
    let markdown_content = r#"# Hello World

This is a **bold** statement and this is *italic*.

## Code Example

Here's some `inline code` and a code block:

```rust
fn main() {
    println!("Hello, world!");
}
```

## Lists

- Item 1
- Item 2
- Item 3

## Table

| Column 1 | Column 2 |
|----------|----------|
| Value 1  | Value 2  |

> This is a blockquote

[This is a link](https://example.com)
"#;

    let article_body = json!({
        "article": {
            "title": "Test Markdown Article",
            "description": "A test article with markdown content",
            "body": markdown_content,
            "tagList": ["test", "markdown"]
        }
    });

    // Act
    let response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&article_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 201);

    let article_response: serde_json::Value = response.json().await.unwrap();
    let article = &article_response["article"];

    // Verify the raw markdown is preserved
    assert_eq!(article["body"].as_str().unwrap(), markdown_content);

    // Verify the rendered HTML is present and contains expected elements
    let rendered_body = article["renderedBody"].as_str().unwrap();
    assert!(rendered_body.contains("<h1>Hello World</h1>"));
    assert!(rendered_body.contains("<strong>bold</strong>"));
    assert!(rendered_body.contains("<em>italic</em>"));
    assert!(rendered_body.contains("<code>inline code</code>"));
    assert!(rendered_body.contains("<pre><code class=\"language-rust\">"));
    assert!(rendered_body.contains("fn main()"));
    assert!(rendered_body.contains("<ul>"));
    assert!(rendered_body.contains("<li>Item 1</li>"));
    assert!(rendered_body.contains("<table>"));
    assert!(rendered_body.contains("<th>Column 1</th>"));
    assert!(rendered_body.contains("<td>Value 1</td>"));
    assert!(rendered_body.contains("<blockquote>"));
    assert!(rendered_body.contains("<a href=\"https://example.com\">This is a link</a>"));
}

#[tokio::test]
async fn test_get_article_returns_rendered_markdown() {
    // Arrange
    let app = spawn_app().await;

    // Register user using helper
    let token = app
        .register_user("testuser2", "test2@example.com", "password123")
        .await;

    let article_body = json!({
        "article": {
            "title": "Get Test Article",
            "description": "Testing get endpoint",
            "body": "## Heading\n\nThis is **markdown** content.",
            "tagList": ["get-test"]
        }
    });

    let create_response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&article_body)
        .send()
        .await
        .expect("Failed to execute request.");

    let create_article_response: serde_json::Value = create_response.json().await.unwrap();
    let slug = create_article_response["article"]["slug"].as_str().unwrap();

    // Act - Get the article
    let response = app
        .client
        .get(format!("{}/api/articles/{}", &app.address, slug))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let article_response: serde_json::Value = response.json().await.unwrap();
    let article = &article_response["article"];

    // Verify both raw markdown and rendered HTML are present
    assert_eq!(
        article["body"].as_str().unwrap(),
        "## Heading\n\nThis is **markdown** content."
    );

    let rendered_body = article["renderedBody"].as_str().unwrap();
    assert!(rendered_body.contains("<h2>Heading</h2>"));
    assert!(rendered_body.contains("<strong>markdown</strong>"));
}
