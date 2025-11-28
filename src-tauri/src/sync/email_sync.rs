use super::attachment_handler::AttachmentHandler;
use super::auth::CredentialStore;
use super::contact_extractor::ContactExtractor;
use super::email_body_splitter::EmailBodySplitter;
use super::email_categorizer::EmailCategorizer;
use super::error::{SyncError, SyncResult};
use super::provider::ProviderFactory;
use super::storage::LocalFileStorage;
use super::types::{ProviderCredentials, SyncEmail, SyncFolder};
use crate::config::Settings;
use crate::database::models::account::{Account, AccountType};
use crate::database::models::email::EmailAddress;
use crate::database::repositories::EmailRepository;
use crate::database::repositories::RepositoryFactory;
use crate::search::SearchManager;
use crate::services::notification_service::NotificationService;
use crate::sync::conversion_mode::EmailConversionMode;
use chrono::{DateTime, Utc};
use html_to_markdown_rs::{convert, ConversionOptions, PreprocessingOptions, PreprocessingPreset};
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Emitter;
use uuid::Uuid;

pub struct EmailSync {
    pool: SqlitePool,
    attachment_handler: AttachmentHandler<LocalFileStorage>,
    credential_store: Arc<CredentialStore>,
    contact_extractor: Arc<ContactExtractor>,
    search_manager: Option<Arc<SearchManager>>,
    app_handle: Option<tauri::AppHandle>,
    notification_service: Option<Arc<NotificationService>>,
    settings: Arc<Settings>,
}

fn emit_folder_event<S: serde::Serialize + Clone>(
    app_handle: &tauri::AppHandle,
    event_name: &str,
    payload: S,
) {
    if let Err(e) = app_handle.emit(event_name, payload) {
        log::error!("Failed to emit folder event '{}': {}", event_name, e);
    }
}

impl EmailSync {
    pub fn new(
        pool: SqlitePool,
        app_data_dir: String,
        credential_store: Arc<CredentialStore>,
        settings: Arc<Settings>,
    ) -> Self {
        let cache_dir = PathBuf::from(app_data_dir).join("attachments");
        let storage = Arc::new(LocalFileStorage::new(cache_dir));

        let repo_factory = RepositoryFactory::new(pool.clone());
        let contact_repo = Arc::new(repo_factory.contact_repository());
        let contact_extractor = Arc::new(ContactExtractor::new(contact_repo));

        Self {
            attachment_handler: AttachmentHandler::new(pool.clone(), storage),
            pool,
            credential_store,
            contact_extractor,
            search_manager: None,
            app_handle: None,
            notification_service: None,
            settings,
        }
    }

    pub fn with_search_manager(mut self, search_manager: Arc<SearchManager>) -> Self {
        self.search_manager = Some(search_manager);
        self
    }

    pub fn with_app_handle(mut self, app_handle: tauri::AppHandle) -> Self {
        self.app_handle = Some(app_handle);
        self
    }

    pub fn with_notification_service(
        mut self,
        notification_service: Arc<NotificationService>,
    ) -> Self {
        self.notification_service = Some(notification_service);
        self
    }

    /// Synchronize emails for a folder using provider-agnostic delta/full sync
    ///
    /// # Arguments
    /// * `account` - The email account
    /// * `folder` - The folder to sync
    /// * `full` - If true, forces a full sync instead of delta sync
    pub async fn sync_folder(
        &self,
        account: &Account,
        folder: &SyncFolder,
        full: bool,
    ) -> SyncResult<usize> {
        let sync_type = if full { "full" } else { "incremental" };
        log::info!(
            "[EmailSync] Starting {} sync for folder {} (account {})",
            sync_type,
            folder.name,
            account.id
        );

        // Check if sync is already in progress - prevent concurrent syncs for same folder
        let current_status = self.get_sync_status(folder).await?;
        log::info!(
            "[EmailSync] Current sync status for folder {} (id {}): {}",
            folder.name,
            folder.id.unwrap(),
            current_status
        );
        if current_status == "syncing" {
            log::warn!(
                "[EmailSync] Sync already in progress for folder {} (account {}), skipping",
                folder.name,
                account.id
            );
            return Err(SyncError::SyncInProgress(format!(
                "Sync already in progress for folder {}",
                folder.name
            )));
        }

        // Set status to syncing
        self.set_sync_status(folder, "syncing").await?;

        // Wrap sync logic in a result handler to set status to idle/error on completion
        let result = self.sync_folder_internal(account, folder, full).await;

        // Update status based on result
        if result.is_ok() {
            let _ = self.set_sync_status(folder, "idle").await;
        } else {
            let _ = self.set_sync_status(folder, "error").await;
        }

        // emit event folder:updated
        emit_folder_event(
            &self.app_handle.as_ref().unwrap(),
            "folder:updated",
            serde_json::json!(folder),
        );

        result
    }

