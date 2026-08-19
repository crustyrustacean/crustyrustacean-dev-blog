// src/lib/repositories/mod.rs

//! Repository pattern implementation for database access.
//!
//! This module provides trait-based abstractions for data access operations,
//! enabling separation of concerns between business logic and data storage.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │          Route Handlers                  │
//! └─────────────────────────────────────────┘
//!                     │
//!                     ▼
//! ┌─────────────────────────────────────────┐
//! │       Repository Traits                  │
//! │  (UserRepository, ArticleRepository, etc)│
//! └─────────────────────────────────────────┘
//!                     │
//!                     ▼
//! ┌─────────────────────────────────────────┐
//! │    Concrete Implementations              │
//! Concrete Implementations, e.g. `PgUserRepository`              │
//! └─────────────────────────────────────────┘
//!                     │
//!                     ▼
//! ┌─────────────────────────────────────────┐
//! │          Database Layer                  │
//!        (PostgreSQL via sqlx)                                │
//! └─────────────────────────────────────────┘
//! ```
//!
//! # Example Usage
//!
//! ```ignore
//! use crate::repositories::{UserRepository, PgUserRepository, NewUser};
//!
//! // Create repository
//! let user_repo = PgUserRepository::new(pool);
//!
//! // Use repository methods
//! let user = user_repo.find_by_email("test@example.com").await?;
//! ```

// Module declarations
pub mod error;

// User repository
pub mod user;
pub mod user_pg;

// Article repository
pub mod article;
pub mod article_pg;

// Tag repository
pub mod tag;
pub mod tag_pg;

// Category repository
pub mod category;
pub mod category_pg;

// Comment repository
pub mod comment;
pub mod comment_pg;

// Newsletter repository
pub mod newsletter;
pub mod newsletter_pg;

// API Key repository
pub mod api_key;
pub mod api_key_pg;

// Media repository
pub mod media;
pub mod media_pg;

// Token repository
pub mod token;
pub mod token_pg;

// Re-exports for convenience

// Error types
pub use error::{RepoResult, RepositoryError};

// User repository
pub use user::{NewUser, UpdateUser, UserRepository, UserSummary};
pub use user_pg::PgUserRepository;

// Article repository
pub use article::{
    ArticleRecord, ArticleRepository, ArticleWithDetails, NewArticle, UpdateArticleData,
};
pub use article_pg::PgArticleRepository;

// Tag repository
pub use tag::{TagRecord, TagRepository};
pub use tag_pg::PgTagRepository;

// Category repository
pub use category::{
    CategoryRecord, CategoryRepository, CategoryWithCount, NewCategory, UpdateCategoryData,
};
pub use category_pg::PgCategoryRepository;

// Comment repository
pub use comment::{CommentRecord, CommentRepository, CommentWithAuthor, NewComment};
pub use comment_pg::PgCommentRepository;

// Newsletter repository
pub use newsletter::{
    NewNewsletterIssue, NewSubscriber, NewsletterRepository, NewsletterStatsData,
    UpdateNewsletterIssueData,
};
pub use newsletter_pg::PgNewsletterRepository;

// API Key repository
pub use api_key::{ApiKeyRecord, ApiKeyRepository, NewApiKey};
pub use api_key_pg::PgApiKeyRepository;

// Media repository
pub use media::{MediaRepository, NewMedia, UpdateMediaData};
pub use media_pg::PgMediaRepository;

// Token repository
pub use token::{EmailVerificationToken, PasswordResetToken, TokenRepository};
pub use token_pg::PgTokenRepository;
