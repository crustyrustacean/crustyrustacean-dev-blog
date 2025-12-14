// tests/api/profile.rs

use crate::assert_status;
use crate::helpers::{
    HtmlResponseValidator, TestArticleBuilder, TestCommentBuilder, TestFixture, spawn_app,
};
use reqwest::StatusCode;

#[tokio::test]
async fn test_profile_page_returns_200_for_existing_user() {
    // Arrange
    let fixture = TestFixture::new()
        .await
        .with_user_default("testauthor")
        .await
        .promote_to_author("testauthor")
        .await;

    let token = fixture.get_token("testauthor");

    // Act - Access profile page
    let response = fixture
        .app
        .client
        .get(format!("{}/profiles/testauthor", &fixture.app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body = response.assert_html_response().await;
    assert!(body.contains("testauthor"));
}

#[tokio::test]
async fn test_profile_page_shows_articles() {
    // Arrange
    let fixture = TestFixture::new()
        .await
        .with_user_default("articleauthor")
        .await
        .promote_to_author("articleauthor")
        .await;

    let token = fixture.get_token("articleauthor");

    // Create some articles
    fixture
        .app
        .create_article(
            &token,
            "Test Article One",
            "Description one",
            "Body one",
            vec!["rust", "testing"],
        )
        .await;

    fixture
        .app
        .create_article(
            &token,
            "Test Article Two",
            "Description two",
            "Body two",
            vec!["web"],
        )
        .await;

    // Act - Access profile page
    let response = fixture
        .app
        .client
        .get(format!("{}/profiles/articleauthor", &fixture.app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body = response.assert_html_response().await;
    assert!(body.contains("Test Article One"));
    assert!(body.contains("Test Article Two"));
    assert!(body.contains("Articles (2)"));
}

#[tokio::test]
async fn test_profile_page_shows_article_tags() {
    // Arrange
    let fixture = TestFixture::new()
        .await
        .with_user_default("tagauthor")
        .await
        .promote_to_author("tagauthor")
        .await;

    let token = fixture.get_token("tagauthor");

    // Create an article with tags
    fixture
        .app
        .create_article(
            &token,
            "Tagged Article",
            "Description",
            "Body content",
            vec!["rust", "async"],
        )
        .await;

    // Act - Access profile page
    let response = fixture
        .app
        .client
        .get(format!("{}/profiles/tagauthor", &fixture.app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body = response.assert_html_response().await;
    assert!(body.contains("rust"));
    assert!(body.contains("async"));
}

#[tokio::test]
async fn test_profile_page_pagination_with_many_articles() {
    // Arrange
    let fixture = TestFixture::new()
        .await
        .with_user_default("paginatedauthor")
        .await
        .promote_to_author("paginatedauthor")
        .await;

    let token = fixture.get_token("paginatedauthor");

    // Create 15 articles (more than default page size of 10)
    for i in 1..=15 {
        fixture
            .app
            .create_article_simple(&token, &format!("Article {}", i))
            .await;
    }

    // Act - Access first page
    let response = fixture
        .app
        .client
        .get(format!("{}/profiles/paginatedauthor", &fixture.app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - First page should show pagination
    assert_status!(response, StatusCode::OK);
    let body = response.assert_html_response().await;
    assert!(body.contains("Articles (15)")); // Total count
    assert!(body.contains("page=2")); // Link to page 2
    assert!(body.contains("Next")); // Next button

    // Act - Access second page
    let response_page2 = fixture
        .app
        .client
        .get(format!(
            "{}/profiles/paginatedauthor?page=2",
            &fixture.app.address
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Second page
    assert_status!(response_page2, StatusCode::OK);
    let body_page2 = response_page2.assert_html_response().await;
    assert!(body_page2.contains("Previous")); // Previous button on page 2
    assert!(body_page2.contains("page=1")); // Link to page 1
}

#[tokio::test]
async fn test_profile_page_shows_favorites_count() {
    // Arrange
    let fixture = TestFixture::new()
        .await
        .with_user_default("favauthor")
        .await
        .with_user_default("fanliker")
        .await
        .promote_to_author("favauthor")
        .await;

    let author_token = fixture.get_token("favauthor");
    let fan_token = fixture.get_token("fanliker");

    // Create an article
    let slug = fixture
        .app
        .create_article_simple(&author_token, "Popular Article")
        .await;

    // Favorite the article
    fixture
        .app
        .client
        .post(format!(
            "{}/api/articles/{}/favorite",
            &fixture.app.address, slug
        ))
        .header("Authorization", format!("Bearer {}", fan_token))
        .send()
        .await
        .expect("Failed to favorite article");

    // Act - Access profile page
    let response = fixture
        .app
        .client
        .get(format!("{}/profiles/favauthor", &fixture.app.address))
        .header("Authorization", format!("Bearer {}", author_token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body = response.assert_html_response().await;
    // The template shows favorites with heart icon
    assert!(body.contains("fa-heart"));
}

#[tokio::test]
async fn test_profile_page_shows_comments_count() {
    // Arrange
    let fixture = TestFixture::new()
        .await
        .with_user_default("commentauthor")
        .await
        .with_user_default("commenter")
        .await
        .promote_to_author("commentauthor")
        .await;

    let author_token = fixture.get_token("commentauthor");
    let commenter_token = fixture.get_token("commenter");

    // Create an article
    let slug = fixture
        .app
        .create_article_simple(&author_token, "Commentable Article")
        .await;

    // Add a comment
    fixture
        .app
        .add_comment(&commenter_token, &slug, "Great article!")
        .await;

    // Act - Access profile page
    let response = fixture
        .app
        .client
        .get(format!("{}/profiles/commentauthor", &fixture.app.address))
        .header("Authorization", format!("Bearer {}", author_token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body = response.assert_html_response().await;
    // The template shows comments with comment icon
    assert!(body.contains("fa-comment"));
}

#[tokio::test]
async fn test_profile_page_without_authentication() {
    // Arrange
    let fixture = TestFixture::new()
        .await
        .with_user_default("publicauthor")
        .await
        .promote_to_author("publicauthor")
        .await;

    let token = fixture.get_token("publicauthor");

    // Create an article
    fixture
        .app
        .create_article_simple(&token, "Public Article")
        .await;

    // Act - Access profile page without authentication
    let response = fixture
        .app
        .client
        .get(format!("{}/profiles/publicauthor", &fixture.app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Profile page should be accessible without auth
    assert_status!(response, StatusCode::OK);
    let body = response.assert_html_response().await;
    assert!(body.contains("publicauthor"));
    assert!(body.contains("Public Article"));
}

#[tokio::test]
async fn test_profile_page_returns_404_for_nonexistent_user() {
    // Arrange
    let app = spawn_app().await;

    // Act - Access profile page for nonexistent user
    let response = app
        .client
        .get(format!("{}/profiles/nonexistentuser", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_profile_page_excludes_draft_articles() {
    // Arrange
    let fixture = TestFixture::new()
        .await
        .with_user_default("draftauthor")
        .await
        .promote_to_author("draftauthor")
        .await;

    let token = fixture.get_token("draftauthor");

    // Create a published article
    fixture
        .app
        .create_article_simple(&token, "Published Article")
        .await;

    // Create a draft article
    fixture
        .app
        .create_draft_article_simple(&token, "Draft Article")
        .await;

    // Act - Access profile page
    let response = fixture
        .app
        .client
        .get(format!("{}/profiles/draftauthor", &fixture.app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_status!(response, StatusCode::OK);
    let body = response.assert_html_response().await;
    assert!(body.contains("Published Article"));
    assert!(!body.contains("Draft Article")); // Draft should not appear in articles list
    assert!(body.contains("Articles (1)")); // Only 1 published article
}
