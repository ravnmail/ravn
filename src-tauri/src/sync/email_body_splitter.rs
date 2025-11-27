use once_cell::sync::Lazy;
use regex::Regex;
use scraper::{Html, Selector};

/// Result of splitting email body
#[derive(Debug, Clone)]
pub struct SplitEmailBody {
    /// Current email body (HTML)
    pub body_html: String,
    /// Quoted/forwarded emails (HTML), if any
    pub other_mails: Option<String>,
}

/// Email body splitter that separates current email from quoted/forwarded content
pub struct EmailBodySplitter;

// Compile regexes once at startup
static QUOTE_HEADER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(On|Am|Le|El|Il|Op|Em|Den)\s+.*?(wrote|schrieb|a écrit|escribió|ha scritto|schreef|napisał|skrev)(\s+[^:]+)?:")
        .unwrap()
});

static FORWARD_HEADER_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)^(From|Von|De|Da|Van|Od|Från|Fra):\s+").unwrap());

static SENT_HEADER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(Sent|Gesendet|Envoyé|Enviado|Inviato|Verzonden|Wysłano|Skickat|Sendt):\s+")
        .unwrap()
});

impl EmailBodySplitter {
    /// Split email body into current email and quoted/forwarded content
    /// Returns None for other_mails if no quoted content is detected
    /// Does not split forwarded emails
    pub fn split_body(html: Option<&str>) -> SplitEmailBody {
        let html = match html {
            Some(h) if !h.is_empty() => h,
            _ => {
                return SplitEmailBody {
                    body_html: String::new(),
                    other_mails: None,
                }
            }
        };

        if Self::is_forwarded_email(html) {
            return SplitEmailBody {
                body_html: html.to_string(),
                other_mails: None,
            };
        }

        let document = Html::parse_document(html);

        // Cache full HTML and text to avoid repeated DOM walks
        let full_html = document.html();
        let full_text = document.root_element().text().collect::<String>();

        if let Some(split) = Self::split_by_quote_classes(&document, &full_html) {
            return split;
        }

        if let Some(split) = Self::split_by_quote_patterns(&document, &full_text, &full_html) {
            return split;
        }

        if let Some(split) = Self::split_by_border_divs(&document, &full_html) {
            return split;
        }

        if let Some(split) = Self::split_by_blockquotes(&document, &full_html) {
            return split;
        }

        SplitEmailBody {
            body_html: html.to_string(),
            other_mails: None,
        }
    }

    /// Check if email is a forwarded message (contains forward indicators)
    fn is_forwarded_email(html: &str) -> bool {
        let html_lower = html.to_lowercase();
        html_lower.contains("---------- forwarded message")
            || html_lower.contains("----original message----")
            || html_lower.contains("begin forwarded message")
    }

