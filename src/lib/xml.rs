// src/lib/xml.rs

/// Escape XML special characters in a string.
///
/// Replaces the following characters with their XML entity equivalents:
/// - `&` → `&amp;`
/// - `<` → `&lt;`
/// - `>` → `&gt;`
/// - `"` → `&quot;`
/// - `'` → `&apos;`
pub fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_xml_ampersand() {
        assert_eq!(escape_xml("Hello & goodbye"), "Hello &amp; goodbye");
    }

    #[test]
    fn test_escape_xml_angle_brackets() {
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
    }

    #[test]
    fn test_escape_xml_quotes() {
        assert_eq!(escape_xml(r#"It's "quoted""#), "It&apos;s &quot;quoted&quot;");
    }

    #[test]
    fn test_escape_xml_all_special_chars() {
        assert_eq!(
            escape_xml("It's \"quoted\" & <escaped>"),
            "It&apos;s &quot;quoted&quot; &amp; &lt;escaped&gt;"
        );
    }

    #[test]
    fn test_escape_xml_no_special_chars() {
        assert_eq!(escape_xml("Hello World"), "Hello World");
    }

    #[test]
    fn test_escape_xml_empty_string() {
        assert_eq!(escape_xml(""), "");
    }
}
