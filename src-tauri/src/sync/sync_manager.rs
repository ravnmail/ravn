use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::auth::CredentialStore;
use super::email_sync::EmailSync;
use super::error::{SyncError, SyncResult};
use super::events::*;
use super::folder_sync::FolderSync;
use super::types::SyncFolder;
use crate::config::Settings;
use crate::database::error::DatabaseError;
use crate::database::models::account::Account;
use crate::database::models::pending_operation::{PendingOperation, PendingOperationType};
use crate::database::repositories::{
    EmailRepository, FolderRepository, SqliteEmailRepository, SqliteFolderRepository,
    SqlitePendingOperationRepository,
};
use crate::search::SearchManager;
use crate::services::notification_service::NotificationService;

/// Central sync manager that coordinates all sync operations
pub struct SyncManager {
    pool: SqlitePool,
    app_data_dir: String,
    folder_sync: Arc<FolderSync>,
    email_sync: Arc<EmailSync>,
    credential_store: Arc<CredentialStore>,
    search_manager: Option<Arc<SearchManager>>,
    settings: Option<Arc<Settings>>,
    notification_service: Option<Arc<NotificationService>>,
    active_syncs: Arc<RwLock<HashMap<Uuid, bool>>>,
    app_handle: Option<tauri::AppHandle>,
}

impl SyncManager {
    pub fn new(
        pool: SqlitePool,
        app_data_dir: String,
        credential_store: Arc<CredentialStore>,
    ) -> Self {
        let folder_sync = Arc::new(FolderSync::new(pool.clone(), Arc::clone(&credential_store)));
        let email_sync = Arc::new(EmailSync::new(
            pool.clone(),
            app_data_dir.clone(),
            Arc::clone(&credential_store),
        ));

        Self {
            pool: pool.clone(),
            app_data_dir,
            folder_sync,
            email_sync,
            credential_store,
            search_manager: None,
            settings: None,
            notification_service: None,
            active_syncs: Arc::new(RwLock::new(HashMap::new())),
            app_handle: None,
        }
    }

    pub fn with_search_manager(mut self, search_manager: Arc<SearchManager>) -> Self {
        self.search_manager = Some(Arc::clone(&search_manager));

        let mut email_sync_builder = EmailSync::new(
            self.pool.clone(),
            self.app_data_dir.clone(),
            Arc::clone(&self.credential_store),
        )
        .with_search_manager(search_manager);

        if let Some(app_handle) = &self.app_handle {
            email_sync_builder = email_sync_builder.with_app_handle(app_handle.clone());
        }

        if let Some(notification_service) = &self.notification_service {
            email_sync_builder =
                email_sync_builder.with_notification_service(Arc::clone(notification_service));
        } else if let (Some(app_handle), Some(settings)) = (&self.app_handle, &self.settings) {
            let notification_service = Arc::new(
                NotificationService::new(self.pool.clone(), Arc::clone(settings))
                    .with_app_handle(app_handle.clone()),
            );
            email_sync_builder = email_sync_builder.with_notification_service(notification_service);
        }

        self.email_sync = Arc::new(email_sync_builder);
        self
    }

    pub fn with_app_handle(mut self, app_handle: tauri::AppHandle) -> Self {
        self.folder_sync = Arc::new(
            FolderSync::new(self.pool.clone(), Arc::clone(&self.credential_store))
                .with_app_handle(app_handle.clone()),
        );

        let mut email_sync_builder = EmailSync::new(
            self.pool.clone(),
            self.app_data_dir.clone(),
            Arc::clone(&self.credential_store),
        )
        .with_app_handle(app_handle.clone());

        if let Some(search_manager) = &self.search_manager {
            email_sync_builder = email_sync_builder.with_search_manager(Arc::clone(search_manager));
        }

        if let Some(notification_service) = &self.notification_service {
            email_sync_builder =
                email_sync_builder.with_notification_service(Arc::clone(notification_service));
        } else if let Some(settings) = &self.settings {
            let notification_service = Arc::new(
                NotificationService::new(self.pool.clone(), Arc::clone(settings))
                    .with_app_handle(app_handle.clone()),
            );
            email_sync_builder = email_sync_builder.with_notification_service(notification_service);
        }

        self.email_sync = Arc::new(email_sync_builder);
        self.app_handle = Some(app_handle);
        self
    }

