use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::config::settings::Settings;
use crate::database::repositories::{EmailRepository, SqliteEmailRepository};
use crate::sync::types::FolderType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
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
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            incoming_sound: Some("notification".to_string()),
            outgoing_sound: None,
            reminder_sound: None,
            notification_folders: Some(vec![]),
            badge_folders: Some(vec![]),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BadgeCount {
    pub count: i64,
}

pub struct NotificationService {
    pool: SqlitePool,
    settings: Arc<Settings>,
    app_handle: Option<AppHandle>,
}

impl NotificationService {
    pub fn new(pool: SqlitePool, settings: Arc<Settings>) -> Self {
        Self {
            pool,
            settings,
            app_handle: None,
        }
    }

    pub fn with_app_handle(mut self, app_handle: AppHandle) -> Self {
        self.app_handle = Some(app_handle);
        self
    }

    /// Get notification settings from config
    pub fn get_notification_settings(&self) -> Result<NotificationSettings, String> {
        match self.settings.get::<NotificationSettings>("notifications") {
            Ok(settings) => Ok(settings),
            Err(_) => {
                log::warn!("Failed to load notification settings, using defaults");
                Ok(NotificationSettings::default())
            }
        }
    }

    /// Check if a folder should trigger incoming notifications
    pub async fn should_notify_for_folder(
        &self,
        folder_id: Uuid,
        folder_type: FolderType,
    ) -> Result<bool, String> {
        let settings = self.get_notification_settings()?;

        match &settings.notification_folders {
            None => Ok(false),
            Some(folders) if folders.is_empty() => Ok(folder_type == FolderType::Inbox),
            Some(folders) => {
                let folder_id_str = folder_id.to_string();
                Ok(folders.contains(&folder_id_str))
            }
        }
    }

    /// Play incoming email sound
    pub async fn play_incoming_sound(&self) -> Result<(), String> {
        let settings = self.get_notification_settings()?;

        if let Some(sound_name) = settings.incoming_sound {
            self.play_sound(&sound_name).await?;
        }

        Ok(())
    }

    /// Play outgoing email sound
    pub async fn play_outgoing_sound(&self) -> Result<(), String> {
        let settings = self.get_notification_settings()?;

        if let Some(sound_name) = settings.outgoing_sound {
            self.play_sound(&sound_name).await?;
        }

        Ok(())
    }

    /// Play a sound file by name
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

    /// Calculate total unread count for badge folders
    pub async fn calculate_badge_count(&self) -> Result<i64, String> {
        let settings = self.get_notification_settings()?;
        let email_repo = SqliteEmailRepository::new(self.pool.clone());

        let count = match &settings.badge_folders {
            None => {
                log::info!("Badge count disabled (no badge folders set)");
                0
            }
            Some(folders) if folders.is_empty() => {
                log::info!("Calculating badge count for all folders");
                email_repo
                    .count_unread_all()
                    .await
                    .map_err(|e| format!("Failed to count unread emails: {}", e))?
            }
            Some(folders) => {
                let folder_ids: Result<Vec<Uuid>, _> = folders
                    .iter()
                    .map(|id_str| Uuid::parse_str(id_str))
                    .collect();

                let folder_ids =
                    folder_ids.map_err(|e| format!("Failed to parse folder IDs: {}", e))?;

                log::info!("Calculating badge count for {} folders", folder_ids.len());

                email_repo
                    .count_unread_by_folders(&folder_ids)
                    .await
                    .map_err(|e| format!("Failed to count unread emails in badge folders: {}", e))?
            }
        };

        Ok(count)
    }

    /// Update app badge count
    pub async fn update_badge_count(&self) -> Result<(), String> {
        let count = self.calculate_badge_count().await?;

        if let Some(app_handle) = &self.app_handle {
            app_handle
                .emit("badge-count-updated", BadgeCount { count })
                .map_err(|e| format!("Failed to emit badge count event: {}", e))?;

            log::debug!("Updated badge count: {}", count);
        } else {
            log::warn!("Cannot update badge count: AppHandle not available");
        }

        Ok(())
    }

    /// Trigger notification for incoming email
    pub async fn notify_incoming_email(
        &self,
        folder_id: Uuid,
        folder_type: FolderType,
    ) -> Result<(), String> {
        if self
            .should_notify_for_folder(folder_id, folder_type)
            .await?
        {
            self.play_incoming_sound().await?;
        }

        self.update_badge_count().await?;

        Ok(())
    }

    /// Trigger notification for outgoing email
    pub async fn notify_outgoing_email(&self) -> Result<(), String> {
        self.play_outgoing_sound().await?;
        Ok(())
    }
}