    /// Split by Gmail/Outlook quote classes
    fn split_by_quote_classes(document: &Html, full_html: &str) -> Option<SplitEmailBody> {
        let gmail_selectors = [".gmail_quote", ".gmail_extra", "blockquote[type='cite']"];

        let outlook_selectors = [
            "[id^='divRplyFwdMsg']",
            ".OutlookMessageHeader",
            ".EmailQuote",
        ];

        let all_selectors: Vec<&str> = gmail_selectors
            .iter()
            .chain(outlook_selectors.iter())
            .copied()
            .collect();

        for selector_str in all_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                let quotes: Vec<_> = document.select(&selector).collect();
                if !quotes.is_empty() {
                    let mut clean_html = full_html.to_string();
                    let mut quoted_html = Vec::new();

                    for quote in quotes {
                        let quote_html = quote.html();
                        quoted_html.push(quote_html.clone());
                        clean_html = clean_html.replace(&quote_html, "");
                    }

                    if !quoted_html.is_empty() {
                        return Some(SplitEmailBody {
                            body_html: clean_html,
                            other_mails: Some(quoted_html.join("\n")),
                        });
                    }
                }
            }
        }

        None
    }

    /// Split by detecting quote header patterns in text
    fn split_by_quote_patterns(_document: &Html, html_text: &str, full_html: &str) -> Option<SplitEmailBody> {
        if QUOTE_HEADER_REGEX.is_match(html_text)
            || FORWARD_HEADER_REGEX.is_match(html_text)
            || SENT_HEADER_REGEX.is_match(html_text)
        {
            let lines: Vec<&str> = html_text.lines().collect();

            for (idx, line) in lines.iter().enumerate() {
                if QUOTE_HEADER_REGEX.is_match(line)
                    || (FORWARD_HEADER_REGEX.is_match(line)
                        && idx + 1 < lines.len()
                        && SENT_HEADER_REGEX.is_match(lines[idx + 1]))
                {
                    let quote_start_text = line;

                    if let Some(split_pos) = full_html.find(quote_start_text) {
                        let body_html = full_html[..split_pos].to_string();
                        let other_mails = full_html[split_pos..].to_string();

                        return Some(SplitEmailBody {
                            body_html,
                            other_mails: Some(other_mails),
                        });
                    }
                }
            }
        }

        None
    }

    /// Split by divs with border-top style (common Outlook pattern)
    fn split_by_border_divs(document: &Html, full_html: &str) -> Option<SplitEmailBody> {
        if let Ok(selector) = Selector::parse("div[style*='border-top']") {
            for div in document.select(&selector) {
                let text = div.text().collect::<String>();
                let text_trimmed = text.trim();

                if QUOTE_HEADER_REGEX.is_match(text_trimmed)
                    || FORWARD_HEADER_REGEX.is_match(text_trimmed)
                    || Self::is_quote_header_pattern(text_trimmed)
                {
                    let div_html = div.html();

                    if let Some(split_pos) = full_html.find(&div_html) {
                        let body_html = full_html[..split_pos].to_string();
                        let other_mails = full_html[split_pos..].to_string();

                        return Some(SplitEmailBody {
                            body_html,
                            other_mails: Some(other_mails),
                        });
                    }
                }
            }
        }

        None
    }

    /// Split by blockquote elements
    fn split_by_blockquotes(document: &Html, full_html: &str) -> Option<SplitEmailBody> {
        if let Ok(selector) = Selector::parse("blockquote") {
            let blockquotes: Vec<_> = document.select(&selector).collect();

            if !blockquotes.is_empty() {
                let substantial_quotes: Vec<_> = blockquotes
                    .iter()
                    .filter(|bq| {
                        let text = bq.text().collect::<String>();
                        text.trim().len() > 50
                    })
                    .collect();

                if !substantial_quotes.is_empty() {
                    let mut clean_html = full_html.to_string();
                    let mut quoted_parts = Vec::new();

                    for quote in substantial_quotes {
                        let quote_html = quote.html();
                        quoted_parts.push(format!("<blockquote>{}</blockquote>", quote_html));
                        clean_html = clean_html.replace(&quote_html, "");
                    }

                    return Some(SplitEmailBody {
                        body_html: clean_html,
                        other_mails: Some(quoted_parts.join("\n")),
                    });
                }
            }
        }

        None
    }

    /// Check if text matches quote header patterns (multi-line header)
    fn is_quote_header_pattern(text: &str) -> bool {
        let lines: Vec<&str> = text.lines().collect();

        let mut has_from = false;
        let mut has_sent = false;

        for line in lines {
            if FORWARD_HEADER_REGEX.is_match(line) {
                has_from = true;
            }
            if SENT_HEADER_REGEX.is_match(line) {
                has_sent = true;
            }
        }

        has_from && has_sent
    }

    /// Convert plain text body to HTML with quote detection
    pub fn split_plain_text(plain: Option<&str>) -> SplitEmailBody {
        let plain = match plain {
            Some(p) if !p.is_empty() => p,
            _ => {
                return SplitEmailBody {
                    body_html: String::new(),
                    other_mails: None,
                }
            }
        };

        let lines: Vec<&str> = plain.lines().collect();
        let mut body_lines = Vec::new();
        let mut quote_lines = Vec::new();
        let mut in_quote = false;

        for line in lines {
            if line.starts_with('>')
                || QUOTE_HEADER_REGEX.is_match(line)
                || FORWARD_HEADER_REGEX.is_match(line)
            {
                in_quote = true;
            }

            if in_quote {
                quote_lines.push(line);
            } else {
                body_lines.push(line);
            }
        }

        let body_html = body_lines.join("\n");
        let other_mails = if !quote_lines.is_empty() {
            Some(quote_lines.join("\n"))
        } else {
            None
        };

        SplitEmailBody {
            body_html,
            other_mails,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_gmail_quote() {
        let html = r#"
            <div>This is my reply</div>
            <div class="gmail_quote">
                <div>On Mon, Jan 1, 2024, John Doe wrote:</div>
                <blockquote>Original message content</blockquote>
            </div>
        "#;

        let result = EmailBodySplitter::split_body(Some(html));
        assert!(result.other_mails.is_some());
        assert!(result.body_html.contains("This is my reply"));
    }

    #[test]
    fn test_no_split_for_forwarded() {
        let html = r#"
            <div>---------- Forwarded message ---------</div>
            <div>From: someone@example.com</div>
            <div>Content here</div>
        "#;

        let result = EmailBodySplitter::split_body(Some(html));
        assert!(result.other_mails.is_none());
        assert!(result.body_html.contains("Forwarded message"));
    }

    #[test]
    fn test_split_blockquote() {
        let html = r#"
            <div>This is my reply to your email</div>
            <blockquote>
                This is a long quoted message that should be detected
                because it has more than 50 characters in it.
            </blockquote>
        "#;

        let result = EmailBodySplitter::split_body(Some(html));
        assert!(result.other_mails.is_some());
        assert!(result.body_html.contains("This is my reply"));
    }

    #[test]
    fn test_empty_input() {
        let result = EmailBodySplitter::split_body(None);
        assert!(result.body_html.is_empty());
        assert!(result.other_mails.is_none());
    }
}