    pub fn with_settings(mut self, settings: Arc<Settings>) -> Self {
        self.settings = Some(Arc::clone(&settings));

        let mut email_sync_builder = EmailSync::new(
            self.pool.clone(),
            self.app_data_dir.clone(),
            Arc::clone(&self.credential_store),
        );

        if let Some(search_manager) = &self.search_manager {
            email_sync_builder = email_sync_builder.with_search_manager(Arc::clone(search_manager));
        }

        if let Some(app_handle) = &self.app_handle {
            email_sync_builder = email_sync_builder.with_app_handle(app_handle.clone());

            if let Some(notification_service) = &self.notification_service {
                email_sync_builder =
                    email_sync_builder.with_notification_service(Arc::clone(notification_service));
            } else {
                let notification_service = Arc::new(
                    NotificationService::new(self.pool.clone(), settings)
                        .with_app_handle(app_handle.clone()),
                );
                email_sync_builder =
                    email_sync_builder.with_notification_service(notification_service);
            }
        }

        self.email_sync = Arc::new(email_sync_builder);
        self
    }

    pub fn with_notification_service(
        mut self,
        notification_service: Arc<NotificationService>,
    ) -> Self {
        self.notification_service = Some(Arc::clone(&notification_service));

        let mut email_sync_builder = EmailSync::new(
            self.pool.clone(),
            self.app_data_dir.clone(),
            Arc::clone(&self.credential_store),
        );

        if let Some(search_manager) = &self.search_manager {
            email_sync_builder = email_sync_builder.with_search_manager(Arc::clone(search_manager));
        }

        if let Some(app_handle) = &self.app_handle {
            email_sync_builder = email_sync_builder.with_app_handle(app_handle.clone());
        }

        email_sync_builder = email_sync_builder.with_notification_service(notification_service);

        self.email_sync = Arc::new(email_sync_builder);
        self
    }

    fn emit_event<T: serde::Serialize + Clone>(&self, event_name: &str, payload: T) {
        if let Some(app_handle) = &self.app_handle {
            emit_event(app_handle, event_name, payload);
        }
    }

    /// Check if an account is currently syncing
    pub async fn is_syncing(&self, account_id: Uuid) -> bool {
        let syncs = self.active_syncs.read().await;
        *syncs.get(&account_id).unwrap_or(&false)
    }

    /// Sync all folders and emails for an account
    pub async fn sync_account(&self, account: &Account) -> SyncResult<SyncReport> {
        if self.is_syncing(account.id).await {
            return Err(SyncError::SyncInProgress(format!(
                "Sync already in progress for account {}",
                account.email
            )));
        }

        {
            let mut syncs = self.active_syncs.write().await;
            syncs.insert(account.id, true);
        }

        log::info!(
            "Starting sync for account {}: {}",
            account.id,
            account.email
        );

        let result = self.sync_account_internal(account).await;

        {
            let mut syncs = self.active_syncs.write().await;
            syncs.insert(account.id, false);
        }

        result
    }

