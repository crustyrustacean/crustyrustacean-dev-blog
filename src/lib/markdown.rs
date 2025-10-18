// src/lib/markdown.rs

use pulldown_cmark::{Options, Parser, html};

/// Converts markdown text to HTML string
pub fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);

    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_markdown_conversion() {
        let markdown = "# Hello World\n\nThis is **bold** and *italic* text.";
        let html = markdown_to_html(markdown);

        assert!(html.contains("<h1>Hello World</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
    }

    #[test]
    fn test_code_blocks() {
        let markdown = "```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```";
        let html = markdown_to_html(markdown);

        assert!(html.contains("<pre><code class=\"language-rust\">"));
        assert!(html.contains("fn main()"));
        assert!(html.contains("println!"));
    }

    #[test]
    fn test_links_and_images() {
        let markdown = "[Link](https://example.com) and ![Image](https://example.com/image.png)";
        let html = markdown_to_html(markdown);

        assert!(html.contains("<a href=\"https://example.com\">Link</a>"));
        assert!(html.contains("<img src=\"https://example.com/image.png\" alt=\"Image\" />"));
    }

    #[test]
    fn test_lists() {
        let markdown = "- Item 1\n- Item 2\n- Item 3";
        let html = markdown_to_html(markdown);

        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>Item 1</li>"));
        assert!(html.contains("<li>Item 2</li>"));
        assert!(html.contains("<li>Item 3</li>"));
        assert!(html.contains("</ul>"));
    }

    #[test]
    fn test_tables() {
        let markdown = "| Header 1 | Header 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |";
        let html = markdown_to_html(markdown);

        assert!(html.contains("<table>"));
        assert!(html.contains("<th>Header 1</th>"));
        assert!(html.contains("<td>Cell 1</td>"));
    }

    #[test]
    fn test_strikethrough() {
        let markdown = "This is ~~strikethrough~~ text.";
        let html = markdown_to_html(markdown);

        assert!(html.contains("<del>strikethrough</del>"));
    }

    #[test]
    fn test_task_lists() {
        let markdown = "- [x] Completed task\n- [ ] Incomplete task";
        let html = markdown_to_html(markdown);

        assert!(html.contains("type=\"checkbox\""));
        assert!(html.contains("checked=\"\""));
    }

    #[test]
    fn test_plain_text_unchanged() {
        let markdown = "Just plain text with no markdown.";
        let html = markdown_to_html(markdown);

        assert!(html.contains("<p>Just plain text with no markdown.</p>"));
    }

    #[test]
    fn test_empty_string() {
        let html = markdown_to_html("");
        assert_eq!(html, "");
    }

    #[test]
    fn test_inline_code() {
        let markdown = "Here is some `inline code` in the text.";
        let html = markdown_to_html(markdown);

        assert!(html.contains("<code>inline code</code>"));
    }
}
