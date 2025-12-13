use crate::database::error::DatabaseError;
use reqwest::Client;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::fs;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AvatarProvider {
    Unavatar,
    Gravatar,
    Favicon,
}

impl AvatarProvider {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "unavatar" => Some(Self::Unavatar),
            "gravatar" => Some(Self::Gravatar),
            "favicon" => Some(Self::Favicon),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Unavatar => "unavatar",
            Self::Gravatar => "gravatar",
            Self::Favicon => "favicon",
        }
    }
}

/// Tracks rate limiting state for a provider
#[derive(Debug, Clone)]
struct RateLimitInfo {
    is_rate_limited: bool,
    rate_limited_at: Option<SystemTime>,
}

impl RateLimitInfo {
    fn new() -> Self {
        Self {
            is_rate_limited: false,
            rate_limited_at: None,
        }
    }

    fn mark_rate_limited(&mut self) {
        self.is_rate_limited = true;
        self.rate_limited_at = Some(SystemTime::now());
    }

    fn is_cooled_down(&self, cooldown_duration: Duration) -> bool {
        if !self.is_rate_limited {
            return true;
        }

        if let Some(limited_at) = self.rate_limited_at {
            if let Ok(elapsed) = limited_at.elapsed() {
                return elapsed >= cooldown_duration;
            }
        }

        false
    }

    fn reset(&mut self) {
        self.is_rate_limited = false;
        self.rate_limited_at = None;
    }
}

pub struct AvatarService {
    cache_dir: PathBuf,
    http_client: Client,
    pub providers: Vec<AvatarProvider>,
    rate_limit_state: Arc<RwLock<RateLimitInfo>>,
    rate_limit_cooldown: Duration,
}

impl AvatarService {
    /// Creates a new AvatarService with a list of providers to try in order
    /// If no providers are specified, defaults to [Unavatar, Favicon]
    pub fn new(cache_dir: PathBuf, providers: Option<Vec<AvatarProvider>>) -> Self {
        let contacts_dir = cache_dir.join("contacts");

        if let Err(e) = std::fs::create_dir_all(&contacts_dir) {
            log::warn!("Could not create avatar cache directory: {}", e);
        }

        let default_providers = vec![AvatarProvider::Unavatar, AvatarProvider::Favicon];

        Self {
            cache_dir: contacts_dir,
            http_client: Client::new(),
            providers: providers.unwrap_or(default_providers),
            rate_limit_state: Arc::new(RwLock::new(RateLimitInfo::new())),
            rate_limit_cooldown: Duration::from_secs(5 * 60),
        }
    }

    pub fn set_providers(&mut self, providers: Vec<AvatarProvider>) {
        self.providers = providers;
    }

    /// Fetches avatar by trying each provider in the configured order
    /// Skips rate-limited providers and resumes when cooldown expires
    pub async fn fetch_avatar(
        &self,
        contact_id: Uuid,
        email: &str,
    ) -> Result<(String, String), DatabaseError> {
        let mut last_error = None;

        for provider in &self.providers {
            if *provider == AvatarProvider::Unavatar {
                let state = self.rate_limit_state.read().await;
                if !state.is_cooled_down(self.rate_limit_cooldown) {
                    log::debug!(
                        "Skipping {} due to rate limiting, will retry after cooldown",
                        provider.as_str()
                    );
                    continue;
                }
            }

            let url = match provider {
                AvatarProvider::Unavatar => self.get_unavatar_url(email),
                AvatarProvider::Gravatar => self.get_gravatar_url(email),
                AvatarProvider::Favicon => self.get_favicon_url(email),
            };

            log::info!(
                "Attempting to fetch avatar from {} for {}",
                provider.as_str(),
                email
            );

            match self
                .fetch_and_cache(&url, contact_id, email, provider)
                .await
            {
                Ok(path) => {
                    log::debug!(
                        "Successfully fetched avatar from {} for {}",
                        provider.as_str(),
                        email
                    );

                    if *provider == AvatarProvider::Unavatar {
                        let mut state = self.rate_limit_state.write().await;
                        state.reset();
                    }

                    return Ok((
                        provider.as_str().to_string(),
                        path.to_string_lossy().to_string(),
                    ));
                }
                Err(e) => {
                    log::debug!("Failed to fetch avatar from {}: {}", provider.as_str(), e);
                    last_error = Some(e);
                }
            }
        }

        Err(DatabaseError::RepositoryError(last_error.unwrap_or_else(
            || "All avatar providers failed".to_string(),
        )))
    }

    fn get_gravatar_url(&self, email: &str) -> String {
        let trimmed = email.trim().to_lowercase();
        let hash = format!("{:x}", md5::compute(trimmed.as_bytes()));
        format!("https://www.gravatar.com/avatar/{}?d=404&s=256", hash)
    }

    fn get_favicon_url(&self, email: &str) -> String {
        let domain = email.split('@').nth(1).unwrap_or("unknown.com");
        format!(
            "https://www.google.com/s2/favicons?domain={}&sz=128",
            domain
        )
    }

    fn get_unavatar_url(&self, email: &str) -> String {
        format!("https://unavatar.io/{}?fallback=false", email)
    }

    async fn fetch_and_cache(
        &self,
        url: &str,
        contact_id: Uuid,
        _email: &str,
        provider: &AvatarProvider,
    ) -> Result<PathBuf, String> {
        let response = match self.http_client.get(url).send().await {
            Ok(response) => {
                let status = response.status();

                if status.as_u16() == 429 && *provider == AvatarProvider::Unavatar {
                    log::warn!("Rate limited by unavatar.io, marking for cooldown");
                    let mut state = self.rate_limit_state.write().await;
                    state.mark_rate_limited();
                    return Err(format!("Rate limited by provider (HTTP 429)"));
                }

                if !status.is_success() {
                    return Err(format!("Failed to fetch avatar: HTTP {}", status));
                }
                response
            }
            Err(e) => return Err(format!("Failed to fetch avatar: {}", e)),
        };

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("image/png");

        let ext = match content_type {
            ct if ct.contains("jpeg") || ct.contains("jpg") => "jpg",
            ct if ct.contains("png") => "png",
            ct if ct.contains("webp") => "webp",
            ct if ct.contains("svg") => "svg",
            ct if ct.contains("gif") => "gif",
            ct if ct.contains("ico") || ct.contains("icon") => "ico",
            _ => "png",
        };

        let bytes = match response.bytes().await {
            Ok(bytes) => bytes,
            Err(e) => return Err(format!("Failed to read avatar bytes: {}", e)),
        };

        if bytes.len() < 100 {
            return Err("Image data too small or empty".to_string());
        }

        let filename = format!("{}.{}", contact_id, ext);
        let cache_path = self.cache_dir.join(filename);

        if let Err(e) = fs::write(&cache_path, &bytes).await {
            return Err(format!("Failed to save avatar to cache: {}", e));
        }

        Ok(cache_path)
    }
}
