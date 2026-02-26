use crate::database::models::pending_operation::PendingOperationType;
use crate::database::repositories::{
    AccountRepository, RepositoryFactory, SqlitePendingOperationRepository,
};
use crate::sync::auth::CredentialStore;
use crate::sync::error::{SyncError, SyncResult};
use crate::sync::events;
use crate::sync::provider::ProviderFactory;
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

/// Background processor for pending email operations (mark read, move, delete, etc.)
///
/// Processes operations asynchronously after they've been optimistically applied locally.
/// Handles retries for transient errors and reports failures to the UI.
pub struct OperationQueue {
    pool: SqlitePool,
    credential_store: Arc<CredentialStore>,
    app_handle: Option<tauri::AppHandle>,
}

impl OperationQueue {
    pub fn new(pool: SqlitePool, credential_store: Arc<CredentialStore>) -> Self {
        Self {
            pool,
            credential_store,
            app_handle: None,
        }
    }

    pub fn with_app_handle(mut self, app_handle: tauri::AppHandle) -> Self {
        self.app_handle = Some(app_handle);
        self
    }

    /// Start the operation queue processor as a background task.
    /// Spawns a task that runs for the lifetime of the application.
    pub fn start(self) {
        log::info!("[OperationQueue] Starting background operation queue");

        tauri::async_runtime::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                if let Err(e) = self.process_pending_operations().await {
                    log::error!("[OperationQueue] Error processing operations: {}", e);
                }
            }
        });
    }

    /// Process all pending operations across all accounts
    async fn process_pending_operations(&self) -> SyncResult<()> {
        let repo = SqlitePendingOperationRepository::new(self.pool.clone());

        let account_ids = repo
            .find_accounts_with_pending_ops()
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        for account_id in account_ids {
            if let Err(e) = self.process_account_operations(account_id).await {
                log::error!(
                    "[OperationQueue] Error processing operations for account {}: {}",
                    account_id,
                    e
                );
            }
        }

        Ok(())
    }

    /// Process all pending operations for a single account
    async fn process_account_operations(&self, account_id: Uuid) -> SyncResult<()> {
        let repo_factory = RepositoryFactory::new(self.pool.clone());
        let pending_repo = repo_factory.pending_operation_repository();
        let account_repo = repo_factory.account_repository();

        let account = account_repo
            .find_by_id(account_id)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
            .ok_or_else(|| {
                SyncError::DatabaseError(format!("Account not found: {}", account_id))
            })?;

        let operations = pending_repo
            .find_pending_by_account(account_id)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        if operations.is_empty() {
            return Ok(());
        }

        log::info!(
            "[OperationQueue] Processing {} operations for account {}",
            operations.len(),
            account_id
        );

        // Create provider once for all operations in this batch
        let provider = ProviderFactory::create_with_app_handle(
            &account,
            Arc::clone(&self.credential_store),
            self.app_handle.clone(),
        )?;

        // Load and authenticate credentials
        let credentials = self.load_credentials(&account).await?;
        let mut provider = provider;
        provider.authenticate(credentials).await?;

        for op in operations {
            let op_id = op.id;
            let op_type = op.operation_type.clone();

            // Mark as in progress
            let _ = pending_repo.mark_in_progress(op_id).await;

            let payload = op.parsed_payload();
            let result = self
                .execute_operation(&*provider, &op_type, &payload)
                .await;

            match result {
                Ok(()) => {
                    log::debug!(
                        "[OperationQueue] Operation {} ({}) completed successfully",
                        op_id,
                        op_type
                    );
                    let _ = pending_repo.mark_completed(op_id).await;
                }
                Err(e) => {
                    let error_msg = e.to_string();

                    // Treat 404 (resource not found) as success — the message no longer
                    // exists on the server, so the operation is moot.
                    if error_msg.contains("404") || error_msg.contains("Not Found") {
                        log::info!(
                            "[OperationQueue] Operation {} ({}) target not found on server, marking completed",
                            op_id,
                            op_type
                        );
                        let _ = pending_repo.mark_completed(op_id).await;
                        continue;
                    }

                    let is_retryable = e.is_retryable();

                    log::warn!(
                        "[OperationQueue] Operation {} ({}) failed: {} (retryable: {})",
                        op_id,
                        op_type,
                        error_msg,
                        is_retryable
                    );

                    let _ = pending_repo.mark_failed(op_id, &error_msg).await;

                    if is_retryable && op.retry_count < op.max_retries {
                        // Reset for retry
                        let _ = pending_repo.reset_for_retry(op_id).await;
                    } else {
                        // Emit failure event to frontend
                        if let Some(app_handle) = &self.app_handle {
                            events::emit_event(
                                app_handle,
                                "sync:operation-failed",
                                events::OperationFailedEvent {
                                    account_id,
                                    operation_id: op_id,
                                    email_id: op.email_id,
                                    operation_type: op_type.clone(),
                                    error: error_msg,
                                },
                            );
                        }

                        // Don't continue processing for this account on non-retryable errors
                        if !is_retryable {
                            log::error!(
                                "[OperationQueue] Non-retryable error for account {}, pausing queue",
                                account_id
                            );
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Execute a single operation against the provider
    async fn execute_operation(
        &self,
        provider: &dyn crate::sync::provider::EmailProvider,
        operation_type: &str,
        payload: &serde_json::Value,
    ) -> SyncResult<()> {
        let remote_id = payload
            .get("remote_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        // Build a SyncFolder from payload data
        let folder = self.folder_from_payload(payload).await?;

        match PendingOperationType::from_str(operation_type) {
            Some(PendingOperationType::MarkRead) => {
                provider.mark_as_read(remote_id, &folder, true).await
            }
            Some(PendingOperationType::MarkUnread) => {
                provider.mark_as_read(remote_id, &folder, false).await
            }
            Some(PendingOperationType::Flag) => {
                provider.set_flag(remote_id, &folder, true).await
            }
            Some(PendingOperationType::Unflag) => {
                provider.set_flag(remote_id, &folder, false).await
            }
            Some(PendingOperationType::Move) => {
                let to_folder_id_str = payload
                    .get("to_folder_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let to_folder_id = Uuid::parse_str(to_folder_id_str)
                    .map_err(|e| SyncError::DatabaseError(e.to_string()))?;
                let to_folder = self.get_folder_by_id(to_folder_id).await?;
                provider.move_email(remote_id, &folder, &to_folder).await
            }
            Some(PendingOperationType::Delete) => {
                provider.delete_email(remote_id, &folder, false).await
            }
            Some(PendingOperationType::PermanentDelete) => {
                provider.delete_email(remote_id, &folder, true).await
            }
            _ => {
                log::warn!(
                    "[OperationQueue] Unsupported operation type: {}",
                    operation_type
                );
                Ok(())
            }
        }
    }

    /// Build a SyncFolder from the payload's folder_id
    async fn folder_from_payload(
        &self,
        payload: &serde_json::Value,
    ) -> SyncResult<crate::sync::types::SyncFolder> {
        let folder_id_str = payload
            .get("folder_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let folder_id = Uuid::parse_str(folder_id_str)
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;
        self.get_folder_by_id(folder_id).await
    }

    async fn get_folder_by_id(
        &self,
        folder_id: Uuid,
    ) -> SyncResult<crate::sync::types::SyncFolder> {
        use crate::database::repositories::{FolderRepository, SqliteFolderRepository};

        let folder_repo = SqliteFolderRepository::new(self.pool.clone());
        let folder = folder_repo
            .find_by_id(folder_id)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
            .ok_or_else(|| SyncError::FolderNotFound(format!("Folder not found: {}", folder_id)))?;

        Ok(crate::sync::types::SyncFolder {
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

    async fn load_credentials(
        &self,
        account: &crate::database::models::account::Account,
    ) -> SyncResult<crate::sync::types::ProviderCredentials> {
        if !self.credential_store.has_credentials(account.id).await {
            return Err(SyncError::InvalidConfiguration(format!(
                "No credentials found for account {}",
                account.id
            )));
        }

        match account.account_type.as_str() {
            "gmail" | "office365" => {
                let oauth_creds = self.credential_store.get_oauth2(account.id).await?;
                Ok(crate::sync::types::ProviderCredentials::OAuth2(oauth_creds))
            }
            "imap" | "apple" => {
                let imap_creds = self.credential_store.get_imap(account.id).await?;
                Ok(crate::sync::types::ProviderCredentials::Imap(imap_creds))
            }
            _ => Err(SyncError::NotSupported(format!(
                "Account type {} not supported",
                account.account_type
            ))),
        }
    }
}
