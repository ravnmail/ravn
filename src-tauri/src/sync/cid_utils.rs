use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

/// Regex to match CID references in HTML (src="cid:xxx", url(cid:xxx), etc.)
static CID_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:src|href|url)\s*=\s*["']?cid:([^"'\s>]+)["']?"#)
        .expect("Failed to compile CID regex")
});

/// Extract all Content-ID references from HTML body
/// Returns a set of content IDs (without the "cid:" prefix)
pub fn extract_cid_references(html_body: &str) -> HashSet<String> {
    CID_REGEX
        .captures_iter(html_body)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

/// Check if a content_id is referenced in the HTML body
pub fn is_cid_referenced(html_body: &str, content_id: &str) -> bool {
    let normalized_cid = content_id.trim_matches(|c| c == '<' || c == '>');

    extract_cid_references(html_body).contains(normalized_cid)
}

/// Replace CID references in HTML with actual asset paths
/// Maps content_id -> cache_path for replacement
pub fn replace_cid_references(
    html_body: &str,
    cid_to_path: &std::collections::HashMap<String, String>,
) -> String {
    let mut result = html_body.to_string();

    for (content_id, cache_path) in cid_to_path {
        let normalized_cid = content_id.trim_matches(|c| c == '<' || c == '>');

        let asset_url = format!("attachment://{}", cache_path);

        let patterns = [
            format!(r#"src="cid:{}""#, normalized_cid),
            format!(r#"src='cid:{}'"#, normalized_cid),
            format!(r#"src=cid:{}"#, normalized_cid),
            format!(r#"href="cid:{}""#, normalized_cid),
            format!(r#"href='cid:{}'"#, normalized_cid),
            format!(r#"url(cid:{})"#, normalized_cid),
        ];

        let replacements = [
            format!(r#"src="{}""#, asset_url),
            format!(r#"src='{}'"#, asset_url),
            format!(r#"src={}"#, asset_url),
            format!(r#"href="{}""#, asset_url),
            format!(r#"href='{}'"#, asset_url),
            format!(r#"url({})"#, asset_url),
        ];

        for (pattern, replacement) in patterns.iter().zip(replacements.iter()) {
            result = result.replace(pattern, replacement);
        }

        let patterns_upper = [
            format!(r#"SRC="cid:{}""#, normalized_cid),
            format!(r#"SRC='cid:{}'"#, normalized_cid),
            format!(r#"HREF="cid:{}""#, normalized_cid),
            format!(r#"HREF='cid:{}'"#, normalized_cid),
        ];

        let replacements_upper = [
            format!(r#"SRC="{}""#, asset_url),
            format!(r#"SRC='{}'"#, asset_url),
            format!(r#"HREF="{}""#, asset_url),
            format!(r#"HREF='{}'"#, asset_url),
        ];

        for (pattern, replacement) in patterns_upper.iter().zip(replacements_upper.iter()) {
            result = result.replace(pattern, replacement);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_cid_references() {
        let html = r#"
            <html>
                <img src="cid:image1">
                <img src='cid:image2'/>
                <div style="background: url(cid:bg-image)"></div>
                <a href="cid:link1">Link</a>
            </html>
        "#;

        let cids = extract_cid_references(html);
        assert_eq!(cids.len(), 4);
        assert!(cids.contains("image1"));
        assert!(cids.contains("image2"));
        assert!(cids.contains("bg-image"));
        assert!(cids.contains("link1"));
    }

    #[test]
    fn test_is_cid_referenced() {
        let html = r#"<img src="cid:test-image-123">"#;

        assert!(is_cid_referenced(html, "test-image-123"));
        assert!(is_cid_referenced(html, "<test-image-123>"));
        assert!(!is_cid_referenced(html, "other-image"));
    }

    #[test]
    fn test_replace_cid_references() {
        let html = r#"<html><img src="cid:img1"><img src='cid:img2'></html>"#;

        let mut cid_map = std::collections::HashMap::new();
        cid_map.insert("img1".to_string(), "account1/email1/image1.png".to_string());
        cid_map.insert("img2".to_string(), "account1/email1/image2.jpg".to_string());

        let result = replace_cid_references(html, &cid_map);

        assert!(result.contains("attachment://account1/email1/image1.png"));
        assert!(result.contains("attachment://account1/email1/image2.jpg"));
        assert!(!result.contains("cid:"));
    }

    #[test]
    fn test_replace_cid_with_angle_brackets() {
        let html = r#"<img src="cid:abc123">"#;

        let mut cid_map = std::collections::HashMap::new();
        cid_map.insert(
            "<abc123>".to_string(),
            "account1/email1/pic.png".to_string(),
        );

        let result = replace_cid_references(html, &cid_map);

        assert!(result.contains("attachment://account1/email1/pic.png"));
        assert!(!result.contains("cid:"));
    }

    #[test]
    fn test_case_insensitive_replacement() {
        let html = r#"<IMG SRC="cid:test">"#;

        let mut cid_map = std::collections::HashMap::new();
        cid_map.insert("test".to_string(), "path/to/image.png".to_string());

        let result = replace_cid_references(html, &cid_map);

        assert!(result.contains("attachment://path/to/image.png"));
    }
}
