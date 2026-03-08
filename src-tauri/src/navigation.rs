use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use url::{form_urlencoded::Serializer, Url};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NavigationUrl {
    pub scheme: String,
    pub path: String,
    pub query: Option<String>,
}

#[derive(Debug, Default)]
pub struct NavigationDispatchState {
    frontend_ready: AtomicBool,
    pending_urls: Mutex<Vec<String>>,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct MailtoPayload {
    to: Vec<String>,
    cc: Vec<String>,
    bcc: Vec<String>,
    subject: Option<String>,
    body: Option<String>,
}

impl NavigationDispatchState {
    pub fn mark_frontend_ready(&self) -> Vec<String> {
        self.frontend_ready.store(true, Ordering::SeqCst);
        let mut pending_urls = self.pending_urls.lock().expect("navigation queue poisoned");
        std::mem::take(&mut *pending_urls)
    }

    fn queue_url(&self, url: String) {
        let mut pending_urls = self.pending_urls.lock().expect("navigation queue poisoned");
        pending_urls.push(url);
    }

    fn is_frontend_ready(&self) -> bool {
        self.frontend_ready.load(Ordering::SeqCst)
    }
}

impl NavigationUrl {
    pub fn parse(url: &str) -> Result<Self, String> {
        if url.starts_with("ravn://") {
            Self::parse_ravn(url)
        } else if url.starts_with("mailto:") {
            Self::parse_mailto(url)
        } else {
            Err(format!("Unsupported navigation URL scheme: {}", url))
        }
    }

    fn parse_ravn(url: &str) -> Result<Self, String> {
        if !url.starts_with("ravn://") {
            return Err(format!("Invalid RAVN URL scheme: {}", url));
        }

        let without_scheme = &url[7..];
        let (path, query) = if let Some(query_start) = without_scheme.find('?') {
            (
                without_scheme[..query_start].to_string(),
                Some(without_scheme[query_start + 1..].to_string()),
            )
        } else {
            (without_scheme.to_string(), None)
        };

        Ok(Self {
            scheme: "ravn".to_string(),
            path,
            query,
        })
    }

    fn parse_mailto(url: &str) -> Result<Self, String> {
        let payload = MailtoPayload::parse(url)?;
        let query = payload.to_query_string();

        Ok(Self {
            scheme: "mailto".to_string(),
            path: "compose".to_string(),
            query: (!query.is_empty()).then_some(query),
        })
    }

    pub fn to_router_path(&self) -> String {
        let base_path = if self.path.is_empty() {
            "/".to_string()
        } else if self.path.starts_with('/') {
            self.path.clone()
        } else {
            format!("/{}", self.path)
        };

        if let Some(query) = &self.query {
            format!("{}?{}", base_path, query)
        } else {
            base_path
        }
    }

    pub fn build(path: &str, query: Option<&str>) -> String {
        let clean_path = path.trim_start_matches('/');
        if let Some(q) = query {
            format!("ravn://{}?{}", clean_path, q)
        } else {
            format!("ravn://{}", clean_path)
        }
    }
}

impl MailtoPayload {
    fn parse(url: &str) -> Result<Self, String> {
        let parsed_url =
            Url::parse(url).map_err(|error| format!("Invalid mailto URL: {}", error))?;

        if parsed_url.scheme() != "mailto" {
            return Err(format!("Invalid mailto URL scheme: {}", url));
        }

        let mut payload = Self {
            to: split_address_list(decode_component(parsed_url.path())?),
            ..Self::default()
        };

        if let Some(query) = parsed_url.query() {
            for (raw_name, raw_value) in parse_mailto_query(query)? {
                match raw_name.to_ascii_lowercase().as_str() {
                    "to" => payload.to.extend(split_address_list(raw_value)),
                    "cc" => payload.cc.extend(split_address_list(raw_value)),
                    "bcc" => payload.bcc.extend(split_address_list(raw_value)),
                    "subject" => payload.subject = Some(raw_value),
                    "body" => payload.body = Some(raw_value),
                    _ => {}
                }
            }
        }

        Ok(payload)
    }