    async fn sync_account_internal(&self, account: &Account) -> SyncResult<SyncReport> {
        let mut report = SyncReport::default();

        self.emit_event(
            "sync:status",
            SyncStatusEvent {
                account_id: account.id,
                folder_id: None,
                status: SyncEventStatus::Started,
            },
        );

        // Step 1: Sync folders
        log::info!("Syncing folders for account {}", account.id);
        let folders = match self.folder_sync.sync_folders(account).await {
            Ok(folders) => {
                report.folders_synced = folders.len();

                self.emit_event(
                    "sync:folders-updated",
                    FoldersUpdatedEvent {
                        account_id: account.id,
                        folders: folders.clone(),
                    },
                );

                folders
            }
            Err(e) => {
                log::error!("Failed to sync folders: {}", e);
                report.errors.push(format!("Folder sync failed: {}", e));

                self.emit_event(
                    "sync:status",
                    SyncStatusEvent {
                        account_id: account.id,
                        folder_id: None,
                        status: SyncEventStatus::Error {
                            message: e.to_string(),
                        },
                    },
                );

                return Ok(report);
            }
        };

        let _ = sqlx::query!(
            "UPDATE sync_state set sync_status = 'idle'
                  WHERE account_id = ?
                  and sync_status = 'syncing'",
            account.id
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError);

        // Step 2: Sync emails for each folder (prioritize by lowest sync_interval)
        let mut sorted_folders = folders.clone();
        sorted_folders.sort_by_key(|folder| folder.sync_interval);

        for folder in &sorted_folders {
            if matches!(
                folder.folder_type,
                super::types::FolderType::Trash | super::types::FolderType::Spam
            ) {
                log::debug!("Skipping auto-sync for folder: {}", folder.name);
                continue;
            }

            log::info!("Syncing emails for folder: {}", folder.name);

            match self.email_sync.sync_folder(account, folder, false).await {
                Ok(count) => {
                    report.emails_synced += count;
                    log::info!("Synced {} emails in folder {}", count, folder.name);

                    // Todo: Emit event for folder sync completion if needed
                }
                Err(e) => {
                    log::error!("Failed to sync folder {}: {}", folder.name, e);
                    report
                        .errors
                        .push(format!("Folder {} sync failed: {}", folder.name, e));
                }
            }
        }

        log::info!(
            "Sync complete for account {}: {} folders, {} emails",
            account.id,
            report.folders_synced,
            report.emails_synced
        );

        self.emit_event(
            "sync:status",
            SyncStatusEvent {
                account_id: account.id,
                folder_id: None,
                status: SyncEventStatus::Completed {
                    folders_synced: report.folders_synced,
                    emails_synced: report.emails_synced,
                },
            },
        );

        Ok(report)
    }

    /// Sync a specific folder
    ///
    /// # Arguments
    /// * `account` - The email account
    /// * `folder` - The folder to sync
    /// * `full` - If true, forces a full sync instead of delta sync (default: false)
    ///   - When false: fetches up to 50 emails per sync (incremental)
    ///   - When true: fetches all emails from folder
    pub async fn sync_folder(
        &self,
        account: &Account,
        folder: &SyncFolder,
        full: bool,
    ) -> SyncResult<usize> {
        let count = self.email_sync.sync_folder(account, folder, full).await?;

        if let Some(folder_id) = folder.id {
            self.emit_event(
                "sync:folder-counts-updated",
                FolderCountsUpdatedEvent {
                    account_id: account.id,
                    folder_id,
                    unread_count: folder.unread_count,
                    total_count: folder.total_count,
                },
            );
        }

        Ok(count)
    }

    /// Get folders for an account
    pub async fn get_folders(&self, account_id: Uuid) -> SyncResult<Vec<SyncFolder>> {
        self.folder_sync.get_folders(account_id).await
    }

