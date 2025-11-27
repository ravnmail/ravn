/// Email HTML rendering service
/// Converts TipTap HTML to email-compatible HTML with inline CSS
use css_inline::CSSInliner;

const EMAIL_CSS: &str = r#"
<style>
    /* Base email styles */
    body {
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
        line-height: 1.6;
        color: #1f2937;
        background-color: #ffffff;
        margin: 0;
        padding: 0;
    }

    .email-content {
        max-width: 768px;
        margin: 0 auto;
        padding: 16px 32px;
    }

    /* ProseMirror editor styles for emails */
    .ProseMirror {
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
        padding: 16px 0;
        color: #1f2937;
        outline: none;
    }

    .ProseMirror p {
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
        margin: 0.75rem 0;
    }

    .ProseMirror p:empty::after {
        content: '\00A0';
    }

    .ProseMirror > * {
        margin-left: 1rem;
    }

    .ProseMirror h1 {
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
        font-size: 1.5rem;
        font-weight: 700;
        margin: 1rem 0;
    }

    .ProseMirror h2 {
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
        font-size: 1.25rem;
        font-weight: 700;
        margin: 0.875rem 0;
    }

    .ProseMirror h3 {
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
        font-size: 1.125rem;
        font-weight: 700;
        margin: 0.75rem 0;
    }

    .ProseMirror ul {
        list-style-type: disc;
        list-style-position: inside;
        padding-left: 2rem;
        margin: 0.75rem 0;
    }

    .ProseMirror ol {
        list-style-type: decimal;
        list-style-position: inside;
        padding-left: 2rem;
        margin: 0.75rem 0;
    }

    .ProseMirror li {
        margin: 0.25rem 0;
    }

    .ProseMirror li p {
        display: inline;
        margin: 0;
    }

    .ProseMirror ul[data-type='taskList'] {
        list-style: none;
        padding: 0;
    }

    .ProseMirror ul[data-type='taskList'] li {
        display: flex;
        align-items: flex-start;
    }

    .ProseMirror ul[data-type='taskList'] li input[type='checkbox'] {
        margin-right: 0.5rem;
        margin-top: 0.25rem;
    }

    .ProseMirror ul[data-type='taskList'] li[data-checked='true'] {
        text-decoration: line-through;
        opacity: 0.7;
    }

    .ProseMirror code {
        background-color: #f9fafb;
        border: 1px solid #f3f4f6;
        border-radius: 0.25rem;
        padding: 0.125rem 0.25rem;
        font-size: 0.875rem;
        font-family: 'Courier New', Courier, monospace;
    }

    .ProseMirror pre {
        background-color: #f9fafb;
        border: 1px solid #f3f4f6;
        border-radius: 0.375rem;
        padding: 1rem;
        overflow-x: auto;
        margin: 1rem 0;
    }

    .ProseMirror pre code {
        background: none;
        border: none;
        padding: 0;
        display: block;
    }

    .ProseMirror blockquote {
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
        border-left: 4px solid #e5e7eb;
        padding-left: 1rem;
        margin: 1rem 0;
        color: #6b7280;
    }

    .ProseMirror mark {
        background-color: rgba(124, 179, 66, 0.3);
        padding: 0.125rem;
        border-radius: 0.125rem;
    }

    .ProseMirror a {
        color: #2563eb;
        text-decoration: underline;
        text-underline-offset: 2px;
    }

    .ProseMirror a:hover {
        color: #1d4ed8;
    }

    .ProseMirror hr {
        border: none;
        border-top: 2px solid #e5e7eb;
        margin: 1.5rem 0;
    }

    /* Indentation */
    .ProseMirror [data-indent='1'] { padding-left: 2em; }
    .ProseMirror [data-indent='2'] { padding-left: 4em; }
    .ProseMirror [data-indent='3'] { padding-left: 6em; }
    .ProseMirror [data-indent='4'] { padding-left: 8em; }
    .ProseMirror [data-indent='5'] { padding-left: 10em; }
    .ProseMirror [data-indent='6'] { padding-left: 12em; }
    .ProseMirror [data-indent='7'] { padding-left: 14em; }

    /* Callout support */
    .callout-wrapper {
        border-left: 4px solid #3b82f6;
        background-color: #eff6ff;
        padding: 1rem;
        margin: 1rem 0;
        border-radius: 0.375rem;
    }

    .callout-wrapper.callout-info {
        border-left-color: #3b82f6;
        background-color: #eff6ff;
    }

    .callout-wrapper.callout-warning {
        border-left-color: #f59e0b;
        background-color: #fffbeb;
    }

    .callout-wrapper.callout-error {
        border-left-color: #ef4444;
        background-color: #fef2f2;
    }

    .callout-wrapper.callout-success {
        border-left-color: #10b981;
        background-color: #f0fdf4;
    }
</style>
"#;

/// Wraps HTML content in a complete email structure with CSS and inlines styles
/// This improves compatibility with email clients, especially Outlook on Windows
pub fn render_email_html(body_html: &str) -> String {
    let html_with_styles = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    {}
</head>
<body>
    <div class="email-content">
        <div class="ProseMirror">
            {}
        </div>
    </div>
</body>
</html>"#,
        EMAIL_CSS, body_html
    );

    let inliner = CSSInliner::options()
        .minify_css(true)
        .load_remote_stylesheets(false)
        .keep_style_tags(false)
        .build();

    match inliner.inline(&html_with_styles) {
        Ok(inlined_html) => inlined_html,
        Err(e) => {
            log::warn!("Failed to inline CSS: {}. Using fallback HTML.", e);
            html_with_styles
        }
    }
}

/// Strips HTML tags for plain text version
pub fn html_to_plain_text(html: &str) -> String {
    let text = html
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("</p>", "\n\n")
        .replace("</div>", "\n")
        .replace("</li>", "\n")
        .replace("</h1>", "\n\n")
        .replace("</h2>", "\n\n")
        .replace("</h3>", "\n\n");

    let re = regex::Regex::new(r"<[^>]*>").unwrap();
    let text = re.replace_all(&text, "");

    let text = text
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");

    let re = regex::Regex::new(r"\n{3,}").unwrap();
    let text = re.replace_all(&text, "\n\n");

    text.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_email_html() {
        let body = "<p>Hello <strong>world</strong>!</p>";
        let result = render_email_html(body);

        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("Hello"));
        assert!(result.contains("world"));
        assert!(result.contains("style="));
    }

    #[test]
    fn test_html_to_plain_text() {
        let html = "<p>Hello <strong>world</strong>!</p><p>This is a test.</p>";
        let plain = html_to_plain_text(html);

        assert_eq!(plain, "Hello world!\n\nThis is a test.");
    }

    #[test]
    fn test_html_to_plain_text_with_entities() {
        let html = "<p>&lt;test&gt; &amp; &quot;quotes&quot;</p>";
        let plain = html_to_plain_text(html);

        assert_eq!(plain, "<test> & \"quotes\"");
    }
}