    /// Internal sync logic (extracted to allow status management in outer method)
    async fn sync_folder_internal(
        &self,
        account: &Account,
        folder: &SyncFolder,
        full: bool,
    ) -> SyncResult<usize> {
        let sync_type = if full { "full" } else { "incremental" };

        let mut provider = ProviderFactory::create_with_app_handle(
            account,
            Arc::clone(&self.credential_store),
            self.app_handle.clone(),
        )?;

        let credentials = self.load_credentials(account).await?;
        provider.authenticate(credentials).await?;

        // For full sync, get local emails upfront for deletion computation
        let local_remote_ids = if full {
            self.get_existing_remote_ids_for_folder(folder).await?
        } else {
            std::collections::HashSet::new()
        };

        let mut total_added = 0;
        let mut total_modified = 0;
        let mut total_deleted = 0;
        let mut provider_remote_ids: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        let mut next_sync_token: Option<String> = None;

        // Try to use Office365 paged processing for better interactivity
        if account.account_type == AccountType::Office365 {
            if let Some(o365_provider) = provider
                .as_any()
                .downcast_ref::<crate::sync::providers::office365::Office365Provider>(
            ) {
                // Get sync token for delta sync
                let sync_token = if !full {
                    self.get_sync_token(folder).await.ok().flatten()
                } else {
                    None
                };

                // Use paged processing - process each page immediately as it arrives
                if let Some(token) = sync_token {
                    // Delta sync with paging
                    let account_id = account.id;
                    let folder_clone = folder.clone();
                    let self_ptr = self.clone();

                    if let Ok((new_token, deleted_ids)) = o365_provider
                        .fetch_emails_delta_paged(folder, &token, move |page_emails| {
                            let sync_inner = self_ptr.clone();
                            let folder_inner = folder_clone.clone();

                            async move {
                                // Store each email from this page immediately
                                for email in &page_emails {
                                    let _ =
                                        sync_inner.upsert_email(email, account_id, "synced").await;
                                }
                                // Store sync state after each page for resilience
                                let _ = sync_inner.update_sync_state(&folder_inner).await;
                                Ok(())
                            }
                        })
                        .await
                    {
                        next_sync_token = new_token;
                        // Process deleted emails from delta response
                        total_deleted = self
                            .process_deleted_emails(&deleted_ids, folder)
                            .await
                            .unwrap_or(0);
                    }
                } else {
                    // Full sync with paging
                    let account_id = account.id;
                    let folder_clone = folder.clone();
                    let self_ptr = self.clone();

                    if let Ok(new_token) = o365_provider
                        .fetch_emails_full_paged(folder, move |page_emails| {
                            let sync_inner = self_ptr.clone();
                            let folder_inner = folder_clone.clone();

                            async move {
                                // Store each email from this page immediately
                                for email in &page_emails {
                                    let _ =
                                        sync_inner.upsert_email(email, account_id, "synced").await;
                                }
                                // Store sync state after each page for resilience
                                let _ = sync_inner.update_sync_state(&folder_inner).await;
                                Ok(())
                            }
                        })
                        .await
                    {
                        next_sync_token = new_token;
                        // After paging completes, get all provider remote IDs from DB
                        provider_remote_ids = self
                            .get_existing_remote_ids_for_folder(folder)
                            .await
                            .unwrap_or_default();
                        // Set total_added to the count of emails from this sync
                        total_added = provider_remote_ids.len();
                    }
                }

                // Compute deletions for full sync
                if full {
                    total_deleted = self
                        .process_deleted_emails(
                            &local_remote_ids
                                .iter()
                                .filter(|id| !provider_remote_ids.contains(*id))
                                .cloned()
                                .collect::<Vec<_>>(),
                            folder,
                        )
                        .await
                        .unwrap_or(0);

                    log::info!(
                        "[EmailSync] Full sync: {} local emails, {} from provider, {} deletions",
                        local_remote_ids.len(),
                        provider_remote_ids.len(),
                        total_deleted
                    );
                }
            } else {
                // Fallback to non-paged processing
                return self.sync_folder_legacy(account, folder, full).await;
            }
        } else {
            // Use legacy paging for non-Office365 providers
            return self.sync_folder_legacy(account, folder, full).await;
        }

        let total = total_added + total_modified + total_deleted;

        // Store final sync token
        if let Some(token) = next_sync_token {
            self.store_sync_token(folder, &token).await.ok();
        }

        // Update sync state and commit search indexer
        self.update_sync_state(folder).await?;
        self.update_folder_synced_at(folder).await?;
        self.commit_search_index().await?;

        log::info!(
            "[EmailSync] Completed {} sync: +{} ~{} -{} (total: {})",
            sync_type,
            total_added,
            total_modified,
            total_deleted,
            total
        );

        Ok(total)
    }

