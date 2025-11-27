use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationUrl {
    pub scheme: String,
    pub path: String,
    pub query: Option<String>,
}

impl NavigationUrl {
    /// Parse a RAVN URL (ravn://path?query)
    pub fn parse(url: &str) -> Result<Self, String> {
        if !url.starts_with("ravn://") {
            return Err(format!("Invalid RAVN URL scheme: {}", url));
        }

        let without_scheme = &url[7..]; // Remove "ravn://"

        let (path, query) = if let Some(query_start) = without_scheme.find('?') {
            let path = &without_scheme[..query_start];
            let query = Some(without_scheme[query_start + 1..].to_string());
            (path, query)
        } else {
            (without_scheme, None)
        };

        Ok(NavigationUrl {
            scheme: "ravn".to_string(),
            path: path.to_string(),
            query,
        })
    }

    /// Convert RAVN URL to Vue Router path
    pub fn to_router_path(&self) -> String {
        let mapped_path = match self.path.as_str() {
            _ => &self.path,
        };

        let base_path = if mapped_path.is_empty() {
            "/".to_string()
        } else if !mapped_path.starts_with('/') {
            format!("/{}", mapped_path)
        } else {
            mapped_path.to_string()
        };

        if let Some(query) = &self.query {
            format!("{}?{}", base_path, query)
        } else {
            base_path
        }
    }

    /// Build a RAVN URL from components
    pub fn build(path: &str, query: Option<&str>) -> String {
        let clean_path = path.trim_start_matches('/');
        if let Some(q) = query {
            format!("ravn://{}?{}", clean_path, q)
        } else {
            format!("ravn://{}", clean_path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_url() {
        let url = NavigationUrl::parse("ravn://settings").unwrap();
        assert_eq!(url.path, "settings");
        assert_eq!(url.query, None);
    }

    #[test]
    fn test_parse_nested_path() {
        let url = NavigationUrl::parse("ravn://settings/ai").unwrap();
        assert_eq!(url.path, "settings/ai");
        assert_eq!(url.query, None);
    }

    #[test]
    fn test_parse_with_query() {
        let url = NavigationUrl::parse("ravn://compose?to=test@example.com").unwrap();
        assert_eq!(url.path, "compose");
        assert_eq!(url.query, Some("to=test@example.com".to_string()));
    }

    #[test]
    fn test_to_router_path() {
        let url = NavigationUrl::parse("ravn://settings/ai").unwrap();
        assert_eq!(url.to_router_path(), "/settings/ai");
    }

    #[test]
    fn test_settings_default_mapping() {
        let url = NavigationUrl::parse("ravn://settings").unwrap();
        assert_eq!(url.to_router_path(), "/settings/ai");
    }

    #[test]
    fn test_mail_default_mapping() {
        let url = NavigationUrl::parse("ravn://mail").unwrap();
        assert_eq!(url.to_router_path(), "/mail/inbox");
    }

    #[test]
    fn test_to_router_path_with_query() {
        let url = NavigationUrl::parse("ravn://compose?to=test@example.com").unwrap();
        assert_eq!(url.to_router_path(), "/compose?to=test@example.com");
    }

    #[test]
    fn test_build_url() {
        let url = NavigationUrl::build("settings/ai", None);
        assert_eq!(url, "ravn://settings/ai");
    }

    #[test]
    fn test_build_url_with_query() {
        let url = NavigationUrl::build("compose", Some("to=test@example.com"));
        assert_eq!(url, "ravn://compose?to=test@example.com");
    }
}
