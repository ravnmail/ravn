/// Notification Tauri commands
use serde::Serialize;
use std::sync::Arc;
use tauri::{Emitter, State};

use crate::services::notification_service::{BadgeCount, NotificationService};
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub success: bool,
    pub message: String,
}

/// Update the app badge count based on current unread emails
#[tauri::command]
pub async fn update_badge_count(state: State<'_, AppState>) -> Result<BadgeCount, String> {
    log::debug!("Manually updating badge count");

    let notification_service = Arc::new(
        NotificationService::new(state.db_pool.clone(), state.settings.clone())
            .with_app_handle(state.app_handle.clone()),
    );

    notification_service
        .update_badge_count()
        .await
        .map_err(|e| format!("Failed to update badge count: {}", e))?;

    let count = notification_service
        .calculate_badge_count()
        .await
        .map_err(|e| format!("Failed to calculate badge count: {}", e))?;

    Ok(BadgeCount { count })
}

/// Get the current badge count without updating
#[tauri::command]
pub async fn get_badge_count(state: State<'_, AppState>) -> Result<BadgeCount, String> {
    log::debug!("Getting current badge count");

    let notification_service = Arc::new(
        NotificationService::new(state.db_pool.clone(), state.settings.clone())
            .with_app_handle(state.app_handle.clone()),
    );

    let count = notification_service
        .calculate_badge_count()
        .await
        .map_err(|e| format!("Failed to calculate badge count: {}", e))?;

    Ok(BadgeCount { count })
}

/// Test notification sound
#[tauri::command]
pub async fn test_notification_sound(
    state: State<'_, AppState>,
    sound_name: String,
) -> Result<NotificationResponse, String> {
    log::info!("Testing notification sound: {}", sound_name);

    // Emit sound event to frontend for testing
    state
        .app_handle
        .emit("play-sound", sound_name.clone())
        .map_err(|e| format!("Failed to emit sound event: {}", e))?;

    Ok(NotificationResponse {
        success: true,
        message: format!("Test sound '{}' triggered", sound_name),
    })
}
