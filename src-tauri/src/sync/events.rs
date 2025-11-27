use super::types::{SyncEmail, SyncFolder};
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use uuid::Uuid;

/// Event emitted when folders are updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoldersUpdatedEvent {
    pub account_id: Uuid,
    pub folders: Vec<SyncFolder>,
}

/// Event emitted when emails are updated in a folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailsUpdatedEvent {
    pub account_id: Uuid,
    pub folder_id: Uuid,
    pub emails: Vec<SyncEmail>,
}

/// Event emitted when folder counts are updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderCountsUpdatedEvent {
    pub account_id: Uuid,
    pub folder_id: Uuid,
    pub unread_count: i32,
    pub total_count: i32,
}

/// Event emitted when an email's read status changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailReadStatusChangedEvent {
    pub account_id: Uuid,
    pub email_id: Uuid,
    pub folder_id: Uuid,
    pub is_read: bool,
}

/// Event emitted when an email is flagged/unflagged
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailFlagChangedEvent {
    pub account_id: Uuid,
    pub email_id: Uuid,
    pub folder_id: Uuid,
    pub is_flagged: bool,
}

/// Event emitted when an email is moved to another folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMovedEvent {
    pub account_id: Uuid,
    pub email_id: Uuid,
    pub from_folder_id: Uuid,
    pub to_folder_id: Uuid,
}

/// Event emitted when an email is deleted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailDeletedEvent {
    pub account_id: Uuid,
    pub email_id: Uuid,
    pub folder_id: Uuid,
    pub permanent: bool,
}

/// Event emitted when a folder is renamed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderRenamedEvent {
    pub account_id: Uuid,
    pub folder_id: Uuid,
    pub old_name: String,
    pub new_name: String,
}

/// Event emitted when a folder is moved to a new parent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderMovedEvent {
    pub account_id: Uuid,
    pub folder_id: Uuid,
    pub old_parent_id: Option<Uuid>,
    pub new_parent_id: Option<Uuid>,
}

/// Sync status event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatusEvent {
    pub account_id: Uuid,
    pub folder_id: Option<Uuid>,
    pub status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Started,
    InProgress {
        current: usize,
        total: usize,
    },
    Completed {
        folders_synced: usize,
        emails_synced: usize,
    },
    Error {
        message: String,
    },
}

/// Event emitted when account credentials are missing or invalid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialsRequiredEvent {
    pub account_id: Uuid,
    pub provider: String,
    pub reason: String,
}

/// Helper to emit events to the frontend
pub fn emit_event<T: Serialize + Clone>(
    app_handle: &tauri::AppHandle,
    event_name: &str,
    payload: T,
) {
    if let Err(e) = app_handle.emit(event_name, payload) {
        log::error!("Failed to emit event '{}': {}", event_name, e);
    }
}