    /// Legacy sync folder implementation (without paging)
    async fn sync_folder_legacy(
        &self,
        account: &Account,
        folder: &SyncFolder,
        full: bool,
    ) -> SyncResult<usize> {
        let mut provider = ProviderFactory::create_with_app_handle(
            account,
            Arc::clone(&self.credential_store),
            self.app_handle.clone(),
        )?;

        let credentials = self.load_credentials(account).await?;
        provider.authenticate(credentials).await?;

        // Get sync token for delta sync (if not forcing full sync)
        let sync_token = if !full {
            self.get_sync_token(folder).await.ok().flatten()
        } else {
            None
        };

        // Get provider's view of the folder
        let mut diff = provider.sync_messages(folder, sync_token).await?;

        // For full sync, compute deletions by comparing local emails with provider's additions
        if full {
            let local_remote_ids = self.get_existing_remote_ids_for_folder(folder).await?;
            let provider_remote_ids: std::collections::HashSet<_> =
                diff.added.iter().map(|e| e.remote_id.clone()).collect();

            diff.deleted = local_remote_ids
                .iter()
                .filter(|id| !provider_remote_ids.contains(*id))
                .cloned()
                .collect();

            log::info!(
                "[EmailSync] Full sync: {} local emails, {} from provider, {} deletions",
                local_remote_ids.len(),
                provider_remote_ids.len(),
                diff.deleted.len()
            );
        }

        // Process changes
        let added_count = self
            .process_added_emails(&diff.added, account.id, folder)
            .await?;
        let modified_count = self
            .process_modified_emails(&diff.modified, account.id, folder)
            .await?;
        let deleted_count = self.process_deleted_emails(&diff.deleted, folder).await?;

        let total = added_count + modified_count + deleted_count;

        // Store next sync token
        if let Some(token) = &diff.next_sync_token {
            self.store_sync_token(folder, token).await.ok();
        }

        // Update sync state and commit search indexer
        self.update_sync_state(folder).await?;
        self.update_folder_synced_at(folder).await?;
        self.commit_search_index().await?;

        log::info!(
            "[EmailSync] Completed legacy sync: +{} ~{} -{} (total: {})",
            added_count,
            modified_count,
            deleted_count,
            total
        );

        Ok(total)
    }

    async fn process_added_emails(
        &self,
        emails: &[SyncEmail],
        account_id: Uuid,
        _folder: &SyncFolder,
    ) -> SyncResult<usize> {
        let mut count = 0;
        for email in emails {
            let (_email_id, _inline_attachments, _is_new, _db_email) =
                self.upsert_email(email, account_id, "synced").await?;
            count += 1;
        }
        log::debug!("[EmailSync] Processed {} added emails", count);
        Ok(count)
    }

    async fn process_modified_emails(
        &self,
        emails: &[SyncEmail],
        account_id: Uuid,
        _folder: &SyncFolder,
    ) -> SyncResult<usize> {
        let mut count = 0;
        for email in emails {
            let (_email_id, _inline_attachments, _is_new, _db_email) =
                self.upsert_email(email, account_id, "synced").await?;
            count += 1;
        }
        log::debug!("[EmailSync] Processed {} modified emails", count);
        Ok(count)
    }

