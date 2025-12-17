# Testing Guide

This document describes the testing strategy and patterns for the `crustyrustacean-dev-blog` codebase.

## Test Structure

```
tests/
├── api/                    # Integration tests
│   ├── main.rs            # Test harness and helpers
│   ├── helpers.rs         # Test utilities and app spawning
│   ├── articles.rs        # Article endpoint tests
│   ├── users.rs           # User endpoint tests
│   ├── auth.rs            # Authentication tests
│   └── ...                # Other endpoint tests
└── shortcodes.rs          # Shortcode parsing tests

src/lib/
├── repositories/
│   ├── user.rs            # Unit tests for user types
│   ├── user_libsql.rs     # Unit tests for libsql implementation
│   └── ...                # Other repository tests
├── migrations/
│   └── mod.rs             # Migration system tests
├── email/
│   └── mod.rs             # Email service tests
└── ...                    # Other module tests
```

## Running Tests

### All Tests

```bash
cargo test
```

### Specific Test Module

```bash
# Run all article tests
cargo test articles

# Run a specific test
cargo test test_create_article_happy_path
```

### With Output

```bash
# Show println! output
cargo test -- --nocapture

# Show test names as they run
cargo test -- --show-output
```

## Test Types

### Unit Tests

Located inline with source code using `#[cfg(test)]` modules:

```rust
// src/lib/repositories/user.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_user_creation() {
        let new_user = NewUser {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash123".to_string(),
            role: Role::Subscriber,
        };

        assert_eq!(new_user.username, "testuser");
        assert_eq!(new_user.role, Role::Subscriber);
    }

    #[test]
    fn test_update_user_default() {
        let update = UpdateUser::default();
        assert!(update.username.is_none());
        assert!(update.email.is_none());
    }
}
```

### Integration Tests

Located in `tests/api/` and test the full HTTP stack with a real in-memory database:

```rust
// tests/api/articles.rs

#[tokio::test]
async fn test_create_article_happy_path() {
    let app = spawn_app().await;

    // Register and authenticate
    let token = app.register_and_login("author", "author@example.com").await;

    // Create article
    let response = app.post_article(
        &token,
        json!({
            "title": "Test Article",
            "description": "A test",
            "body": "Article content"
        }),
    ).await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let body: Value = response.json().await;
    assert_eq!(body["title"], "Test Article");
}
```

### Doc Tests

Documentation examples that are tested:

```rust
/// Parses shortcodes from markdown content.
///
/// # Examples
///
/// ```
/// use crustyrustacean_dev_blog_lib::shortcodes::parse_shortcodes;
///
/// let text = "Check out [[my-article]]!";
/// let shortcodes = parse_shortcodes(text);
/// assert_eq!(shortcodes.len(), 1);
/// ```
pub fn parse_shortcodes(content: &str) -> Vec<Shortcode> {
    // ...
}
```

## Test Helpers

### TestApp Structure

The `spawn_app()` function creates a test application:

```rust
pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db: DatabaseConnection,
    pub client: reqwest::Client,
}

