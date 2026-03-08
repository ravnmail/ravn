use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::{NotificationExt, PermissionState};
use uuid::Uuid;

use crate::config::settings::Settings;
use crate::database::models::email::Email;
use crate::database::repositories::{
    ContactRepository, FolderRepository, SqliteContactRepository, SqliteFolderRepository,
};
use crate::sync::types::FolderType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    #[serde(rename = "enabled")]
    pub enabled: Option<bool>,
    #[serde(rename = "incomingSound")]
    pub incoming_sound: Option<String>,
    #[serde(rename = "outgoingSound")]
    pub outgoing_sound: Option<String>,
    #[serde(rename = "reminderSound")]
    pub reminder_sound: Option<String>,
    #[serde(rename = "notificationFolders")]
    pub notification_folders: Option<Vec<String>>,
    #[serde(rename = "badgeFolders")]
    pub badge_folders: Option<Vec<String>>,
    #[serde(rename = "badgeType")]
    pub badge_type: Option<String>,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: Some(true),
            incoming_sound: Some("notification".to_string()),
            outgoing_sound: None,
            reminder_sound: None,
            notification_folders: Some(vec![]),
            badge_folders: Some(vec![]),
            badge_type: Some("count".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BadgeCount {
    pub count: i64,
    pub visible: bool,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationEmailPreview {
    pub id: String,
    pub account_id: String,
    pub folder_id: String,
    pub conversation_id: Option<String>,
    pub sender_name: Option<String>,
    pub sender_address: Option<String>,
    pub subject: Option<String>,
    pub snippet: Option<String>,
    pub avatar_url: Option<String>,
    pub remind_at: Option<String>,
    pub navigation_target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationEventPayload {
    pub kind: String,
    pub title: String,
    pub body: Option<String>,
    pub email: Option<NotificationEmailPreview>,
    pub play_sound: bool,
    pub suppress_during_bootstrap: bool,
    pub tag: Option<String>,
}

pub struct NotificationService {
    pool: SqlitePool,
    settings: Arc<Settings>,
    app_handle: Option<AppHandle>,
    suppress_notifications: bool,
}

impl NotificationService {
    pub fn new(pool: SqlitePool, settings: Arc<Settings>) -> Self {
        Self {
            pool,
            settings,
            app_handle: None,
            suppress_notifications: false,
        }
    }

    pub fn with_app_handle(mut self, app_handle: AppHandle) -> Self {
        self.app_handle = Some(app_handle);
        self
    }

    pub fn suppress_notifications(mut self, suppress_notifications: bool) -> Self {
        self.suppress_notifications = suppress_notifications;
        self
    }

    pub fn get_notification_settings(&self) -> Result<NotificationSettings, String> {
        match self.settings.get::<NotificationSettings>("notifications") {
            Ok(settings) => Ok(settings),
            Err(_) => {
                log::warn!("Failed to load notification settings, using defaults");
                Ok(NotificationSettings::default())
            }
        }
    }

    fn notifications_enabled(&self, settings: &NotificationSettings) -> bool {
        settings.enabled.unwrap_or(true)
    }

    fn badge_mode(&self, settings: &NotificationSettings) -> String {
        settings
            .badge_type
            .clone()
            .unwrap_or_else(|| "count".to_string())
    }

    fn badge_visible(&self, settings: &NotificationSettings, count: i64) -> bool {
        match self.badge_mode(settings).as_str() {
            "none" => false,
            "dot" => count > 0,
            "count" => count > 0,
            _ => count > 0,
        }
    }

    async fn apply_badge_count(&self, count: i64) -> Result<(), String> {
        let settings = self.get_notification_settings()?;
        let mode = self.badge_mode(&settings);

        if let Some(app_handle) = &self.app_handle {
            if let Some(window) = app_handle.get_webview_window("main") {
                let badge_count = match mode.as_str() {
                    "none" => None,
                    "dot" => {
                        if count > 0 {
                            Some(1)
                        } else {
                            None
                        }
                    }
                    "count" => {
                        if count > 0 {
                            Some(count)
                        } else {
                            None
                        }
                    }
                    _ => {
                        if count > 0 {
                            Some(count)
                        } else {
                            None
                        }
                    }
                };

                window
                    .set_badge_count(badge_count)
                    .map_err(|e| format!("Failed to set badge count: {}", e))?;
            } else {
                log::warn!("Cannot update badge count: main window not available");
            }
        } else {
            log::warn!("Cannot update badge count: AppHandle not available");
        }

        Ok(())
    }

    async fn show_native_notification(&self, title: &str, body: &str) -> Result<(), String> {
        let settings = self.get_notification_settings()?;
        if !self.notifications_enabled(&settings) {
            return Ok(());
        }

        let Some(app_handle) = &self.app_handle else {
            log::warn!("Cannot show native notification: AppHandle not available");
            return Ok(());
        };

        let notification = app_handle.notification();

        match notification.permission_state() {
            Ok(PermissionState::Granted) => {}
            Ok(PermissionState::Denied) => {
                log::info!("Notification permission denied by OS");
                return Ok(());
            }
            Ok(PermissionState::Prompt) | Ok(PermissionState::PromptWithRationale) => {
                match notification.request_permission() {
                    Ok(PermissionState::Granted) => {}
                    Ok(state) => {
                        log::info!("Notification permission not granted: {:?}", state);
                        return Ok(());
                    }
                    Err(e) => {
                        return Err(format!("Failed to request notification permission: {}", e));
                    }
                }
            }
            Err(e) => {
                return Err(format!(
                    "Failed to get notification permission state: {}",
                    e
                ));
            }
        }

        notification
            .builder()
            .title(title)
            .body(body)
            .show()
            .map_err(|e| format!("Failed to show native notification: {}", e))?;

        Ok(())
    }

    fn emit_native_notification_event(
        &self,
        payload: &NotificationEventPayload,
    ) -> Result<(), String> {
        let Some(app_handle) = &self.app_handle else {
            log::warn!("Cannot emit native notification event: AppHandle not available");
            return Ok(());
        };

        app_handle
            .emit("native-notification", payload)
            .map_err(|e| format!("Failed to emit native notification event: {}", e))
    }

    fn emit_bootstrap_sync_state(&self, in_progress: bool) -> Result<(), String> {
        let Some(app_handle) = &self.app_handle else {
            log::warn!("Cannot emit bootstrap sync state: AppHandle not available");
            return Ok(());
        };

        app_handle
            .emit("notifications:bootstrap-sync-state", in_progress)
            .map_err(|e| format!("Failed to emit bootstrap sync state: {}", e))
    }

    pub fn begin_bootstrap_sync(&self) -> Result<(), String> {
        self.emit_bootstrap_sync_state(true)
    }

    pub fn end_bootstrap_sync(&self) -> Result<(), String> {
        self.emit_bootstrap_sync_state(false)
    }

    async fn resolve_avatar_url(&self, sender_address: &str) -> Option<String> {
        let repo = SqliteContactRepository::new(self.pool.clone());
        match repo.find_by_email(sender_address).await {
            Ok(Some(contact)) => contact.avatar_path,
            Ok(None) => None,
            Err(error) => {
                log::warn!(
                    "Failed to resolve notification avatar for {}: {}",
                    sender_address,
                    error
                );
                None
            }
        }
    }

    async fn build_email_preview(&self, email: &Email) -> NotificationEmailPreview {
        let sender_address = email.from.address.clone();
        let avatar_url = self.resolve_avatar_url(&sender_address).await;
        let email_id = email.id.to_string();
        let folder_id = email.folder_id.to_string();
        let account_id = email.account_id.to_string();
        let conversation_id = email.conversation_id.clone();
        let navigation_target = if let Some(conversation_id) = &conversation_id {
            Some(format!(
                "ravn://mail/{}/folders/{}/conversations/{}",
                account_id, folder_id, conversation_id
            ))
        } else {
            Some(format!(
                "ravn://mail/{}/folders/{}/emails/{}",
                account_id, folder_id, email_id
            ))
        };

        NotificationEmailPreview {
            id: email_id,
            account_id,
            folder_id,
            conversation_id,
            sender_name: email.from.name.clone(),
            sender_address: Some(sender_address),
            subject: email.subject.clone(),
            snippet: email.snippet.clone(),
            avatar_url,
            remind_at: email.remind_at.map(|value| value.to_rfc3339()),
            navigation_target,
        }
    }

    async fn build_incoming_notification_payload(&self, email: &Email) -> NotificationEventPayload {
        let preview = self.build_email_preview(email).await;
        let sender = preview
            .sender_name
            .clone()
            .or(preview.sender_address.clone())
            .unwrap_or_else(|| "Unknown sender".to_string());
        let subject = preview
            .subject
            .clone()
            .unwrap_or_else(|| "(no subject)".to_string());
        let body = preview
            .snippet
            .clone()
            .unwrap_or_else(|| format!("{} — {}", sender, subject));

        NotificationEventPayload {
            kind: "incoming-email".to_string(),
            title: sender,
            body: Some(body),
            email: Some(preview),
            play_sound: !self.suppress_notifications,
            suppress_during_bootstrap: true,
            tag: Some(format!("incoming-email:{}", email.id)),
        }
    }

    async fn build_reminder_notification_payload(&self, email: &Email) -> NotificationEventPayload {
        let preview = self.build_email_preview(email).await;
        let sender = preview
            .sender_name
            .clone()
            .or(preview.sender_address.clone())
            .unwrap_or_else(|| "Unknown sender".to_string());
        let subject = preview
            .subject
            .clone()
            .unwrap_or_else(|| "(no subject)".to_string());
        let body = preview
            .snippet
            .clone()
            .unwrap_or_else(|| format!("Reminder for {} — {}", sender, subject));

        NotificationEventPayload {
            kind: "reminder-email".to_string(),
            title: format!("Reminder: {}", subject),
            body: Some(body),
            email: Some(preview.clone()),
            play_sound: !self.suppress_notifications,
            suppress_during_bootstrap: false,
            tag: preview
                .remind_at
                .as_ref()
                .map(|remind_at| format!("reminder-email:{}:{}", email.id, remind_at)),
        }
    }

    fn build_outgoing_notification_payload(&self) -> NotificationEventPayload {
        NotificationEventPayload {
            kind: "outgoing-email".to_string(),
            title: "Email sent".to_string(),
            body: Some("Your email was sent successfully.".to_string()),
            email: None,
            play_sound: false,
            suppress_during_bootstrap: false,
            tag: Some("outgoing-email".to_string()),
        }
    }

    pub async fn should_notify_for_folder(
        &self,
        folder_id: Uuid,
        folder_type: FolderType,
    ) -> Result<bool, String> {
        let settings = self.get_notification_settings()?;

        if !self.notifications_enabled(&settings) {
            return Ok(false);
        }

        match &settings.notification_folders {
            None => Ok(false),
            Some(folders) if folders.is_empty() => Ok(folder_type == FolderType::Inbox),
            Some(folders) => {
                let folder_id_str = folder_id.to_string();
                Ok(folders.contains(&folder_id_str))
            }
        }
    }

    pub async fn play_incoming_sound(&self) -> Result<(), String> {
        let settings = self.get_notification_settings()?;

        if let Some(sound_name) = settings.incoming_sound {
            self.play_sound(&sound_name).await?;
        }

        Ok(())
    }

    pub async fn play_outgoing_sound(&self) -> Result<(), String> {
        let settings = self.get_notification_settings()?;

        if let Some(sound_name) = settings.outgoing_sound {
            self.play_sound(&sound_name).await?;
        }

        Ok(())
    }

    pub async fn play_reminder_sound(&self) -> Result<(), String> {
        let settings = self.get_notification_settings()?;

        if let Some(sound_name) = settings.reminder_sound {
            self.play_sound(&sound_name).await?;
        }

        Ok(())
    }

    async fn play_sound(&self, sound_name: &str) -> Result<(), String> {
        if let Some(app_handle) = &self.app_handle {
            app_handle
                .emit("play-sound", sound_name)
                .map_err(|e| format!("Failed to emit sound event: {}", e))?;

            log::debug!("Emitted play-sound event for: {}", sound_name);
        } else {
            log::warn!("Cannot play sound: AppHandle not available");
        }

        Ok(())
    }

    pub async fn calculate_badge_count(&self) -> Result<i64, String> {
        let settings = self.get_notification_settings()?;
        let folder_repo = SqliteFolderRepository::new(self.pool.clone());

        let mode = self.badge_mode(&settings);
        if mode == "none" {
            log::info!("Badge disabled by settings");
            return Ok(0);
        }

        let count = match &settings.badge_folders {
            None => {
                log::info!("Badge count disabled (no badge folders set)");
                0
            }
            Some(folders) if folders.is_empty() => {
                log::info!("Calculating badge count from unread totals for all folders");
                let folders = folder_repo
                    .get_all()
                    .await
                    .map_err(|e| format!("Failed to load folders for badge count: {}", e))?;

                folders.iter().map(|folder| folder.unread_count).sum()
            }
            Some(folder_ids) => {
                let parsed_folder_ids: Result<Vec<Uuid>, _> = folder_ids
                    .iter()
                    .map(|id_str| Uuid::parse_str(id_str))
                    .collect();

                let parsed_folder_ids =
                    parsed_folder_ids.map_err(|e| format!("Failed to parse folder IDs: {}", e))?;

                log::info!(
                    "Calculating badge count from unread totals for {} folders",
                    parsed_folder_ids.len()
                );

                let mut total = 0_i64;
                for folder_id in parsed_folder_ids {
                    if let Some(folder) = folder_repo.find_by_id(folder_id).await.map_err(|e| {
                        format!("Failed to load folder {} for badge count: {}", folder_id, e)
                    })? {
                        total += folder.unread_count;
                    }
                }

                total
            }
        };

        Ok(count)
    }

    pub async fn update_badge_count(&self) -> Result<(), String> {
        let settings = self.get_notification_settings()?;
        let count = self.calculate_badge_count().await?;
        let visible = self.badge_visible(&settings, count);
        let mode = self.badge_mode(&settings);

        self.apply_badge_count(count).await?;

        if let Some(app_handle) = &self.app_handle {
            app_handle
                .emit(
                    "badge-count-updated",
                    BadgeCount {
                        count,
                        visible,
                        mode,
                    },
                )
                .map_err(|e| format!("Failed to emit badge count event: {}", e))?;

            log::debug!("Updated badge count: {}, visible: {}", count, visible);
        } else {
            log::warn!("Cannot update badge count: AppHandle not available");
        }

        Ok(())
    }

    pub async fn notify_incoming_email(
        &self,
        folder_id: Uuid,
        folder_type: FolderType,
        email: &Email,
    ) -> Result<(), String> {
        if self
            .should_notify_for_folder(folder_id, folder_type)
            .await?
        {
            let payload = self.build_incoming_notification_payload(email).await;

            if !self.suppress_notifications {
                let body = payload
                    .body
                    .as_deref()
                    .unwrap_or("You have received a new email.");
                self.show_native_notification(&payload.title, body).await?;
                self.play_incoming_sound().await?;
            }

            self.emit_native_notification_event(&payload)?;
        }

        self.update_badge_count().await?;

        Ok(())
    }

    pub async fn notify_reminder_email(&self, email: &Email) -> Result<(), String> {
        let settings = self.get_notification_settings()?;
        if !self.notifications_enabled(&settings) {
            return Ok(());
        }

        let payload = self.build_reminder_notification_payload(email).await;

        if !self.suppress_notifications {
            let body = payload
                .body
                .as_deref()
                .unwrap_or("A reminder is due for one of your emails.");
            self.show_native_notification(&payload.title, body).await?;
            self.play_reminder_sound().await?;
        }

        self.emit_native_notification_event(&payload)?;
        Ok(())
    }

    pub async fn notify_outgoing_email(&self) -> Result<(), String> {
        let settings = self.get_notification_settings()?;
        if self.notifications_enabled(&settings) {
            let payload = self.build_outgoing_notification_payload();
            let body = payload
                .body
                .as_deref()
                .unwrap_or("Your email was sent successfully.");

            self.show_native_notification(&payload.title, body).await?;
            self.emit_native_notification_event(&payload)?;
        }
        self.play_outgoing_sound().await?;
        Ok(())
    }
}
