use super::client::ActivationClient;
use super::types::*;
use chrono::Utc;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

const LICENSE_FILE_NAME: &str = "license.json";
const LICENSE_STALE_HOURS: i64 = 1;

pub struct LicenseManager {
    app_data_dir: PathBuf,
    machine_id: String,
    client: Option<ActivationClient>,
    cached_license: Arc<RwLock<Option<CachedLicense>>>,
    is_open_source_mode: bool,
}

impl LicenseManager {
    pub fn new(
        app_data_dir: PathBuf,
        activation_service_url: Option<String>,
        mid_secret: Option<String>,
    ) -> Result<Self, ActivationError> {
        let is_open_source_mode = activation_service_url.is_none() || mid_secret.is_none();

        let machine_id = if let Some(secret) = mid_secret {
            mid::get(&secret).map_err(|e| {
                ActivationError::Unknown(format!("Failed to generate machine ID: {}", e))
            })?
        } else {
            "opensource".to_string()
        };

        let client = activation_service_url.map(|url| ActivationClient::new(url));

        log::info!(
            "LicenseManager initialized - Open Source Mode: {}, Machine ID: {}",
            is_open_source_mode,
            &machine_id[..8.min(machine_id.len())]
        );

        Ok(Self {
            app_data_dir,
            machine_id,
            client,
            cached_license: Arc::new(RwLock::new(None)),
            is_open_source_mode,
        })
    }

    pub fn is_open_source_mode(&self) -> bool {
        self.is_open_source_mode
    }

    pub fn get_machine_id(&self) -> String {
        self.machine_id.clone()
    }

    fn license_file_path(&self) -> PathBuf {
        self.app_data_dir.join(LICENSE_FILE_NAME)
    }

    pub async fn load_cached_license(&self) -> Result<(), ActivationError> {
        let path = self.license_file_path();

        if !path.exists() {
            log::info!("No cached license found");
            return Ok(());
        }

        let contents = fs::read_to_string(&path).await?;
        let license: CachedLicense = serde_json::from_str(&contents)?;

        log::info!("Loaded cached license for user: {}", license.user_email);

        let mut cached = self.cached_license.write().await;
        *cached = Some(license);

        Ok(())
    }

    async fn persist_license(&self, license: &CachedLicense) -> Result<(), ActivationError> {
        let path = self.license_file_path();
        let contents = serde_json::to_string_pretty(license)?;

        fs::write(&path, contents).await?;

        log::info!("License persisted to disk");

        Ok(())
    }

    pub async fn activate(
        &self,
        license_key: String,
    ) -> Result<ActivationResponse, ActivationError> {
        if self.is_open_source_mode {
            return Err(ActivationError::ActivationFailed(
                "Cannot activate in open source mode".to_string(),
            ));
        }

        let client = self
            .client
            .as_ref()
            .ok_or_else(|| ActivationError::ServiceUnavailable)?;

        let response = client
            .activate(self.machine_id.clone(), license_key)
            .await?;

        let cached_license = CachedLicense::from(response.clone());
        self.persist_license(&cached_license).await?;

        let mut cached = self.cached_license.write().await;
        *cached = Some(cached_license);

        log::info!("License activated and cached");

        Ok(response)
    }

    pub async fn start_trial(&self, email: String) -> Result<ActivationResponse, ActivationError> {
        if self.is_open_source_mode {
            return Err(ActivationError::TrialFailed(
                "Cannot start trial in open source mode".to_string(),
            ));
        }

        let client = self
            .client
            .as_ref()
            .ok_or_else(|| ActivationError::ServiceUnavailable)?;

        let response = client.start_trial(self.machine_id.clone(), email).await?;

        let cached_license = CachedLicense::from(response.clone());
        self.persist_license(&cached_license).await?;

        let mut cached = self.cached_license.write().await;
        *cached = Some(cached_license);

        log::info!("Trial started and cached");

        Ok(response)
    }

    pub async fn validate_license(&self) -> Result<bool, ActivationError> {
        if self.is_open_source_mode {
            log::debug!("Open source mode - skipping validation");
            return Ok(true);
        }

        let cached = self.cached_license.read().await;
        let license = cached
            .as_ref()
            .ok_or_else(|| ActivationError::LicenseNotFound)?;

        let client = self
            .client
            .as_ref()
            .ok_or_else(|| ActivationError::ServiceUnavailable)?;

        // Check if service is reachable
        if !client.is_service_reachable().await {
            log::warn!("Activation service not reachable - using cached license");

            // Soft pass if cached license is not too old
            if !license.is_stale(LICENSE_STALE_HOURS * 24) {
                // Allow up to 24 hours offline
                log::info!("Using cached license (offline grace period)");
                return Ok(true);
            } else {
                log::warn!("Cached license is too old and service is unreachable");
                return Err(ActivationError::ValidationFailed(
                    "Service unreachable and cached license expired".to_string(),
                ));
            }
        }

        // Validate with service
        match client.validate(license.license_key.clone()).await {
            Ok(response) => {
                drop(cached); // Release read lock before acquiring write lock
                let mut cached = self.cached_license.write().await;
                if let Some(license) = cached.as_mut() {
                    license.update_validation(response);
                    self.persist_license(license).await?;
                }
                log::info!("License validated and updated");
                Ok(true)
            }
            Err(e) => {
                log::error!("License validation failed: {}", e);
                Err(e)
            }
        }
    }

