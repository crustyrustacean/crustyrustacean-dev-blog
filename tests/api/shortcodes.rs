// Integration tests for shortcode processing with database
// Tests the full shortcode resolution pipeline

use crate::helpers::{spawn_app, TestArticleBuilder, TestUserBuilder};

#[tokio::test]
async fn test_process_shortcodes_resolves_existing_article() {
    let app = spawn_app().await;

    // Create a user and article
    let token = app.register_user_default("testuser").await;
    let _slug = app
        .create_article(
            &token,
            "Introduction to Rust",
            "A beginner's guide",
            "This is the article body",
            vec![],
        )
        .await;

    // Process shortcode
    let text = "Check out [[article:introduction-to-rust]] for more info";
    let processed = crustyrustacean_dev_blog_lib::shortcodes::process_shortcodes(text, &app.db)
        .await
        .expect("Failed to process shortcodes");

    // Should resolve to markdown link with article title
    assert_eq!(
        processed,
        "Check out [Introduction to Rust](/articles/introduction-to-rust) for more info"
    );
}

#[tokio::test]
async fn test_process_shortcodes_handles_missing_article() {
    let app = spawn_app().await;

    // No article exists with this slug
    let text = "See [[article:non-existent-article]] for details";
    let processed = crustyrustacean_dev_blog_lib::shortcodes::process_shortcodes(text, &app.db)
        .await
        .expect("Failed to process shortcodes");

    // Should create link with "Article not found" tooltip
    assert_eq!(
        processed,
        r#"See [non-existent-article](/articles/non-existent-article "Article not found") for details"#
    );
}

#[tokio::test]
async fn test_process_shortcodes_with_custom_text() {
    let app = spawn_app().await;

    // Create a user and article
    let token = app.register_user_default("testuser").await;
    let _slug = app
        .create_article(
            &token,
            "Advanced Patterns",
            "Deep dive",
            "Article content",
            vec![],
        )
        .await;

    // Process shortcode with custom text
    let text = "Read [[article:advanced-patterns|this advanced guide]] first";
    let processed = crustyrustacean_dev_blog_lib::shortcodes::process_shortcodes(text, &app.db)
        .await
        .expect("Failed to process shortcodes");

    // Should use custom text instead of article title
    assert_eq!(
        processed,
        "Read [this advanced guide](/articles/advanced-patterns) first"
    );
}

#[tokio::test]
async fn test_process_shortcodes_with_custom_text_for_missing_article() {
    let app = spawn_app().await;

    // No article exists
    let text = "Check [[article:missing-slug|this missing article]] out";
    let processed = crustyrustacean_dev_blog_lib::shortcodes::process_shortcodes(text, &app.db)
        .await
        .expect("Failed to process shortcodes");

    // Should use custom text with tooltip
    assert_eq!(
        processed,
        r#"Check [this missing article](/articles/missing-slug "Article not found") out"#
    );
}

#[tokio::test]
async fn test_process_shortcodes_multiple_links() {
    let app = spawn_app().await;

    // Create two articles
    let token = app.register_user_default("testuser").await;
    app.create_article(&token, "First Article", "Desc", "Body", vec![])
        .await;
    app.create_article(&token, "Second Article", "Desc", "Body", vec![])
        .await;

    // Process multiple shortcodes
    let text = "See [[article:first-article]] and [[article:second-article]] for more";
    let processed = crustyrustacean_dev_blog_lib::shortcodes::process_shortcodes(text, &app.db)
        .await
        .expect("Failed to process shortcodes");

    assert_eq!(
        processed,
        "See [First Article](/articles/first-article) and [Second Article](/articles/second-article) for more"
    );
}

#[tokio::test]
async fn test_process_shortcodes_mixed_existing_and_missing() {
    let app = spawn_app().await;

    // Create one article
    let token = app.register_user_default("testuser").await;
    app.create_article(&token, "Existing Article", "Desc", "Body", vec![])
        .await;

    // Mix of existing and missing
    let text = "Check [[article:existing-article]] and [[article:missing-article]] out";
    let processed = crustyrustacean_dev_blog_lib::shortcodes::process_shortcodes(text, &app.db)
        .await
        .expect("Failed to process shortcodes");

    assert_eq!(
        processed,
        r#"Check [Existing Article](/articles/existing-article) and [missing-article](/articles/missing-article "Article not found") out"#
    );
}

#[tokio::test]
async fn test_process_shortcodes_no_shortcodes_returns_original() {
    let app = spawn_app().await;

    let text = "This is just plain text with no shortcodes";
    let processed = crustyrustacean_dev_blog_lib::shortcodes::process_shortcodes(text, &app.db)
        .await
        .expect("Failed to process shortcodes");

    assert_eq!(processed, text);
}

#[tokio::test]
async fn test_process_shortcodes_preserves_markdown() {
    let app = spawn_app().await;

    // Create article
    let token = app.register_user_default("testuser").await;
    app.create_article(&token, "Test Article", "Desc", "Body", vec![])
        .await;

    // Text with existing markdown
    let text = "Check **[[article:test-article]]** and [external link](https://example.com)";
    let processed = crustyrustacean_dev_blog_lib::shortcodes::process_shortcodes(text, &app.db)
        .await
        .expect("Failed to process shortcodes");

    assert_eq!(
        processed,
        "Check **[Test Article](/articles/test-article)** and [external link](https://example.com)"
    );
}

