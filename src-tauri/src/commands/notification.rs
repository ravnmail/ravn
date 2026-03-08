/// Notification Tauri commands
use chrono::{DateTime, Utc};
use serde::Serialize;
use tauri::{Emitter, State};

use crate::database::repositories::{EmailRepository, RepositoryFactory};
use crate::services::notification_service::{BadgeCount, NotificationService};
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderNotificationEmail {
    pub id: String,
    pub account_id: String,
    pub folder_id: String,
    pub from: ReminderNotificationAddress,
    pub subject: Option<String>,
    pub snippet: Option<String>,
    pub remind_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReminderNotificationAddress {
    pub address: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderNotificationQueryResponse {
    pub due: Vec<ReminderNotificationEmail>,
    pub next_reminder_at: Option<String>,
}

fn notification_service_from_state(state: &State<'_, AppState>) -> NotificationService {
    NotificationService::new(state.db_pool.clone(), state.settings.clone())
        .with_app_handle(state.app_handle.clone())
}

fn map_reminder_email(email: crate::database::models::email::Email) -> ReminderNotificationEmail {
    ReminderNotificationEmail {
        id: email.id.to_string(),
        account_id: email.account_id.to_string(),
        folder_id: email.folder_id.to_string(),
        from: ReminderNotificationAddress {
            address: email.from.address.clone(),
            name: email.from.name.clone(),
        },
        subject: email.subject,
        snippet: email.snippet,
        remind_at: email.remind_at.map(|value| value.to_rfc3339()),
    }
}

/// Update the app badge count based on current unread emails
#[tauri::command]
pub async fn update_badge_count(state: State<'_, AppState>) -> Result<BadgeCount, String> {
    log::debug!("Manually updating badge count");

    let notification_service = notification_service_from_state(&state);

    notification_service
        .update_badge_count()
        .await
        .map_err(|e| format!("Failed to update badge count: {}", e))?;

    let count = notification_service
        .calculate_badge_count()
        .await
        .map_err(|e| format!("Failed to calculate badge count: {}", e))?;

    let settings = notification_service
        .get_notification_settings()
        .map_err(|e| format!("Failed to load notification settings: {}", e))?;
    let mode = settings
        .badge_type
        .clone()
        .unwrap_or_else(|| "count".to_string());
    let visible = match mode.as_str() {
        "none" => false,
        "dot" | "count" => count > 0,
        _ => count > 0,
    };

    Ok(BadgeCount {
        count,
        visible,
        mode,
    })
}

/// Get the current badge count without updating
#[tauri::command]
pub async fn get_badge_count(state: State<'_, AppState>) -> Result<BadgeCount, String> {
    log::debug!("Getting current badge count");

    let notification_service = notification_service_from_state(&state);

    let count = notification_service
        .calculate_badge_count()
        .await
        .map_err(|e| format!("Failed to calculate badge count: {}", e))?;

    let settings = notification_service
        .get_notification_settings()
        .map_err(|e| format!("Failed to load notification settings: {}", e))?;
    let mode = settings
        .badge_type
        .clone()
        .unwrap_or_else(|| "count".to_string());
    let visible = match mode.as_str() {
        "none" => false,
        "dot" | "count" => count > 0,
        _ => count > 0,
    };

    Ok(BadgeCount {
        count,
        visible,
        mode,
    })
}

/// Test notification sound
#[tauri::command]
pub async fn test_notification_sound(
    state: State<'_, AppState>,
    sound_name: String,
) -> Result<NotificationResponse, String> {
    log::info!("Testing notification sound: {}", sound_name);

    state
        .app_handle
        .emit("play-sound", sound_name.clone())
        .map_err(|e| format!("Failed to emit sound event: {}", e))?;

    Ok(NotificationResponse {
        success: true,
        message: format!("Test sound '{}' triggered", sound_name),
    })
}

#[tauri::command]
pub async fn get_due_reminder_notifications(
    state: State<'_, AppState>,
) -> Result<ReminderNotificationQueryResponse, String> {
    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let email_repo = repo_factory.email_repository();

    let now = Utc::now();
    let all_emails = email_repo
        .find_with_folder_type()
        .await
        .map_err(|e| format!("Failed to query reminder emails: {}", e))?;

    let mut due = Vec::new();
    let mut next_reminder_at: Option<DateTime<Utc>> = None;

    for (email, _folder_type) in all_emails {
        if email.is_deleted || email.is_draft {
            continue;
        }

        let Some(remind_at) = email.remind_at else {
            continue;
        };

        if remind_at <= now {
            due.push(map_reminder_email(email));
        } else if next_reminder_at
            .map(|current| remind_at < current)
            .unwrap_or(true)
        {
            next_reminder_at = Some(remind_at);
        }
    }

    due.sort_by(|a, b| {
        let a_ts = a
            .remind_at
            .as_deref()
            .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.timestamp())
            .unwrap_or_default();
        let b_ts = b
            .remind_at
            .as_deref()
            .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.timestamp())
            .unwrap_or_default();

        a_ts.cmp(&b_ts).then_with(|| a.id.cmp(&b.id))
    });

    Ok(ReminderNotificationQueryResponse {
        due,
        next_reminder_at: next_reminder_at.map(|value| value.to_rfc3339()),
    })
}