impl TestApp {
    pub async fn get(&self, path: &str) -> Response {
        self.client
            .get(&format!("{}{}", self.address, path))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn post_json<T: Serialize>(&self, path: &str, body: &T) -> Response {
        self.client
            .post(&format!("{}{}", self.address, path))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn register_and_login(&self, username: &str, email: &str) -> String {
        // Register user
        self.post_json("/api/users/register", &json!({
            "username": username,
            "email": email,
            "password": "password123"
        })).await;

        // Login and get token
        let response = self.post_json("/api/users/login", &json!({
            "email": email,
            "password": "password123"
        })).await;

        let body: Value = response.json().await;
        body["token"].as_str().unwrap().to_string()
    }
}
```

### Authenticated Requests

```rust
pub async fn post_with_auth<T: Serialize>(
    &self,
    path: &str,
    token: &str,
    body: &T,
) -> Response {
    self.client
        .post(&format!("{}{}", self.address, path))
        .header("Authorization", format!("Bearer {}", token))
        .json(body)
        .send()
        .await
        .expect("Failed to execute request")
}
```

## Test Patterns

### Happy Path Tests

Test the expected successful behavior:

```rust
#[tokio::test]
async fn test_create_article_happy_path() {
    let app = spawn_app().await;
    let token = app.register_and_login("author", "author@example.com").await;

    let response = app.post_with_auth(
        "/api/articles",
        &token,
        &json!({
            "title": "My Article",
            "description": "Description",
            "body": "Content"
        }),
    ).await;

    assert_eq!(response.status(), StatusCode::CREATED);
}
```

### Error Cases

Test validation and error handling:

```rust
#[tokio::test]
async fn test_create_article_without_auth() {
    let app = spawn_app().await;

    let response = app.post_json("/api/articles", &json!({
        "title": "Test",
        "description": "Test",
        "body": "Test"
    })).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_article_with_invalid_data() {
    let app = spawn_app().await;
    let token = app.register_and_login("author", "author@example.com").await;

    // Missing required field
    let response = app.post_with_auth(
        "/api/articles",
        &token,
        &json!({ "title": "Only title" }),
    ).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
```

### Authorization Tests

Test role-based access control:

```rust
#[tokio::test]
async fn test_subscriber_cannot_create_articles() {
    let app = spawn_app().await;

    // First user is admin, create a second user (subscriber)
    app.register_and_login("admin", "admin@example.com").await;
    let subscriber_token = app.register_and_login("subscriber", "sub@example.com").await;

    let response = app.post_with_auth(
        "/api/articles",
        &subscriber_token,
        &json!({
            "title": "Test",
            "description": "Test",
            "body": "Test"
        }),
    ).await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
```

### Idempotency Tests

Test that operations can be repeated safely:

```rust
#[tokio::test]
async fn test_favorite_already_favorited_article() {
    let app = spawn_app().await;
    let token = app.register_and_login("user", "user@example.com").await;

    // Create article
    let article = app.create_article(&token, "Test").await;

    // Favorite twice
    let first = app.post_with_auth(
        &format!("/api/articles/{}/favorite", article.slug),
        &token,
        &json!({}),
    ).await;
    assert_eq!(first.status(), StatusCode::OK);

    let second = app.post_with_auth(
        &format!("/api/articles/{}/favorite", article.slug),
        &token,
        &json!({}),
    ).await;
    assert_eq!(second.status(), StatusCode::OK);
}
```

## Repository Testing

### Unit Tests for Types

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_article_creation() {
        let article = NewArticle {
            title: "Test".to_string(),
            description: "Desc".to_string(),
            body: "Body".to_string(),
            author_id: Uuid::new_v4(),
            category_id: None,
            draft: false,
            tags: vec!["rust".to_string()],
        };

        assert_eq!(article.title, "Test");
        assert!(!article.draft);
    }

    #[test]
    fn test_update_article_default() {
        let update = UpdateArticle::default();
        assert!(update.title.is_none());
        assert!(update.body.is_none());
        assert!(update.draft.is_none());
    }
}
```

### Integration Tests via API

Repository implementations are tested through API integration tests:

```rust
#[tokio::test]
async fn test_update_article_happy_path() {
    let app = spawn_app().await;
    let token = app.register_and_login("author", "author@example.com").await;

    // Create
    let article = app.create_article(&token, "Original Title").await;

    // Update via API (tests repository.update internally)
    let response = app.put_with_auth(
        &format!("/api/articles/{}", article.slug),
        &token,
        &json!({ "title": "Updated Title" }),
    ).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await;
    assert_eq!(body["title"], "Updated Title");
}
```

## Migration Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_are_sequential() {
        let versions: Vec<i64> = MIGRATIONS.iter().map(|m| m.version).collect();
        for (i, version) in versions.iter().enumerate() {
            assert_eq!(*version, (i + 1) as i64);
        }
    }

    #[test]
    fn test_all_migrations_have_names() {
        for migration in MIGRATIONS {
            assert!(!migration.name.is_empty());
        }
    }
}
```

## Mocking External Services

### Email Service

The email service supports a mock sender for testing:

```rust
// Use MockSender in tests
let email_service = EmailService::new(
    MockSender::new(),
    "noreply@test.com".to_string(),
    "http://localhost".to_string(),
);

// Verify emails were "sent"
let sent_emails = email_service.get_sent_emails();
assert_eq!(sent_emails.len(), 1);
```

### Storage Backend

```rust
// Tests use in-memory storage
let storage = InMemoryStorage::new();
```

## Test Coverage

Current test coverage includes:

- **86** unit tests (lib crate)
- **341** integration tests (API endpoints)
- **13** shortcode tests
- **3** doc tests

### Running Coverage Report

```bash
# Install coverage tool
cargo install cargo-tarpaulin

# Generate report
cargo tarpaulin --out Html
```

## Best Practices

1. **One assertion per concept**: Each test should verify one logical behavior
2. **Descriptive names**: Use `test_<action>_<condition>_<result>` naming
3. **Test isolation**: Each test should be independent and not rely on others
4. **Clean up**: Tests using shared resources should clean up after themselves
5. **Fast tests**: Avoid unnecessary delays; use in-memory databases
6. **Test edge cases**: Empty inputs, boundary values, error conditions