    pub async fn refresh_license(&self) -> Result<(), ActivationError> {
        if self.is_open_source_mode {
            return Ok(());
        }

        let cached = self.cached_license.read().await;
        let license = match cached.as_ref() {
            Some(l) => l,
            None => {
                log::debug!("No license to refresh");
                return Ok(());
            }
        };

        if !license.is_stale(LICENSE_STALE_HOURS) {
            log::debug!("License is still fresh, skipping refresh");
            return Ok(());
        }

        let license_key = license.license_key.clone();
        drop(cached); // Release read lock

        log::info!("Refreshing stale license");
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| ActivationError::ServiceUnavailable)?;

        // Check if service is reachable before attempting refresh
        if !client.is_service_reachable().await {
            log::warn!("Service not reachable, skipping refresh");
            return Ok(());
        }

        match client.validate(license_key).await {
            Ok(response) => {
                let mut cached = self.cached_license.write().await;
                if let Some(license) = cached.as_mut() {
                    license.update_validation(response);
                    self.persist_license(license).await?;
                }
                log::info!("License refreshed successfully");
                Ok(())
            }
            Err(e) => {
                log::error!("License refresh failed: {}", e);
                Err(e)
            }
        }
    }

    pub async fn get_status(&self) -> LicenseStatus {
        if self.is_open_source_mode {
            return LicenseStatus {
                is_licensed: true,
                mode: LicenseMode::OpenSource,
                status: None,
                user_name: Some("Open Source User".to_string()),
                user_email: None,
                ai_mode: Some(AiMode::BYOK),
                ai_limit: Some(f64::INFINITY),
                ai_limit_remaining: Some(f64::INFINITY),
                expires_at: None,
                trial_ends_at: None,
                validated_at: Some(Utc::now()),
            };
        }

        let cached = self.cached_license.read().await;
        match cached.as_ref() {
            Some(license) => {
                let mode = match license.status {
                    LicenseStatusType::Trial => LicenseMode::Trial,
                    LicenseStatusType::Active => LicenseMode::Licensed,
                    LicenseStatusType::Expired | LicenseStatusType::Suspended => {
                        LicenseMode::Unlicensed
                    }
                };

                LicenseStatus {
                    is_licensed: !license.is_expired() && mode != LicenseMode::Unlicensed,
                    mode,
                    status: Some(license.status.clone()),
                    user_name: Some(license.user_name.clone()),
                    user_email: Some(license.user_email.clone()),
                    ai_mode: Some(license.ai_mode.clone()),
                    ai_limit: license.ai_details.as_ref().map(|d| d.limit),
                    ai_limit_remaining: license.ai_details.as_ref().map(|d| d.limit_remaining),
                    expires_at: license.expires_at.clone(),
                    trial_ends_at: license.trial_ends_at.clone(),
                    validated_at: Some(license.validated_at),
                }
            }
            None => LicenseStatus::default(),
        }
    }

    pub async fn get_cached_license(&self) -> Option<CachedLicense> {
        let cached = self.cached_license.read().await;
        cached.clone()
    }

    pub async fn get_ai_token(&self) -> Option<String> {
        if self.is_open_source_mode {
            return None;
        }

        let cached = self.cached_license.read().await;
        cached
            .as_ref()
            .and_then(|license| license.ai_details.as_ref())
            .filter(|_| cached.as_ref().unwrap().ai_mode == AiMode::SaaS)
            .map(|details| details.token.clone())
    }

    pub async fn get_ai_limits(&self) -> (f64, f64) {
        if self.is_open_source_mode {
            return (f64::INFINITY, f64::INFINITY);
        }

        let cached = self.cached_license.read().await;
        match cached
            .as_ref()
            .and_then(|license| license.ai_details.as_ref())
        {
            Some(details) => (details.limit, details.limit_remaining),
            None => (0.0, 0.0),
        }
    }

    pub async fn should_enable_ai(&self, user_api_key: Option<String>) -> bool {
        if self.is_open_source_mode {
            // In BYOK mode, enable only if user has configured their own key
            return user_api_key.is_some();
        }

        let cached = self.cached_license.read().await;
        let license = match cached.as_ref() {
            Some(l) => l,
            None => return false, // No license = no AI
        };

        match license.ai_mode {
            AiMode::BYOK => {
                // BYOK mode: enable only if user has configured their own key
                user_api_key.is_some()
            }
            AiMode::SaaS => {
                // SaaS mode: enable if user has their own key OR if license provides one
                user_api_key.is_some()
                    || license
                        .ai_details
                        .as_ref()
                        .map(|d| !d.token.is_empty())
                        .unwrap_or(false)
            }
        }
    }

    pub async fn clear_license(&self) -> Result<(), ActivationError> {
        let path = self.license_file_path();
        if path.exists() {
            fs::remove_file(&path).await?;
            log::info!("License file removed");
        }

        let mut cached = self.cached_license.write().await;
        *cached = None;

        log::info!("License cleared from cache");

        Ok(())
    }
}