    /// Move an email between folders (local-first: updates DB immediately, queues provider sync)
    pub async fn move_email(
        &self,
        account: &Account,
        email_id: Uuid,
        to_folder_id: Uuid,
    ) -> SyncResult<()> {
        let email_repo = SqliteEmailRepository::new(self.pool.clone());
        let pending_repo = SqlitePendingOperationRepository::new(self.pool.clone());

        let (from_folder_id, remote_id) = email_repo
            .find_for_remote_operation(email_id)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
            .ok_or_else(|| SyncError::EmailNotFound(format!("Email not found: {}", email_id)))?;

        // 1. Optimistic local update
        email_repo
            .update_folder(email_id, to_folder_id)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        // 2. Queue provider operation
        let op = PendingOperation::new(
            account.id,
            Some(email_id),
            Some(from_folder_id),
            PendingOperationType::Move,
            serde_json::json!({
                "remote_id": remote_id,
                "folder_id": from_folder_id.to_string(),
                "to_folder_id": to_folder_id.to_string(),
            }),
        );
        let _ = pending_repo
            .create(&op)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()));

        log::info!(
            "Queued move for email {} to folder {}",
            email_id,
            to_folder_id
        );

        // 3. Emit event immediately
        self.emit_event(
            "sync:email-moved",
            EmailMovedEvent {
                account_id: account.id,
                email_id,
                from_folder_id,
                to_folder_id,
            },
        );

        Ok(())
    }

    /// Delete an email (local-first: updates DB immediately, queues provider sync)
    pub async fn delete_email(
        &self,
        account: &Account,
        email_id: Uuid,
        permanent: bool,
    ) -> SyncResult<()> {
        let email_repo = SqliteEmailRepository::new(self.pool.clone());
        let pending_repo = SqlitePendingOperationRepository::new(self.pool.clone());

        let (folder_id, remote_id) = email_repo
            .find_for_remote_operation(email_id)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
            .ok_or_else(|| SyncError::EmailNotFound(format!("Email not found: {}", email_id)))?;

        // 1. Optimistic local update
        if permanent {
            email_repo
                .delete(email_id)
                .await
                .map_err(|e| SyncError::DatabaseError(e.to_string()))?;
        } else {
            email_repo
                .soft_delete(email_id)
                .await
                .map_err(|e| SyncError::DatabaseError(e.to_string()))?;
        }

        // 2. Queue provider operation
        let op_type = if permanent {
            PendingOperationType::PermanentDelete
        } else {
            PendingOperationType::Delete
        };
        let op = PendingOperation::new(
            account.id,
            Some(email_id),
            Some(folder_id),
            op_type,
            serde_json::json!({
                "remote_id": remote_id,
                "folder_id": folder_id.to_string(),
            }),
        );
        let _ = pending_repo
            .create(&op)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()));

        log::info!(
            "Queued {} for email {}",
            if permanent {
                "permanent delete"
            } else {
                "delete"
            },
            email_id
        );

        // 3. Emit event immediately
        self.emit_event(
            "sync:email-deleted",
            EmailDeletedEvent {
                account_id: account.id,
                email_id,
                folder_id,
                permanent,
            },
        );

        Ok(())
    }

    /// Mark email as read/unread (local-first: updates DB immediately, queues provider sync)
    pub async fn mark_as_read(
        &self,
        account: &Account,
        email_id: Uuid,
        is_read: bool,
    ) -> SyncResult<()> {
        let email_repo = SqliteEmailRepository::new(self.pool.clone());
        let pending_repo = SqlitePendingOperationRepository::new(self.pool.clone());

        let (folder_id, remote_id) = email_repo
            .find_for_remote_operation(email_id)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
            .ok_or_else(|| SyncError::EmailNotFound(format!("Email not found: {}", email_id)))?;

        // 1. Optimistic local update
        email_repo
            .update_read_status(email_id, is_read)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        // 2. Queue provider operation
        let op_type = if is_read {
            PendingOperationType::MarkRead
        } else {
            PendingOperationType::MarkUnread
        };
        let op = PendingOperation::new(
            account.id,
            Some(email_id),
            Some(folder_id),
            op_type,
            serde_json::json!({
                "remote_id": remote_id,
                "folder_id": folder_id.to_string(),
            }),
        );
        let _ = pending_repo
            .create(&op)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()));

        log::info!("Queued mark_as_read={} for email {}", is_read, email_id);

        // 3. Emit event immediately
        self.emit_event(
            "sync:email-read-status-changed",
            EmailReadStatusChangedEvent {
                account_id: account.id,
                email_id,
                folder_id,
                is_read,
            },
        );

        if let Some(notification_service) = &self.notification_service {
            notification_service
                .update_badge_count()
                .await
                .map_err(SyncError::InvalidConfiguration)?;
        }

        Ok(())
    }

    /// Flag/unflag an email (local-first: updates DB immediately, queues provider sync)
    pub async fn set_flag(
        &self,
        account: &Account,
        email_id: Uuid,
        flagged: bool,
    ) -> SyncResult<()> {
        let email_repo = SqliteEmailRepository::new(self.pool.clone());
        let pending_repo = SqlitePendingOperationRepository::new(self.pool.clone());

        let (folder_id, remote_id) = email_repo
            .find_for_remote_operation(email_id)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
            .ok_or_else(|| SyncError::EmailNotFound(format!("Email not found: {}", email_id)))?;

        // 1. Optimistic local update
        email_repo
            .update_flagged_status(email_id, flagged)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        // 2. Queue provider operation
        let op_type = if flagged {
            PendingOperationType::Flag
        } else {
            PendingOperationType::Unflag
        };
        let op = PendingOperation::new(
            account.id,
            Some(email_id),
            Some(folder_id),
            op_type,
            serde_json::json!({
                "remote_id": remote_id,
                "folder_id": folder_id.to_string(),
            }),
        );
        let _ = pending_repo
            .create(&op)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()));

        log::info!("Queued set_flag={} for email {}", flagged, email_id);

        // 3. Emit event immediately
        self.emit_event(
            "sync:email-flag-changed",
            EmailFlagChangedEvent {
                account_id: account.id,
                email_id,
                folder_id,
                is_flagged: flagged,
            },
        );

        Ok(())
    }

    /// Rename a folder and sync to provider
    pub async fn rename_folder(
        &self,
        account: &Account,
        folder_id: Uuid,
        old_name: &str,
        new_name: &str,
    ) -> SyncResult<()> {
        let folder = self.get_folder_by_id(folder_id).await?;

        let provider = super::provider::ProviderFactory::create_with_app_handle(
            account,
            Arc::clone(&self.credential_store),
            self.app_handle.clone(),
        )?;
        provider.rename_folder(&folder, new_name).await?;

        log::info!(
            "Renamed folder {} from '{}' to '{}'",
            folder_id,
            old_name,
            new_name
        );

        self.emit_event(
            "sync:folder-renamed",
            FolderRenamedEvent {
                account_id: account.id,
                folder_id,
                old_name: old_name.to_string(),
                new_name: new_name.to_string(),
            },
        );

        Ok(())
    }

    /// Move a folder to a new parent and sync to provider
    pub async fn move_folder(
        &self,
        account: &Account,
        folder_id: Uuid,
        old_parent_id: Option<Uuid>,
        new_parent_id: Option<Uuid>,
    ) -> SyncResult<()> {
        let folder = self.get_folder_by_id(folder_id).await?;

        let new_parent_path = if let Some(parent_id) = new_parent_id {
            let parent_folder = self.get_folder_by_id(parent_id).await?;
            Some(parent_folder.remote_id.clone())
        } else {
            None
        };

        let provider = super::provider::ProviderFactory::create_with_app_handle(
            account,
            Arc::clone(&self.credential_store),
            self.app_handle.clone(),
        )?;
        provider
            .move_folder(&folder, new_parent_path.as_deref())
            .await?;

        log::info!("Moved folder {} to parent {:?}", folder_id, new_parent_id);

        self.emit_event(
            "sync:folder-moved",
            FolderMovedEvent {
                account_id: account.id,
                folder_id,
                old_parent_id,
                new_parent_id,
            },
        );

        Ok(())
    }

    async fn get_folder_by_id(&self, folder_id: Uuid) -> SyncResult<SyncFolder> {
        let folder_repo = SqliteFolderRepository::new(self.pool.clone());
        let folder = folder_repo
            .find_by_id(folder_id)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
            .ok_or_else(|| SyncError::DatabaseError(format!("Folder not found: {}", folder_id)))?;

        Ok(SyncFolder {
            id: Some(folder.id),
            account_id: folder.account_id,
            name: folder.name,
            folder_type: folder.folder_type,
            remote_id: folder.remote_id.unwrap_or_default(),
            parent_id: folder.parent_id,
            icon: folder.icon,
            color: folder.color,
            sync_interval: folder.sync_interval,
            synced_at: Some(folder.synced_at),
            attributes: Vec::new(),
            unread_count: folder.unread_count as i32,
            total_count: folder.total_count as i32,
            expanded: folder.expanded,
            hidden: folder.hidden,
        })
    }
}

#[derive(Debug, Default)]
pub struct SyncReport {
    pub folders_synced: usize,
    pub emails_synced: usize,
    pub errors: Vec<String>,
}