    async fn process_deleted_emails(
        &self,
        remote_ids: &[String],
        folder: &SyncFolder,
    ) -> SyncResult<usize> {
        let mut count = 0;
        let folder_id_str = folder.id.unwrap().to_string();

        for remote_id in remote_ids {
            sqlx::query!(
                "UPDATE emails SET is_deleted = 1 WHERE folder_id = ? AND remote_id = ?",
                folder_id_str,
                remote_id
            )
            .execute(&self.pool)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

            count += 1;
        }

        log::debug!("[EmailSync] Marked {} emails as deleted", count);
        Ok(count)
    }

    async fn commit_search_index(&self) -> SyncResult<()> {
        if let Some(search_manager) = &self.search_manager {
            if let Err(e) = search_manager.commit().await {
                log::warn!("[EmailSync] Failed to commit search indexer: {}", e);
            }
        }
        Ok(())
    }

    /// Get existing remote IDs for a folder
    async fn get_existing_remote_ids_for_folder(
        &self,
        folder: &SyncFolder,
    ) -> SyncResult<std::collections::HashSet<String>> {
        let folder_id_str = folder.id.unwrap().to_string();

        let records = sqlx::query!(
            r#"
            SELECT remote_id
            FROM emails
            WHERE folder_id = ? AND is_deleted = 0 AND remote_id IS NOT NULL
            "#,
            folder_id_str
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        Ok(records.into_iter().filter_map(|r| r.remote_id).collect())
    }

    /// Handle emails that are no longer in the sync results by fetching them individually
    /// Called for both full and incremental syncs, but uses different strategies:
    /// - Full sync: aggressively detects moved and deleted emails
    /// - Incremental sync: conservatively detects moved emails, skips unknown deletions
    /// Respects pagination by only checking emails that could reasonably be affected
    async fn handle_missing_emails(
        &self,
        account: &Account,
        provider: &mut Box<dyn super::provider::EmailProvider>,
        folder: &SyncFolder,
        existing_remote_ids: &std::collections::HashSet<String>,
        synced_remote_ids: &std::collections::HashSet<String>,
        is_full_sync: bool,
    ) -> SyncResult<usize> {
        let account_id_str = account.id.to_string();
        let folder_id_str = folder.id.unwrap().to_string();

        let missing_remote_ids: Vec<String> = existing_remote_ids
            .difference(synced_remote_ids)
            .cloned()
            .collect();

        if missing_remote_ids.is_empty() {
            return Ok(0);
        }

        log::info!(
            "[EmailSync] {} sync detected {} potentially moved/deleted emails in folder {} (out of {} total)",
            if is_full_sync { "Full" } else { "Incremental" },
            missing_remote_ids.len(),
            folder.name,
            existing_remote_ids.len()
        );

        let mut updated_count = 0;
        let mut deleted_count = 0;
        let mut skipped_count = 0;

        for remote_id in missing_remote_ids {
            if remote_id.is_empty() {
                continue;
            }

            match provider.fetch_email(folder, &remote_id).await {
                Ok(email) => {
                    log::debug!(
                        "[EmailSync] Email {} still exists on server in folder {}, updating",
                        remote_id,
                        folder.name
                    );

                    let sync_status = if account.account_type == AccountType::Imap
                        && email.body_plain.is_none()
                        && email.body_html.is_none()
                    {
                        "headers_only"
                    } else {
                        "synced"
                    };

                    let (email_id, uncached_inline_ids, _is_new, _db_email) =
                        self.upsert_email(&email, account.id, sync_status).await?;

                    if sync_status == "synced" && !uncached_inline_ids.is_empty() {
                        for attachment_id in uncached_inline_ids {
                            let attachment = self
                                .attachment_handler
                                .get_attachment_metadata(attachment_id)
                                .await?;

                            if attachment.is_inline {
                                match provider.fetch_attachment(&attachment).await {
                                    Ok(data) => {
                                        self.attachment_handler
                                            .cache_attachment(
                                                attachment_id,
                                                account.id,
                                                email_id,
                                                &data,
                                                &attachment.filename,
                                            )
                                            .await?;
                                    }
                                    Err(e) => {
                                        log::warn!(
                                            "[EmailSync] Failed to fetch inline attachment {} for email {}: {}",
                                            attachment_id,
                                            email_id,
                                            e
                                        );
                                    }
                                }
                            }
                        }
                    }

                    updated_count += 1;
                }
                Err(_) => {
                    let other_folder = sqlx::query!(
                        r#"
                        SELECT id, folder_id
                        FROM emails
                        WHERE account_id = ? AND remote_id = ? AND folder_id != ? AND is_deleted = 0
                        LIMIT 1
                        "#,
                        account_id_str,
                        remote_id,
                        folder_id_str
                    )
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

                    if other_folder.is_some() {
                        log::debug!(
                            "[EmailSync] Email {} was moved to another folder, marking old instance as deleted in {}",
                            remote_id,
                            folder.name
                        );

                        sqlx::query!(
                            r#"
                            UPDATE emails
                            SET is_deleted = 1, updated_at = CURRENT_TIMESTAMP
                            WHERE account_id = ? AND folder_id = ? AND remote_id = ? AND is_deleted = 0
                            "#,
                            account_id_str,
                            folder_id_str,
                            remote_id
                        )
                        .execute(&self.pool)
                        .await
                        .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

                        deleted_count += 1;
                    } else if is_full_sync {
                        log::info!(
                            "[EmailSync] Email {} deleted from server (full sync), marking as deleted in folder {}",
                            remote_id,
                            folder.name
                        );

                        sqlx::query!(
                            r#"
                            UPDATE emails
                            SET is_deleted = 1, updated_at = CURRENT_TIMESTAMP
                            WHERE account_id = ? AND folder_id = ? AND remote_id = ? AND is_deleted = 0
                            "#,
                            account_id_str,
                            folder_id_str,
                            remote_id
                        )
                        .execute(&self.pool)
                        .await
                        .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

                        deleted_count += 1;
                    } else {
                        log::debug!(
                            "[EmailSync] Email {} not found in folder {} (incremental sync), skipping - may be old email or moved to unsynced folder",
                            remote_id,
                            folder.name
                        );
                        skipped_count += 1;
                    }
                }
            }
        }

        if updated_count > 0 {
            log::info!(
                "[EmailSync] Updated {} emails that were found on server",
                updated_count
            );
        }

        if deleted_count > 0 {
            log::info!(
                "[EmailSync] Marked {} emails as deleted in folder {}",
                deleted_count,
                folder.name
            );
        }

        if skipped_count > 0 {
            log::debug!(
                "[EmailSync] Skipped {} potentially old emails in incremental sync of folder {}",
                skipped_count,
                folder.name
            );
        }

        Ok(deleted_count)
    }

    /// Load credentials from keyring based on account type
    async fn load_credentials(&self, account: &Account) -> SyncResult<ProviderCredentials> {
        if !self.credential_store.has_credentials(account.id).await {
            return Err(SyncError::InvalidConfiguration(format!(
                "No credentials found for account {} ({}). Please complete account setup first.",
                account.id, account.email
            )));
        }

        match account.account_type.as_str() {
            "gmail" | "office365" => {
                let oauth_creds = self.credential_store.get_oauth2(account.id).await?;
                Ok(ProviderCredentials::OAuth2(oauth_creds))
            }
            "imap" | "apple" => {
                let imap_creds = self.credential_store.get_imap(account.id).await?;
                Ok(ProviderCredentials::Imap(imap_creds))
            }
            _ => Err(SyncError::NotSupported(format!(
                "Account type {} not supported",
                account.account_type
            ))),
        }
    }

    /// Get last synced UID for a folder
    async fn get_last_synced_uid(&self, folder: &SyncFolder) -> SyncResult<Option<u32>> {
        let folder_id_str = folder.id.unwrap().to_string();
        let record = sqlx::query!(
            "SELECT last_uid FROM sync_state WHERE folder_id = ?",
            folder_id_str
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| super::error::SyncError::DatabaseError(e.to_string()))?;

        Ok(record.and_then(|r| r.last_uid.map(|uid| uid as u32)))
    }

    /// Get sync token (delta link) for Office365 incremental sync
    async fn get_sync_token(&self, folder: &SyncFolder) -> SyncResult<Option<String>> {
        let folder_id_str = folder.id.unwrap().to_string();
        let record = sqlx::query!(
            "SELECT sync_token FROM sync_state WHERE folder_id = ?",
            folder_id_str
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        Ok(record.and_then(|r| r.sync_token))
    }

    /// Get current sync status to prevent concurrent syncs
    async fn get_sync_status(&self, folder: &SyncFolder) -> SyncResult<String> {
        let folder_id_str = folder.id.unwrap().to_string();
        let record = sqlx::query!(
            "SELECT sync_status FROM sync_state WHERE folder_id = ?",
            folder_id_str
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        // Default to 'idle' if no record exists
        Ok(record
            .map(|r| r.sync_status)
            .unwrap_or_else(|| "idle".to_string()))
    }

    /// Set sync status (syncing, idle, error, paused)
    async fn set_sync_status(&self, folder: &SyncFolder, status: &str) -> SyncResult<()> {
        let id = Uuid::now_v7().to_string();
        let account_id_str = folder.account_id.to_string();
        let folder_id_str = folder.id.unwrap().to_string();

        let x = sqlx::query!(
            r#"
            INSERT INTO sync_state (id, account_id, folder_id, sync_status)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(account_id, folder_id)
            DO UPDATE SET
                sync_status = ?
            "#,
            id,
            account_id_str,
            folder_id_str,
            status,
            status
        )
        .execute(&self.pool)
        .await
        .map_err(|e| super::error::SyncError::DatabaseError(e.to_string()))?;

        log::info!(
            "[EmailSync] Set sync status for folder {} (account {}) to {}, {}",
            folder.name,
            folder.account_id,
            status,
            x.rows_affected()
        );

        Ok(())
    }

    /// Store sync token (delta link) for Office365 incremental sync
    /// Preserves the current sync_status instead of resetting to idle
    async fn store_sync_token(&self, folder: &SyncFolder, token: &str) -> SyncResult<()> {
        let id = Uuid::now_v7().to_string();
        let account_id_str = folder.account_id.to_string();
        let folder_id_str = folder.id.unwrap().to_string();

        sqlx::query!(
            r#"
            INSERT INTO sync_state (id, account_id, folder_id, sync_token)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(account_id, folder_id)
            DO UPDATE SET
                sync_token = ?,
                updated_at = CURRENT_TIMESTAMP
            "#,
            id,
            account_id_str,
            folder_id_str,
            token,
            token
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        log::debug!(
            "[EmailSync] Stored sync token {} for folder {} (account {})",
            token,
            folder.name,
            folder.account_id
        );

        Ok(())
    }

    /// Update sync state after successful sync
    async fn update_sync_state(&self, folder: &SyncFolder) -> SyncResult<()> {
        let id = Uuid::now_v7().to_string();
        let account_id_str = folder.account_id.to_string();
        let folder_id_str = folder.id.unwrap().to_string();

        sqlx::query!(
            r#"
            INSERT INTO sync_state (id, account_id, folder_id, last_sync_at)
            VALUES (?, ?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(account_id, folder_id)
            DO UPDATE SET
                last_sync_at = CURRENT_TIMESTAMP,
                error_count = 0,
                error_message = NULL
            "#,
            id,
            account_id_str,
            folder_id_str
        )
        .execute(&self.pool)
        .await
        .map_err(|e| super::error::SyncError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Update folder's synced_at timestamp after successful sync
    async fn update_folder_synced_at(&self, folder: &SyncFolder) -> SyncResult<()> {
        let folder_id_str = folder.id.unwrap().to_string();

        sqlx::query!(
            r#"
            UPDATE folders
            SET synced_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            folder_id_str
        )
        .execute(&self.pool)
        .await
        .map_err(|e| super::error::SyncError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Find or create a conversation by remote_id (provider thread/conversation ID)
    /// Returns the conversation's UUID to use as the foreign key
    async fn find_or_create_conversation(&self, remote_id: &str) -> SyncResult<Uuid> {
        let existing = sqlx::query!(
            "SELECT id FROM conversations WHERE remote_id = ?",
            remote_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        if let Some(record) = existing {
            Uuid::parse_str(&record.id)
                .map_err(|e| SyncError::DatabaseError(format!("Invalid conversation UUID: {}", e)))
        } else {
            let conversation_id = Uuid::now_v7();
            let conversation_id_str = conversation_id.to_string();

            sqlx::query!(
                r#"
                INSERT INTO conversations (id, remote_id, message_count, ai_cache)
                VALUES (?, ?, 0, NULL)
                "#,
                conversation_id_str,
                remote_id
            )
            .execute(&self.pool)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

            Ok(conversation_id)
        }
    }

    /// Convert SyncEmail to database Email model
    fn sync_email_to_db_model(
        &self,
        sync_email: &SyncEmail,
        email_id: Uuid,
        account_id: Uuid,
        sync_status: &str,
        category: Option<String>,
        body_plain: Option<String>,
        body_html: Option<String>,
        other_mails: Option<String>,
        conversation_uuid: Option<Uuid>,
        change_key: Option<String>,
        last_modified_at: Option<DateTime<Utc>>,
    ) -> SyncResult<crate::database::models::email::Email> {
        use crate::database::models::email::Email;
        use sqlx::types::Json;

        Ok(Email {
            id: email_id,
            account_id,
            folder_id: sync_email.folder_id,
            message_id: sync_email.message_id.clone(),
            conversation_id: conversation_uuid.map(|u| u.to_string()),
            remote_id: Some(sync_email.remote_id.clone()),
            from: Json(sync_email.from.clone()),
            to: Json(sync_email.to.clone()),
            cc: Json(sync_email.cc.clone()),
            bcc: Json(sync_email.bcc.clone()),
            reply_to: sync_email.reply_to.as_ref().map(|r| Json(r.clone())),
            subject: sync_email.subject.clone(),
            snippet: sync_email.snippet.clone(),
            body_plain,
            body_html,
            other_mails,
            category,
            ai_cache: None,
            received_at: sync_email.received_at,
            sent_at: sync_email.sent_at,
            scheduled_send_at: None,
            is_read: sync_email.flags.contains(&"\\Seen".to_string()),
            is_flagged: sync_email.flags.contains(&"\\Flagged".to_string()),
            is_draft: sync_email.flags.contains(&"\\Draft".to_string()),
            has_attachments: sync_email.has_attachments,
            is_deleted: false,
            headers: sync_email
                .headers
                .as_ref()
                .map(|h| serde_json::to_string(h))
                .transpose()
                .map_err(|e| SyncError::JsonError(e))?,
            sync_status: sync_status.to_string(),
            tracking_blocked: true,
            images_blocked: true,
            body_fetch_attempts: 0,
            last_body_fetch_attempt: None,
            change_key,
            last_modified_at,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            size: sync_email.size,
        })
    }

    fn get_conversion_mode(&self) -> EmailConversionMode {
        self.settings
            .get::<String>("email.conversionMode")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(EmailConversionMode::Markdown)
    }

    /// Upsert an email into the database using repository pattern
    /// Returns (email_id, inline_attachment_ids, is_new, db_email)
    async fn upsert_email(
        &self,
        email: &SyncEmail,
        account_id: Uuid,
        sync_status: &str,
    ) -> SyncResult<(Uuid, Vec<Uuid>, bool, crate::database::models::email::Email)> {
        let repo_factory = RepositoryFactory::new(self.pool.clone());
        let email_repo = repo_factory.email_repository();
        let conversion_mode = self.get_conversion_mode();

        let category = EmailCategorizer::categorize(
            email.headers.as_ref(),
            email.subject.as_deref(),
            email.body_plain.as_deref(),
            email.body_html.as_deref(),
            &email.from.address,
        )
        .map(|c| c.as_str().to_string());

        let existing = email_repo
            .find_by_remote_id_or_message_id(account_id, &email.remote_id, &email.message_id)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;
        let mut body_plain = existing.as_ref().and_then(|e| e.body_plain.clone());
        let mut body_html = existing.as_ref().and_then(|e| e.body_html.clone());
        let mut other_mails = existing.as_ref().and_then(|e| e.other_mails.clone());

        if body_html.is_none() {
            let split_result = EmailBodySplitter::split_body(email.body_html.as_deref());
            body_html = if split_result.body_html.is_empty() {
                email.body_html.clone()
            } else {
                Some(split_result.body_html)
            };
            other_mails = split_result.other_mails;
        }

        if body_plain.is_none() {
            body_plain = if email.body_plain.is_none()
                || email
                    .body_plain
                    .as_ref()
                    .map_or(true, |s| s.trim().is_empty())
            {
                if let Some(ref html) = body_html {
                    match conversion_mode {
                        EmailConversionMode::Markdown => {
                            let options = ConversionOptions {
                                extract_metadata: false,
                                preserve_tags: ["table"].iter().map(|s| s.to_string()).collect(),
                                preprocessing: PreprocessingOptions {
                                    enabled: true,
                                    remove_forms: true,
                                    remove_navigation: true,
                                    preset: PreprocessingPreset::Aggressive,
                                },
                                ..ConversionOptions::default()
                            };

                            match convert(html, Some(options)) {
                                Ok(markdown) => Some(markdown),
                                Err(_) => email.body_plain.clone(),
                            }
                        }
                        _ => match html2text::from_read(html.as_bytes(), 80) {
                            Ok(text) => Some(text),
                            Err(_) => email.body_plain.clone(),
                        },
                    }
                } else {
                    email.body_plain.clone()
                }
            } else {
                email.body_plain.clone()
            }
        };

        let conversation_uuid = if let Some(ref provider_conv_id) = email.conversation_id {
            Some(self.find_or_create_conversation(provider_conv_id).await?)
        } else {
            None
        };

        let (email_id, is_new, db_email) = if let Some(existing_email) = existing {
            let email_id = existing_email.id;
            let existing_sync_status = existing_email.sync_status.clone();
            let existing_folder_id = existing_email.folder_id;
            let was_deleted = existing_email.is_deleted;

            if was_deleted {
                log::info!(
                    "[EmailSync] Un-deleting email {} (remote_id: {}) - found on server again",
                    email_id,
                    email.remote_id
                );
                email_repo
                    .undelete(email_id)
                    .await
                    .map_err(|e| SyncError::DatabaseError(e.to_string()))?;
            }

            if existing_folder_id != email.folder_id {
                log::info!(
                    "[EmailSync] Email {} moved from folder {} to {}",
                    email_id,
                    existing_folder_id,
                    email.folder_id
                );
            }

            let should_update_body = sync_status == "synced"
                || existing_sync_status == "headers_only"
                || existing_sync_status == "error";

            log::debug!(
                "[EmailSync] Updating email {} - existing status: {}, new status: {}, will update body: {}",
                email_id,
                existing_sync_status,
                sync_status,
                should_update_body
            );

            let db_email = self.sync_email_to_db_model(
                email,
                email_id,
                account_id,
                sync_status,
                category,
                body_plain.clone(),
                body_html.clone(),
                other_mails.clone(),
                conversation_uuid,
                email.change_key.clone(),
                email.last_modified_at,
            )?;

            if should_update_body {
                log::debug!(
                    "[EmailSync] Full update for email {}, folder_id: {:?}",
                    email_id,
                    email.folder_id
                );
                email_repo
                    .update(&db_email)
                    .await
                    .map_err(|e| SyncError::DatabaseError(e.to_string()))?;
            } else {
                log::debug!(
                    "[EmailSync] Metadata update for email {}, folder_id: {:?}",
                    email_id,
                    email.folder_id
                );
                email_repo
                    .update_metadata_only(&db_email)
                    .await
                    .map_err(|e| SyncError::DatabaseError(e.to_string()))?;
            }

            (email_id, false, db_email)
        } else {
            let email_id = Uuid::now_v7();
            let db_email = self.sync_email_to_db_model(
                email,
                email_id,
                account_id,
                sync_status,
                category,
                body_plain.clone(),
                body_html.clone(),
                other_mails.clone(),
                conversation_uuid,
                email.change_key.clone(),
                email.last_modified_at,
            )?;

            email_repo
                .create(&db_email)
                .await
                .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

            (email_id, true, db_email)
        };

        let inline_attachment_ids = if !email.attachments.is_empty() {
            let processed = self
                .attachment_handler
                .process_attachments(email_id, account_id, &email.attachments)
                .await?;

            let mut uncached_inline = Vec::new();
            for (att_id, _is_inline) in processed {
                if !self.attachment_handler.is_cached(att_id).await? {
                    uncached_inline.push(att_id);
                }
            }
            uncached_inline
        } else {
            Vec::new()
        };

        if sync_status == "synced" {
            if let Some(search_manager) = &self.search_manager {
                if let Err(e) = search_manager.index_email(&db_email).await {
                    log::warn!(
                        "[EmailSync] Failed to index email {} in search: {}",
                        email_id,
                        e
                    );
                }
            }
        }

        Ok((email_id, inline_attachment_ids, is_new, db_email))
    }
}
