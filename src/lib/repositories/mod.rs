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
//! │  (LibSqlUserRepository, etc)             │
//! └─────────────────────────────────────────┘
//!                     │
//!                     ▼
//! ┌─────────────────────────────────────────┐
//! │          Database Layer                  │
//! │        (libsql / Turso)                  │
//! └─────────────────────────────────────────┘
//! ```
//!
//! # Example Usage
//!
//! ```ignore
//! use crate::repositories::{UserRepository, LibSqlUserRepository, NewUser};
//!
//! // Create repository
//! let user_repo = LibSqlUserRepository::new(db_connection);
//!
//! // Use repository methods
//! let user = user_repo.find_by_email("test@example.com").await?;
//! ```

// Module declarations
pub mod error;

// User repository
pub mod user;
pub mod user_libsql;

// Article repository
pub mod article;
pub mod article_libsql;

// Tag repository
pub mod tag;
pub mod tag_libsql;

// Category repository
pub mod category;
pub mod category_libsql;

// Comment repository
pub mod comment;
pub mod comment_libsql;

// Newsletter repository
pub mod newsletter;
pub mod newsletter_libsql;

// API Key repository
pub mod api_key;
pub mod api_key_libsql;

// Media repository
pub mod media;
pub mod media_libsql;

// Token repository
pub mod token;
pub mod token_libsql;

// Re-exports for convenience

// Error types
pub use error::{RepoResult, RepositoryError};

// User repository
pub use user::{NewUser, UpdateUser, UserRepository, UserSummary};
pub use user_libsql::LibSqlUserRepository;

// Article repository
pub use article::{
    ArticleRecord, ArticleRepository, ArticleWithDetails, NewArticle, UpdateArticleData,
};
pub use article_libsql::LibSqlArticleRepository;

// Tag repository
pub use tag::{TagRecord, TagRepository};
pub use tag_libsql::LibSqlTagRepository;

// Category repository
pub use category::{
    CategoryRecord, CategoryRepository, CategoryWithCount, NewCategory, UpdateCategoryData,
};
pub use category_libsql::LibSqlCategoryRepository;

// Comment repository
pub use comment::{CommentRecord, CommentRepository, CommentWithAuthor, NewComment};
pub use comment_libsql::LibSqlCommentRepository;

// Newsletter repository
pub use newsletter::{
    NewNewsletterIssue, NewSubscriber, NewsletterRepository, NewsletterStatsData,
    UpdateNewsletterIssueData,
};
pub use newsletter_libsql::LibSqlNewsletterRepository;

// API Key repository
pub use api_key::{ApiKeyRecord, ApiKeyRepository, NewApiKey};
pub use api_key_libsql::LibSqlApiKeyRepository;

// Media repository
pub use media::{MediaRepository, NewMedia, UpdateMediaData};
pub use media_libsql::LibSqlMediaRepository;

// Token repository
pub use token::{EmailVerificationToken, PasswordResetToken, TokenRepository};
pub use token_libsql::LibSqlTokenRepository;
