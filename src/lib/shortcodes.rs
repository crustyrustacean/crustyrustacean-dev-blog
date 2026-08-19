// Shortcode parsing and processing for internal article links
//
// Syntax: [[article:slug]] or [[article:slug|Custom Text]]
//
// Processing happens at article submission time:
// 1. Parse shortcodes from article body
// 2. Query database for article titles
// 3. Replace with markdown links
// 4. Store processed markdown in database

use regex::Regex;
use std::sync::LazyLock;

use sqlx::PgPool;
use crate::errors::ApiError;

/// Represents a parsed shortcode
#[derive(Debug, Clone, PartialEq)]
pub struct Shortcode {
    pub slug: String,
    pub custom_text: Option<String>,
}

// Regex to match [[article:slug]] or [[article:slug|custom text]]
// Pattern explanation:
// \[\[           - literal [[
// article:       - literal "article:"
// ([a-zA-Z0-9_-]+) - slug (alphanumeric, hyphens, underscores)
// (?:\|([^\]]+))? - optional group: pipe followed by custom text (anything except ])
// \]\]           - literal ]]
static SHORTCODE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[\[article:([a-zA-Z0-9_-]+)(?:\|([^\]]+))?\]\]")
        .expect("Failed to compile shortcode regex")
});

/// Parse all shortcodes from the given text
///
/// # Examples
///
/// ```
/// use crustyrustacean_dev_blog_lib::shortcodes::parse_shortcodes;
///
/// let text = "Check out [[article:rust-intro]] for more";
/// let shortcodes = parse_shortcodes(text);
/// assert_eq!(shortcodes.len(), 1);
/// assert_eq!(shortcodes[0].slug, "rust-intro");
/// ```
pub fn parse_shortcodes(text: &str) -> Vec<Shortcode> {
    SHORTCODE_REGEX
        .captures_iter(text)
        .filter_map(|cap| {
            let slug = cap.get(1)?.as_str().to_string();

            // If slug is empty after extraction, skip this shortcode
            if slug.is_empty() {
                return None;
            }

            // Get optional custom text from capture group 2
            let custom_text = cap.get(2).and_then(|m| {
                let text = m.as_str().trim();
                if text.is_empty() {
                    None
                } else {
                    Some(text.to_string())
                }
            });

            Some(Shortcode { slug, custom_text })
        })
        .collect()
}

/// Resolve a single shortcode by querying the database for the article
///
/// Returns a markdown link:
/// - If article found: `[Article Title](/articles/slug)`
/// - If article not found: `[slug](/articles/slug "Article not found")`
/// - If custom text provided: `[Custom Text](/articles/slug)`
async fn resolve_shortcode(
    shortcode: &Shortcode,
    db: &PgPool,
) -> Result<String, ApiError> {
    // Query database for article with this slug
    let article_title: Option<String> =
        sqlx::query_scalar("SELECT title FROM articles WHERE slug = $1")
            .bind(&shortcode.slug)
            .fetch_optional(db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to query article for shortcode: {}", e);
                ApiError::InternalServerError(e.to_string())
            })?;

    match article_title {
        Some(title) => {
            let link_text = shortcode.custom_text.as_deref().unwrap_or(&title);
            Ok(format!("[{}](/articles/{})", link_text, shortcode.slug))
        }
        None => {
            // Article not found - use slug as fallback text and add tooltip
            let link_text = shortcode.custom_text.as_deref().unwrap_or(&shortcode.slug);
            Ok(format!(
                r#"[{}](/articles/{} \"Article not found\")"#,
                link_text, shortcode.slug
            ))
        }
    }
}

/// Process all shortcodes in the given text
///
/// This is the main entry point for shortcode processing.
/// It finds all shortcodes, resolves them against the database,
/// and replaces them with markdown links.
///
/// # Examples
///
/// ```ignore
/// // This example requires a database connection
/// let text = "Check out [[article:rust-intro]] for more";
/// let processed = process_shortcodes(text, &db).await?;
/// // If the article exists: "Check out [Introduction to Rust](/articles/rust-intro) for more"
/// // If not: "Check out [rust-intro](/articles/rust-intro \"Article not found\") for more"
/// ```
pub async fn process_shortcodes(text: &str, db: &PgPool) -> Result<String, ApiError> {
    let shortcodes = parse_shortcodes(text);

    // If no shortcodes found, return original text
    if shortcodes.is_empty() {
        return Ok(text.to_string());
    }

    // Build a new string with shortcodes replaced
    let mut result = text.to_string();

    // Process shortcodes in reverse order to maintain correct string positions
    // when replacing (replacing from end to start keeps earlier positions valid)
    let mut replacements: Vec<(String, String)> = Vec::new();

    for shortcode in &shortcodes {
        let original = if let Some(custom_text) = &shortcode.custom_text {
            format!("[[article:{}|{}]]", shortcode.slug, custom_text)
        } else {
            format!("[[article:{}]]", shortcode.slug)
        };

        let replacement = resolve_shortcode(shortcode, db).await?;
        replacements.push((original, replacement));
    }

    // Apply all replacements
    for (original, replacement) in replacements {
        result = result.replace(&original, &replacement);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcode_regex_simple() {
        let text = "[[article:test-slug]]";
        let captures: Vec<_> = SHORTCODE_REGEX.captures_iter(text).collect();
        assert_eq!(captures.len(), 1);
        assert_eq!(captures[0].get(1).unwrap().as_str(), "test-slug");
        assert!(captures[0].get(2).is_none());
    }

    #[test]
    fn test_shortcode_regex_with_custom_text() {
        let text = "[[article:test-slug|Custom Text]]";
        let captures: Vec<_> = SHORTCODE_REGEX.captures_iter(text).collect();
        assert_eq!(captures.len(), 1);
        assert_eq!(captures[0].get(1).unwrap().as_str(), "test-slug");
        assert_eq!(captures[0].get(2).unwrap().as_str(), "Custom Text");
    }

    #[test]
    fn test_shortcode_regex_ignores_single_brackets() {
        let text = "[article:test-slug]";
        let captures: Vec<_> = SHORTCODE_REGEX.captures_iter(text).collect();
        assert_eq!(captures.len(), 0);
    }
}
