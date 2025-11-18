// src/lib/routes/mod.rs

// modules
pub mod account;
pub mod api_keys;
pub mod articles;
pub mod auth;
pub mod categories;
pub mod comments;
pub mod error_pages;
pub mod health_check;
pub mod index;
pub mod media;
pub mod newsletters;
pub mod password_reset;
pub mod profile;
pub mod robots;
pub mod rss;
pub mod search;
pub mod sitemap;
pub mod tags;
pub mod users;

// re-exports
pub use account::*;
pub use api_keys::*;
pub use articles::*;
pub use auth::*;
pub use categories::*;
pub use comments::*;
pub use error_pages::*;
pub use health_check::*;
pub use index::*;
pub use media::*;
pub use newsletters::*;
pub use password_reset::*;
pub use profile::*;
pub use robots::*;
pub use rss::*;
pub use search::*;
pub use sitemap::*;
pub use tags::*;
pub use users::*;
