use serde_json::Value;

/// Email category types based on content analysis
#[derive(Debug, Clone, PartialEq)]
pub enum EmailCategory {
    Personal,
    Transactions,
    Updates,
    Promotions,
}

impl EmailCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmailCategory::Personal => "personal",
            EmailCategory::Transactions => "transactions",
            EmailCategory::Updates => "updates",
            EmailCategory::Promotions => "promotions",
        }
    }
}

/// Categorizes an email based on headers, subject, and body content
pub struct EmailCategorizer;

impl EmailCategorizer {
    /// Categorize email based on headers, subject, and body
    pub fn categorize(
        headers: Option<&Value>,
        subject: Option<&str>,
        body_plain: Option<&str>,
        body_html: Option<&str>,
        from_address: &str,
    ) -> Option<EmailCategory> {
        if let Some(category) = Self::categorize_by_headers(headers) {
            return Some(category);
        }

        if let Some(category) =
            Self::categorize_by_content(subject, body_plain, body_html, from_address)
        {
            return Some(category);
        }

        Some(EmailCategory::Personal)
    }

    /// Categorize based on email headers (most reliable method)
    fn categorize_by_headers(headers: Option<&Value>) -> Option<EmailCategory> {
        let headers = headers?;

        if let Some(x_mailer) = headers.get("x-mailer").or_else(|| headers.get("X-Mailer")) {
            if let Some(mailer_str) = x_mailer.as_str() {
                let mailer_lower = mailer_str.to_lowercase();
                if mailer_lower.contains("mailchimp")
                    || mailer_lower.contains("sendgrid")
                    || mailer_lower.contains("mailgun")
                    || mailer_lower.contains("mandrill")
                    || mailer_lower.contains("campaignmonitor")
                    || mailer_lower.contains("constant contact")
                {
                    return Some(EmailCategory::Promotions);
                }
            }
        }

        if headers.get("list-unsubscribe").is_some() || headers.get("List-Unsubscribe").is_some() {
            if let Some(precedence) = headers
                .get("precedence")
                .or_else(|| headers.get("Precedence"))
            {
                if let Some(prec_str) = precedence.as_str() {
                    if prec_str.to_lowercase() == "bulk" {
                        return Some(EmailCategory::Promotions);
                    }
                }
            }
            return Some(EmailCategory::Updates);
        }

        if let Some(content_type) = headers
            .get("content-type")
            .or_else(|| headers.get("Content-Type"))
        {
            if let Some(ct_str) = content_type.as_str() {
                if ct_str.contains("receipt") || ct_str.contains("invoice") {
                    return Some(EmailCategory::Transactions);
                }
            }
        }

        if let Some(auto_submitted) = headers
            .get("auto-submitted")
            .or_else(|| headers.get("Auto-Submitted"))
        {
            if let Some(auto_str) = auto_submitted.as_str() {
                let auto_lower = auto_str.to_lowercase();
                if auto_lower != "no" {
                    return Some(EmailCategory::Updates);
                }
            }
        }

        None
    }

    /// Categorize based on subject, body content, and sender
    fn categorize_by_content(
        subject: Option<&str>,
        body_plain: Option<&str>,
        body_html: Option<&str>,
        from_address: &str,
    ) -> Option<EmailCategory> {
        let subject_lower = subject.map(|s| s.to_lowercase()).unwrap_or_default();
        let body_text = body_plain
            .or(body_html)
            .map(|b| b.to_lowercase())
            .unwrap_or_default();
        let from_lower = from_address.to_lowercase();

        let transaction_keywords = [
            "receipt",
            "invoice",
            "payment",
            "order",
            "confirmation",
            "shipped",
            "delivery",
            "tracking",
            "purchase",
            "transaction",
            "refund",
            "billing",
            "statement",
            "paid",
            "subscription",
            "renew",
            "expire",
            "due",
        ];

        if Self::contains_keywords(&subject_lower, &transaction_keywords)
            || Self::contains_keywords(&body_text, &transaction_keywords)
            || from_lower.contains("noreply")
            || from_lower.contains("no-reply")
            || from_lower.contains("receipt")
            || from_lower.contains("billing")
        {
            if subject_lower.contains("receipt")
                || subject_lower.contains("invoice")
                || subject_lower.contains("payment")
                || subject_lower.contains("order")
                || body_text.contains("total:")
                || body_text.contains("amount:")
                || body_text.contains("$")
                || body_text.contains("€")
                || body_text.contains("£")
            {
                return Some(EmailCategory::Transactions);
            }
        }

        let promotional_keywords = [
            "sale",
            "discount",
            "offer",
            "deal",
            "promo",
            "coupon",
            "limited time",
            "shop now",
            "buy now",
            "% off",
            "free shipping",
            "subscribe",
            "unsubscribe",
            "special offer",
            "exclusive",
            "clearance",
            "save",
            "today only",
        ];

        if Self::contains_keywords(&subject_lower, &promotional_keywords)
            || Self::contains_keywords(&body_text, &promotional_keywords)
        {
            return Some(EmailCategory::Promotions);
        }

        let update_keywords = [
            "newsletter",
            "update",
            "digest",
            "weekly",
            "monthly",
            "notification",
            "alert",
            "reminder",
            "summary",
            "report",
            "bulletin",
            "news",
            "announcement",
            "release notes",
            "changelog",
        ];

        if Self::contains_keywords(&subject_lower, &update_keywords)
            || Self::contains_keywords(&body_text, &update_keywords)
        {
            return Some(EmailCategory::Updates);
        }

        if from_lower.contains("notification")
            || from_lower.contains("alert")
            || from_lower.contains("update")
            || from_lower.contains("news")
        {
            return Some(EmailCategory::Updates);
        }

        None
    }

    fn contains_keywords(text: &str, keywords: &[&str]) -> bool {
        keywords.iter().any(|keyword| text.contains(keyword))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_categorize_promotional_by_header() {
        let headers = json!({
            "X-Mailer": "MailChimp Mailer",
            "List-Unsubscribe": "<mailto:unsubscribe@example.com>"
        });

        let category = EmailCategorizer::categorize(
            Some(&headers),
            Some("Special Offer Inside!"),
            None,
            None,
            "marketing@example.com",
        );

        assert_eq!(category, Some(EmailCategory::Promotions));
    }

    #[test]
    fn test_categorize_transaction_by_subject() {
        let category = EmailCategorizer::categorize(
            None,
            Some("Your receipt from Amazon"),
            Some("Order total: $49.99"),
            None,
            "no-reply@amazon.com",
        );

        assert_eq!(category, Some(EmailCategory::Transactions));
    }

    #[test]
    fn test_categorize_updates_by_keywords() {
        let category = EmailCategorizer::categorize(
            None,
            Some("Weekly Newsletter - Tech Updates"),
            Some("Here's your weekly digest of tech news..."),
            None,
            "newsletter@techsite.com",
        );

        assert_eq!(category, Some(EmailCategory::Updates));
    }

    #[test]
    fn test_categorize_personal_default() {
        let category = EmailCategorizer::categorize(
            None,
            Some("Hey, how are you?"),
            Some("Just wanted to catch up..."),
            None,
            "friend@example.com",
        );

        assert_eq!(category, Some(EmailCategory::Personal));
    }

    #[test]
    fn test_categorize_promotional_by_subject() {
        let category = EmailCategorizer::categorize(
            None,
            Some("50% OFF Everything - Today Only!"),
            Some("Shop now and save big!"),
            None,
            "sales@store.com",
        );

        assert_eq!(category, Some(EmailCategory::Promotions));
    }
}