    fn to_query_string(&self) -> String {
        let mut serializer = Serializer::new(String::new());

        for recipient in &self.to {
            serializer.append_pair("to", recipient);
        }
        for recipient in &self.cc {
            serializer.append_pair("cc", recipient);
        }
        for recipient in &self.bcc {
            serializer.append_pair("bcc", recipient);
        }
        if let Some(subject) = &self.subject {
            serializer.append_pair("subject", subject);
        }
        if let Some(body) = &self.body {
            serializer.append_pair("body", body);
        }

        serializer.finish()
    }
}

pub fn reveal_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub fn dispatch_navigation_url<R: Runtime>(app: &AppHandle<R>, url: String) {
    reveal_main_window(app);

    let navigation_state = app.state::<NavigationDispatchState>();
    if navigation_state.is_frontend_ready() {
        emit_navigation_url(app, &url);
    } else {
        navigation_state.queue_url(url);
    }
}

fn emit_navigation_url<R: Runtime>(app: &AppHandle<R>, url: &str) {
    if let Some(window) = app.get_webview_window("main") {
        if let Err(error) = window.emit("navigate-to-url", url) {
            log::error!("[Navigation] Failed to emit navigation event: {}", error);
        }
    } else {
        log::error!("[Navigation] Main window not found for URL: {}", url);
    }
}

fn parse_mailto_query(query: &str) -> Result<Vec<(String, String)>, String> {
    query
        .split('&')
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut parts = segment.splitn(2, '=');
            let name = decode_component(parts.next().unwrap_or_default())?;
            let value = decode_component(parts.next().unwrap_or_default())?;
            Ok((name, value))
        })
        .collect()
}

fn decode_component(value: &str) -> Result<String, String> {
    percent_decode_str(value)
        .decode_utf8()
        .map(|value| value.into_owned())
        .map_err(|error| {
            format!(
                "Invalid percent-encoding in URL component '{}': {}",
                value, error
            )
        })
}

fn split_address_list(value: String) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ravn_url() {
        let url = NavigationUrl::parse("ravn://settings/signatures").unwrap();

        assert_eq!(
            url,
            NavigationUrl {
                scheme: "ravn".to_string(),
                path: "settings/signatures".to_string(),
                query: None,
            }
        );
        assert_eq!(url.to_router_path(), "/settings/signatures");
    }

    #[test]
    fn parses_mailto_path_and_headers() {
        let url = NavigationUrl::parse(
            "mailto:alice@example.com,bob@example.com?cc=team@example.com&subject=Launch%20Plan&body=Line%201%0D%0ALine%202",
        )
        .unwrap();

        assert_eq!(url.scheme, "mailto");
        assert_eq!(url.path, "compose");
        assert_eq!(
            url.query,
            Some(
                "to=alice%40example.com&to=bob%40example.com&cc=team%40example.com&subject=Launch+Plan&body=Line+1%0D%0ALine+2"
                    .to_string(),
            )
        );
        assert_eq!(
            url.to_router_path(),
            "/compose?to=alice%40example.com&to=bob%40example.com&cc=team%40example.com&subject=Launch+Plan&body=Line+1%0D%0ALine+2"
        );
    }

    #[test]
    fn preserves_plus_signs_in_mailto_headers() {
        let url = NavigationUrl::parse("mailto:alice@example.com?subject=C++%20Notes&body=1+1=2")
            .unwrap();

        assert_eq!(
            url.query,
            Some("to=alice%40example.com&subject=C%2B%2B+Notes&body=1%2B1%3D2".to_string())
        );
    }

    #[test]
    fn supports_query_recipients_when_path_is_empty() {
        let url = NavigationUrl::parse(
            "mailto:?to=alice@example.com,bob@example.com&cc=carol@example.com&bcc=dave@example.com",
        )
        .unwrap();

        assert_eq!(
            url.query,
            Some(
                "to=alice%40example.com&to=bob%40example.com&cc=carol%40example.com&bcc=dave%40example.com"
                    .to_string(),
            )
        );
    }

    #[test]
    fn decodes_percent_encoded_recipients() {
        let url =
            NavigationUrl::parse("mailto:alice@example.com%2Cbob@example.com?subject=Hi").unwrap();

        assert_eq!(
            url.query,
            Some("to=alice%40example.com&to=bob%40example.com&subject=Hi".to_string(),)
        );
    }

    #[test]
    fn builds_ravn_url() {
        let url = NavigationUrl::build("settings/ai", None);
        assert_eq!(url, "ravn://settings/ai");
    }
}
