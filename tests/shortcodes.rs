// Unit tests for shortcode parsing and processing
// These are unit tests rather than integration tests since we're testing
// the shortcode parser logic in isolation

use crustyrustacean_dev_blog_lib::shortcodes::parse_shortcodes;

#[test]
fn test_parse_simple_article_shortcode() {
    let text = "Check out [[article:rust-intro]] for more info";
    let shortcodes = parse_shortcodes(text);

    assert_eq!(shortcodes.len(), 1);
    assert_eq!(shortcodes[0].slug, "rust-intro");
    assert_eq!(shortcodes[0].custom_text, None);
}

#[test]
fn test_parse_article_shortcode_with_custom_text() {
    let text = "Read [[article:getting-started|this guide]] first";
    let shortcodes = parse_shortcodes(text);

    assert_eq!(shortcodes.len(), 1);
    assert_eq!(shortcodes[0].slug, "getting-started");
    assert_eq!(shortcodes[0].custom_text, Some("this guide".to_string()));
}

#[test]
fn test_parse_multiple_shortcodes() {
    let text = "See [[article:intro]] and [[article:advanced|advanced topics]] for details";
    let shortcodes = parse_shortcodes(text);

    assert_eq!(shortcodes.len(), 2);
    assert_eq!(shortcodes[0].slug, "intro");
    assert_eq!(shortcodes[0].custom_text, None);
    assert_eq!(shortcodes[1].slug, "advanced");
    assert_eq!(shortcodes[1].custom_text, Some("advanced topics".to_string()));
}

#[test]
fn test_parse_no_shortcodes() {
    let text = "This is plain text with no shortcodes";
    let shortcodes = parse_shortcodes(text);

    assert_eq!(shortcodes.len(), 0);
}

#[test]
fn test_parse_ignores_malformed_shortcodes() {
    let text = "Bad: [[article:]] and [[article]] and [[notatype:slug]]";
    let shortcodes = parse_shortcodes(text);

    assert_eq!(shortcodes.len(), 0);
}

#[test]
fn test_parse_handles_special_characters_in_slug() {
    // Slugs can contain hyphens, underscores, numbers
    let text = "[[article:rust-2024_edition-part-1]]";
    let shortcodes = parse_shortcodes(text);

    assert_eq!(shortcodes.len(), 1);
    assert_eq!(shortcodes[0].slug, "rust-2024_edition-part-1");
}

#[test]
fn test_parse_handles_spaces_in_custom_text() {
    let text = "[[article:slug|Check out this amazing article]]";
    let shortcodes = parse_shortcodes(text);

    assert_eq!(shortcodes.len(), 1);
    assert_eq!(shortcodes[0].custom_text, Some("Check out this amazing article".to_string()));
}

#[test]
fn test_parse_shortcode_at_start_of_text() {
    let text = "[[article:first-post]] is great";
    let shortcodes = parse_shortcodes(text);

    assert_eq!(shortcodes.len(), 1);
    assert_eq!(shortcodes[0].slug, "first-post");
}

#[test]
fn test_parse_shortcode_at_end_of_text() {
    let text = "Check out [[article:last-post]]";
    let shortcodes = parse_shortcodes(text);

    assert_eq!(shortcodes.len(), 1);
    assert_eq!(shortcodes[0].slug, "last-post");
}

#[test]
fn test_parse_consecutive_shortcodes() {
    let text = "[[article:first]][[article:second]][[article:third]]";
    let shortcodes = parse_shortcodes(text);

    assert_eq!(shortcodes.len(), 3);
    assert_eq!(shortcodes[0].slug, "first");
    assert_eq!(shortcodes[1].slug, "second");
    assert_eq!(shortcodes[2].slug, "third");
}

#[test]
fn test_parse_ignores_incomplete_brackets() {
    let text = "Not a shortcode: [article:slug] or [[article:slug or article:slug]]";
    let shortcodes = parse_shortcodes(text);

    assert_eq!(shortcodes.len(), 0);
}

#[test]
fn test_parse_handles_pipe_without_text() {
    // [[article:slug|]] should be treated as malformed
    let text = "Bad shortcode: [[article:slug|]]";
    let shortcodes = parse_shortcodes(text);

    assert_eq!(shortcodes.len(), 0);
}

#[test]
fn test_parse_preserves_order() {
    let text = "[[article:third]] appears after [[article:first]] and [[article:second]]";
    let shortcodes = parse_shortcodes(text);

    assert_eq!(shortcodes.len(), 3);
    assert_eq!(shortcodes[0].slug, "third");
    assert_eq!(shortcodes[1].slug, "first");
    assert_eq!(shortcodes[2].slug, "second");
}
