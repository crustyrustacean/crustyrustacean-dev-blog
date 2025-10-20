// src/lib/routes/mod.rs

// modules
pub mod articles;
pub mod auth;
pub mod comments;
pub mod error_pages;
pub mod health_check;
pub mod index;
pub mod profile;
pub mod robots;
pub mod rss;
pub mod sitemap;
pub mod tags;
pub mod users;

// re-exports
pub use articles::*;
pub use auth::*;
pub use comments::*;
pub use error_pages::*;
pub use health_check::*;
pub use index::*;
pub use profile::*;
pub use robots::*;
pub use rss::*;
pub use sitemap::*;
pub use tags::*;
pub use users::*;
