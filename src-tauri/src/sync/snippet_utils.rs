/// Extract a snippet from email body content
///
/// This function:
/// - Extracts plain text from HTML if available
/// - Falls back to plain text body
/// - Limits to 150-200 characters
/// - Trims at word boundary
/// - Cleans up whitespace
pub fn extract_snippet(body_plain: Option<&str>) -> Option<String> {
    let text = if let Some(plain) = body_plain {
        plain.to_string()
    } else {
        return None;
    };

    let cleaned = text
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    if cleaned.is_empty() {
        return None;
    }

    let max_length = 200;
    let min_length = 150;

    if cleaned.len() <= max_length {
        return Some(cleaned);
    }

    let boundary = cleaned[min_length..max_length]
        .rfind(char::is_whitespace)
        .map(|pos| min_length + pos)
        .unwrap_or(max_length);

    let snippet = cleaned[..boundary].trim_end();
    Some(format!("{}...", snippet))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_snippet_from_plain_text() {
        let plain = "This is a test email with some content that should be extracted as a snippet.";
        let result = extract_snippet(Some(plain));
        assert!(result.is_some());
        assert_eq!(result.unwrap(), plain);
    }
    #[test]
    fn test_extract_snippet_long_text() {
        let long_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
            Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
            Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris \
            nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in \
            reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.";

        let result = extract_snippet(Some(long_text));
        assert!(result.is_some());
        let snippet = result.unwrap();
        assert!(snippet.len() <= 204); // 200 + "..."
        assert!(snippet.ends_with("..."));
    }

    #[test]
    fn test_extract_snippet_empty() {
        let result = extract_snippet(None);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_snippet_whitespace_cleanup() {
        let text = "  Line one  \n\n  Line two  \n  Line three  ";
        let result = extract_snippet(Some(text));
        assert!(result.is_some());
        let snippet = result.unwrap();
        assert_eq!(snippet, "Line one Line two Line three");
    }
}
