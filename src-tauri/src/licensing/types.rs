use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ActivationError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Invalid response from activation service: {0}")]
    InvalidResponse(String),

    #[error("License activation failed: {0}")]
    ActivationFailed(String),

    #[error("License validation failed: {0}")]
    ValidationFailed(String),

    #[error("Trial activation failed: {0}")]
    TrialFailed(String),

    #[error("License not found")]
    LicenseNotFound,

    #[error("License already activated")]
    LicenseAlreadyActivated,

    #[error("Trial already used")]
    TrialAlreadyUsed,

    #[error("Invalid instance name")]
    InvalidInstanceName,

    #[error("Invalid email address")]
    InvalidEmail,

    #[error("Service unavailable")]
    ServiceUnavailable,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("License expired")]
    LicenseExpired,

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl ActivationError {
    pub fn from_response(status: u16, body: &str) -> Self {
        match status {
            400 => Self::ActivationFailed(body.to_string()),
            403 => Self::TrialAlreadyUsed,
            404 => Self::LicenseNotFound,
            409 => Self::LicenseAlreadyActivated,
            503 => Self::ServiceUnavailable,
            _ => Self::Unknown(format!("{}: {}", status, body)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationRequest {
    #[serde(rename = "instanceName")]
    pub instance_name: String,
    #[serde(rename = "licenseKey")]
    pub license_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRequest {
    #[serde(rename = "licenseKey")]
    pub license_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialRequest {
    #[serde(rename = "instanceName")]
    pub instance_name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationResponse {
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    #[serde(rename = "licenseKey")]
    pub license_key: String,
    pub user_name: String,
    pub user_email: String,
    pub ai_mode: AiMode,
    pub status: LicenseStatusType,
    pub expires_at: Option<String>,
    pub trial_ends_at: Option<String>,
    pub ai_details: Option<AiDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LicenseStatusType {
    Active,
    Trial,
    Expired,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AiMode {
    #[serde(rename = "saas")]
    SaaS,
    #[serde(rename = "byok")]
    BYOK,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDetails {
    pub token: String,
    pub limit: f64,
    pub limit_remaining: f64,
    pub expires_at: String, // ISO 8601 datetime
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedLicense {
    pub instance_id: String,
    pub license_key: String,
    pub user_name: String,
    pub user_email: String,
    pub ai_mode: AiMode,
    pub status: LicenseStatusType,
    pub expires_at: Option<String>,
    pub trial_ends_at: Option<String>,
    pub ai_details: Option<AiDetails>,
    pub validated_at: DateTime<Utc>,
    pub cached_at: DateTime<Utc>,
}

impl From<ActivationResponse> for CachedLicense {
    fn from(response: ActivationResponse) -> Self {
        let now = Utc::now();
        Self {
            instance_id: response.instance_id,
            license_key: response.license_key,
            user_name: response.user_name,
            user_email: response.user_email,
            ai_mode: response.ai_mode,
            status: response.status,
            expires_at: response.expires_at,
            trial_ends_at: response.trial_ends_at,
            ai_details: response.ai_details,
            validated_at: now,
            cached_at: now,
        }
    }
}

impl CachedLicense {
    pub fn is_stale(&self, max_age_hours: i64) -> bool {
        let age = Utc::now().signed_duration_since(self.validated_at);
        age.num_hours() >= max_age_hours
    }

    pub fn update_validation(&mut self, response: ActivationResponse) {
        self.instance_id = response.instance_id;
        self.user_name = response.user_name;
        self.user_email = response.user_email;
        self.ai_mode = response.ai_mode;
        self.status = response.status;
        self.expires_at = response.expires_at;
        self.trial_ends_at = response.trial_ends_at;
        self.ai_details = response.ai_details;
        self.validated_at = Utc::now();
        self.cached_at = Utc::now();
    }

    pub fn get_expiration_date(&self) -> Option<String> {
        match self.status {
            LicenseStatusType::Trial => self.trial_ends_at.clone(),
            _ => self.expires_at.clone(),
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expiration) = self.get_expiration_date() {
            if let Ok(exp_date) = DateTime::parse_from_rfc3339(&expiration) {
                return exp_date.with_timezone(&Utc) < Utc::now();
            }
        }
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseStatus {
    pub is_licensed: bool,
    pub mode: LicenseMode,
    pub status: Option<LicenseStatusType>,
    pub user_name: Option<String>,
    pub user_email: Option<String>,
    pub ai_mode: Option<AiMode>,
    pub ai_limit: Option<f64>,
    pub ai_limit_remaining: Option<f64>,
    pub expires_at: Option<String>,
    pub trial_ends_at: Option<String>,
    pub validated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LicenseMode {
    OpenSource,
    Licensed,
    Trial,
    Unlicensed,
}

impl Default for LicenseStatus {
    fn default() -> Self {
        Self {
            is_licensed: false,
            mode: LicenseMode::Unlicensed,
            status: None,
            user_name: None,
            user_email: None,
            ai_mode: None,
            ai_limit: None,
            ai_limit_remaining: None,
            expires_at: None,
            trial_ends_at: None,
            validated_at: None,
        }
    }
}