#[tokio::test]
async fn test_process_shortcodes_at_boundaries() {
    let app = spawn_app().await;

    // Create article
    let token = app.register_user_default("testuser").await;
    app.create_article(&token, "Boundary Test", "Desc", "Body", vec![])
        .await;

    // Shortcode at start
    let text = "[[article:boundary-test]] is excellent";
    let processed = crustyrustacean_dev_blog_lib::shortcodes::process_shortcodes(text, &app.db)
        .await
        .expect("Failed to process shortcodes");
    assert_eq!(
        processed,
        "[Boundary Test](/articles/boundary-test) is excellent"
    );

    // Shortcode at end
    let text = "Check out [[article:boundary-test]]";
    let processed = crustyrustacean_dev_blog_lib::shortcodes::process_shortcodes(text, &app.db)
        .await
        .expect("Failed to process shortcodes");
    assert_eq!(
        processed,
        "Check out [Boundary Test](/articles/boundary-test)"
    );
}

#[tokio::test]
async fn test_process_shortcodes_with_hyphens_and_underscores() {
    let app = spawn_app().await;

    // Create article with complex slug
    let token = app.register_user_default("testuser").await;
    app.create_article(&token, "Rust 2024 Edition", "Desc", "Body", vec![])
        .await;

    let text = "See [[article:rust-2024-edition]] for updates";
    let processed = crustyrustacean_dev_blog_lib::shortcodes::process_shortcodes(text, &app.db)
        .await
        .expect("Failed to process shortcodes");

    assert_eq!(
        processed,
        "See [Rust 2024 Edition](/articles/rust-2024-edition) for updates"
    );
}

// ============================================================================
// End-to-End Article Creation Tests
// ============================================================================

#[tokio::test]
async fn test_create_article_with_shortcodes_processes_them() {
    let app = spawn_app().await;

    // Create user and first article (to be referenced)
    let token = app.register_user_default("testuser").await;
    app.create_article(&token, "Referenced Article", "Desc", "Body", vec![])
        .await;

    // Create article with shortcode in body
    let response = app
        .client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "article": {
                "title": "Article With Links",
                "description": "This article has internal links",
                "body": "Check out [[article:referenced-article]] for more info. Also see [[article:non-existent]] which doesn't exist.",
                "tagList": []
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    let stored_body = body["article"]["body"].as_str().unwrap();

    // Debug: print the actual body
    eprintln!("Stored body: {}", stored_body);

    // Verify shortcodes were processed in stored body
    assert!(stored_body.contains("[Referenced Article](/articles/referenced-article)"),
        "Expected processed shortcode in body, got: {}", stored_body);
    assert!(stored_body.contains(r#"[non-existent](/articles/non-existent "Article not found")"#),
        "Expected missing article shortcode in body, got: {}", stored_body);
}

#[tokio::test]
async fn test_update_article_with_shortcodes_processes_them() {
    let app = spawn_app().await;

    // Create user and articles
    let token = app.register_user_default("testuser").await;
    app.create_article(&token, "Target Article", "Desc", "Body", vec![])
        .await;
    let article_slug = app
        .create_article(&token, "Source Article", "Desc", "Original body", vec![])
        .await;

    // Update article with shortcodes
    let response = app
        .client
        .put(format!("{}/api/articles/{}", &app.address, article_slug))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "article": {
                "body": "Updated body with [[article:target-article]] link"
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    let stored_body = body["article"]["body"].as_str().unwrap();

    // Verify shortcode was processed
    assert!(stored_body.contains("[Target Article](/articles/target-article)"));
}

#[tokio::test]
async fn test_article_with_shortcodes_renders_as_html() {
    let app = spawn_app().await;

    // Create user and articles
    let token = app.register_user_default("testuser").await;
    app.create_article(&token, "Referenced Post", "Desc", "Body", vec![])
        .await;

    // Create article with shortcode
    app.client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "article": {
                "title": "Article With Link",
                "description": "Has link",
                "body": "See [[article:referenced-post]] for details",
                "tagList": []
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Fetch the article page
    let response = app
        .client
        .get(format!("{}/articles/article-with-link", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);
    let html = response.text().await.expect("Failed to get response body");

    // Verify the shortcode was converted to markdown, then to HTML
    // The markdown link [Referenced Post](/articles/referenced-post) becomes:
    // <a href="/articles/referenced-post">Referenced Post</a>
    assert!(html.contains(r#"<a href="/articles/referenced-post">Referenced Post</a>"#));
}

#[tokio::test]
async fn test_article_with_missing_reference_shows_tooltip() {
    let app = spawn_app().await;

    // Create user and article with missing reference
    let token = app.register_user_default("testuser").await;
    app.client
        .post(format!("{}/api/articles", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "article": {
                "title": "Broken Link Article",
                "description": "Has broken link",
                "body": "See [[article:does-not-exist]] for nothing",
                "tagList": []
            }
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Fetch the article page
    let response = app
        .client
        .get(format!("{}/articles/broken-link-article", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);
    let html = response.text().await.expect("Failed to get response body");

    // Verify the markdown with tooltip becomes HTML with title attribute
    // [does-not-exist](/articles/does-not-exist "Article not found") becomes:
    // <a href="/articles/does-not-exist" title="Article not found">does-not-exist</a>
    assert!(html.contains(r#"title="Article not found""#));
    assert!(html.contains(r#"href="/articles/does-not-exist""#));
}
