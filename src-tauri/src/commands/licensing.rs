use crate::licensing::{ActivationError, LicenseStatus};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivateRequest {
    pub license_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrialRequest {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseResponse {
    pub success: bool,
    pub message: String,
    pub status: Option<LicenseStatus>,
}

impl From<ActivationError> for LicenseResponse {
    fn from(error: ActivationError) -> Self {
        Self {
            success: false,
            message: error.to_string(),
            status: None,
        }
    }
}

#[tauri::command]
pub async fn license_activate(
    request: ActivateRequest,
    state: State<'_, AppState>,
) -> Result<LicenseResponse, String> {
    log::info!("License activation requested");

    match state.license_manager.activate(request.license_key).await {
        Ok(response) => {
            log::info!("License activated for user: {}", response.user_email);

            // Trigger AI service reconfiguration
            if let Err(e) = state.app_handle.emit("license-updated", ()) {
                log::error!("Failed to emit license-updated event: {}", e);
            }

            let status = state.license_manager.get_status().await;

            Ok(LicenseResponse {
                success: true,
                message: format!(
                    "License activated successfully. Welcome, {}!",
                    response.user_name
                ),
                status: Some(status),
            })
        }
        Err(e) => {
            log::error!("License activation failed: {}", e);
            Ok(LicenseResponse::from(e))
        }
    }
}

#[tauri::command]
pub async fn license_trial(
    request: TrialRequest,
    state: State<'_, AppState>,
) -> Result<LicenseResponse, String> {
    log::info!("Trial activation requested for email: {}", request.email);

    match state.license_manager.start_trial(request.email).await {
        Ok(response) => {
            log::info!("Trial started for user: {}", response.user_email);

            // Trigger AI service reconfiguration
            if let Err(e) = state.app_handle.emit("license-updated", ()) {
                log::error!("Failed to emit license-updated event: {}", e);
            }

            let status = state.license_manager.get_status().await;

            Ok(LicenseResponse {
                success: true,
                message: format!(
                    "Trial started successfully. Welcome, {}!",
                    response.user_name
                ),
                status: Some(status),
            })
        }
        Err(e) => {
            log::error!("Trial activation failed: {}", e);
            Ok(LicenseResponse::from(e))
        }
    }
}

#[tauri::command]
pub async fn license_status(state: State<'_, AppState>) -> Result<LicenseStatus, String> {
    log::debug!("License status requested");
    Ok(state.license_manager.get_status().await)
}

#[tauri::command]
pub async fn license_validate(state: State<'_, AppState>) -> Result<LicenseResponse, String> {
    log::info!("License validation requested");

    match state.license_manager.validate_license().await {
        Ok(_) => {
            let status = state.license_manager.get_status().await;
            Ok(LicenseResponse {
                success: true,
                message: "License is valid".to_string(),
                status: Some(status),
            })
        }
        Err(e) => {
            log::error!("License validation failed: {}", e);
            Ok(LicenseResponse::from(e))
        }
    }
}

#[tauri::command]
pub async fn license_clear(state: State<'_, AppState>) -> Result<LicenseResponse, String> {
    log::info!("License clear requested");

    match state.license_manager.clear_license().await {
        Ok(_) => {
            // Trigger AI service reconfiguration
            if let Err(e) = state.app_handle.emit("license-updated", ()) {
                log::error!("Failed to emit license-updated event: {}", e);
            }

            Ok(LicenseResponse {
                success: true,
                message: "License cleared successfully".to_string(),
                status: None,
            })
        }
        Err(e) => {
            log::error!("License clear failed: {}", e);
            Ok(LicenseResponse::from(e))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseDetails {
    pub is_licensed: bool,
    pub mode: String,
    pub key: Option<String>,
    pub status: Option<String>,
    pub user_name: Option<String>,
    pub user_email: Option<String>,
    pub ai_mode: Option<String>,
    pub expires_at: Option<String>,
    pub trial_ends_at: Option<String>,
    pub expiration_date: Option<String>,
    pub is_expired: bool,
    pub is_trial: bool,
    pub days_remaining: Option<i64>,
}

#[tauri::command]
pub async fn license_details(state: State<'_, AppState>) -> Result<LicenseDetails, String> {
    log::debug!("License details requested");

    let status = state.license_manager.get_status().await;
    let cached_license = state.license_manager.get_cached_license().await;

    let expiration_date = status.trial_ends_at.clone().or(status.expires_at.clone());
    let is_expired = cached_license
        .as_ref()
        .map(|l| l.is_expired())
        .unwrap_or(false);
    let is_trial = matches!(status.mode, crate::licensing::LicenseMode::Trial);
    let masked_key = cached_license.as_ref().map(|l| {
        let key = &l.license_key;
        if key.starts_with("trial-") {
            let parts: Vec<&str> = key.split('-').collect();
            if parts.len() == 4 {
                format!("{}-************-************-{}", parts[0], parts[3])
            } else {
                "****".to_string()
            }
        } else if key.contains('-') {
            let parts: Vec<&str> = key.split('-').collect();
            if parts.len() == 5 && parts[4].len() >= 8 {
                let first = &parts[0][..4.min(parts[0].len())];
                let last = &parts[4][parts[4].len().saturating_sub(8)..];
                format!("{}****-****-****-****-****{}", first, last)
            } else {
                "****".to_string()
            }
        } else {
            if key.len() >= 8 {
                let start = &key[..4];
                let end = &key[key.len() - 4..];
                format!("{}****{}", start, end)
            } else {
                "****".to_string()
            }
        }
    });

    let days_remaining = if let Some(exp_str) = &expiration_date {
        if let Ok(exp_date) = chrono::DateTime::parse_from_rfc3339(exp_str) {
            let now = chrono::Utc::now();
            let duration = exp_date
                .with_timezone(&chrono::Utc)
                .signed_duration_since(now);
            Some(duration.num_days())
        } else {
            None
        }
    } else {
        None
    };

    Ok(LicenseDetails {
        is_licensed: status.is_licensed,
        mode: format!("{:?}", status.mode),
        key: masked_key,
        status: status.status.as_ref().map(|s| format!("{:?}", s)),
        user_name: status.user_name,
        user_email: status.user_email,
        ai_mode: status.ai_mode.as_ref().map(|m| format!("{:?}", m)),
        expires_at: status.expires_at,
        trial_ends_at: status.trial_ends_at,
        expiration_date,
        is_expired,
        is_trial,
        days_remaining,
    })
}
