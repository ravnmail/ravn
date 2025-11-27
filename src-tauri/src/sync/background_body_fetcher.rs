use super::attachment_handler::AttachmentHandler;
use super::auth::CredentialStore;
use super::error::{SyncError, SyncResult};
use super::provider::ProviderFactory;
use super::storage::LocalFileStorage;
use super::types::{ProviderCredentials, SyncFolder};
use crate::database::models::account::AccountType;
use crate::database::models::{account::Account, email::EmailSyncStatus};
use crate::database::repositories::{AccountRepository, RepositoryFactory};
use chrono::Utc;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use uuid::Uuid;

const MAX_FETCH_ATTEMPTS: i64 = 3;
const FETCH_BATCH_SIZE: i64 = 10;
const FETCH_INTERVAL_SECS: u64 = 5;

pub struct BackgroundBodyFetcher {
    pool: SqlitePool,
    app_data_dir: String,
    credential_store: Arc<CredentialStore>,
    active_fetches: Arc<RwLock<HashMap<Uuid, bool>>>,
    shutdown_tx: tokio::sync::broadcast::Sender<()>,
}

impl BackgroundBodyFetcher {
    pub fn new(
        pool: SqlitePool,
        app_data_dir: String,
        credential_store: Arc<CredentialStore>,
    ) -> Self {
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);

        Self {
            pool,
            app_data_dir,
            credential_store,
            active_fetches: Arc::new(RwLock::new(HashMap::new())),
            shutdown_tx,
        }
    }

    /// Start the background body fetcher for all accounts
    pub async fn start(&self) -> SyncResult<()> {
        log::info!("[BackgroundBodyFetcher] Starting background body fetcher service");

        let pool = self.pool.clone();
        let credential_store = Arc::clone(&self.credential_store);
        let active_fetches = Arc::clone(&self.active_fetches);
        let app_data_dir = self.app_data_dir.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        log::info!("[BackgroundBodyFetcher] Shutdown signal received");
                        break;
                    }
                    _ = sleep(Duration::from_secs(FETCH_INTERVAL_SECS)) => {
                        if let Err(e) = Self::fetch_pending_bodies(
                            &pool,
                            &app_data_dir,
                            &credential_store,
                            &active_fetches,
                        ).await {
                            log::error!("[BackgroundBodyFetcher] Error fetching bodies: {}", e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the background body fetcher
    pub fn stop(&self) {
        log::info!("[BackgroundBodyFetcher] Stopping background body fetcher service");
        let _ = self.shutdown_tx.send(());
    }

    /// Fetch pending bodies for all accounts
    async fn fetch_pending_bodies(
        pool: &SqlitePool,
        app_data_dir: &str,
        credential_store: &Arc<CredentialStore>,
        active_fetches: &Arc<RwLock<HashMap<Uuid, bool>>>,
    ) -> SyncResult<()> {
        let repo_factory = RepositoryFactory::new(pool.clone());
        let account_repo = repo_factory.account_repository();

        let accounts = account_repo
            .find_all()
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        log::debug!(
            "[BackgroundBodyFetcher] Checking {} accounts for pending bodies",
            accounts.len()
        );

        for account in accounts {
            {
                let fetches = active_fetches.read().await;
                if fetches.get(&account.id).copied().unwrap_or(false) {
                    log::debug!(
                        "[BackgroundBodyFetcher] Account {} already being processed, skipping",
                        account.id
                    );
                    continue;
                }
            }

            {
                let mut fetches = active_fetches.write().await;
                fetches.insert(account.id, true);
            }

            let pool_clone = pool.clone();
            let app_data_dir_clone = app_data_dir.to_string();
            let credential_store_clone = Arc::clone(credential_store);
            let active_fetches_clone = Arc::clone(active_fetches);

            tokio::spawn(async move {
                if let Err(e) = Self::fetch_bodies_for_account(
                    &pool_clone,
                    &app_data_dir_clone,
                    &credential_store_clone,
                    &account,
                )
                .await
                {
                    log::error!(
                        "[BackgroundBodyFetcher] Error fetching bodies for account {}: {}",
                        account.id,
                        e
                    );
                }

                let mut fetches = active_fetches_clone.write().await;
                fetches.insert(account.id, false);
            });
        }

        Ok(())
    }

    /// Fetch bodies for a specific account
    async fn fetch_bodies_for_account(
        pool: &SqlitePool,
        app_data_dir: &str,
        credential_store: &Arc<CredentialStore>,
        account: &Account,
    ) -> SyncResult<()> {
        log::debug!(
            "[BackgroundBodyFetcher] Fetching bodies for account {} ({})",
            account.id,
            account.email
        );

        if account.account_type != AccountType::Imap {
            log::debug!(
                "[BackgroundBodyFetcher] Skipping non-IMAP account {}",
                account.id
            );
            return Ok(());
        }

        // Get emails that need body fetching
        // Only fetch emails that are in 'headers_only' state (not 'fetching_body' to avoid duplicates)
        let account_id_str = account.id.to_string();
        let emails = sqlx::query!(
            r#"
            SELECT e.id, e.remote_id, e.folder_id, e.body_fetch_attempts,
                   f.remote_id as folder_remote_id, f.name as folder_name
            FROM emails e
            JOIN folders f ON e.folder_id = f.id
            WHERE e.account_id = ?
              AND e.sync_status = 'headers_only'
              AND e.body_fetch_attempts < ?
              AND (e.last_body_fetch_attempt IS NULL OR
                   datetime(e.last_body_fetch_attempt) < datetime('now', '-30 seconds'))
            ORDER BY e.received_at DESC
            LIMIT ?
            "#,
            account_id_str,
            MAX_FETCH_ATTEMPTS,
            FETCH_BATCH_SIZE
        )
        .fetch_all(pool)
        .await
        .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        if emails.is_empty() {
            log::debug!(
                "[BackgroundBodyFetcher] No pending bodies for account {}",
                account.id
            );
            return Ok(());
        }

        log::info!(
            "[BackgroundBodyFetcher] Found {} emails needing body fetch for account {}",
            emails.len(),
            account.id
        );

        let mut provider = ProviderFactory::create(account, Arc::clone(credential_store))?;
        let credentials = Self::load_credentials(credential_store, account).await?;
        provider.authenticate(credentials).await?;

        let imap_provider = provider
            .as_any()
            .downcast_ref::<crate::sync::providers::imap::ImapProvider>()
            .ok_or_else(|| {
                SyncError::InvalidConfiguration("Failed to downcast to IMAP provider".to_string())
            })?;

        let cache_dir = std::path::PathBuf::from(app_data_dir).join("attachments");
        let storage = Arc::new(LocalFileStorage::new(cache_dir));
        let attachment_handler = AttachmentHandler::new(pool.clone(), storage);

        for email in emails {
            let email_id_str = email.id.as_str();
            let email_id = Uuid::parse_str(email_id_str)
                .map_err(|e| SyncError::DatabaseError(format!("Invalid email ID: {}", e)))?;
            let remote_id = email.remote_id.as_deref().unwrap();
            let folder_id = Uuid::parse_str(&email.folder_id)
                .map_err(|e| SyncError::DatabaseError(format!("Invalid folder ID: {}", e)))?;
            let folder = SyncFolder {
                id: Some(folder_id),
                account_id: account.id,
                name: email.folder_name,
                folder_type: super::types::FolderType::Custom,
                remote_id: email.folder_remote_id.unwrap(),
                parent_id: None,
                icon: None,
                color: None,
                sync_interval: 0,
                synced_at: None,
                attributes: vec![],
                unread_count: 0,
                total_count: 0,
                expanded: false,
                hidden: false,
            };

            log::debug!(
                "[BackgroundBodyFetcher] Fetching body for email {} (remote_id: {}) in folder {}",
                email_id,
                remote_id,
                folder.name
            );

            let now = Utc::now();
            let email_id_str = email_id.to_string();
            sqlx::query!(
                "UPDATE emails SET sync_status = 'fetching_body', body_fetch_attempts = body_fetch_attempts + 1, last_body_fetch_attempt = ? WHERE id = ?",
                now,
                email_id_str
            )
            .execute(pool)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

            match imap_provider.fetch_email_body(&folder, remote_id).await {
                Ok((body_plain, body_html, headers, sent_at, attachments, snippet)) => {
                    log::debug!(
                        "[BackgroundBodyFetcher] Successfully fetched body for email {}",
                        email_id
                    );

                    let headers_json = headers
                        .as_ref()
                        .map(|h| serde_json::to_string(h))
                        .transpose()
                        .map_err(|e| SyncError::JsonError(e))?;

                    let has_attachments = !attachments
                        .iter()
                        .filter(|a| !a.is_inline)
                        .collect::<Vec<_>>()
                        .is_empty();

                    let email_id_str = email_id.to_string();
                    sqlx::query!(
                        "UPDATE emails SET body_plain = ?, body_html = ?, snippet = ?, headers = ?, sent_at = ?, has_attachments = ?, sync_status = 'synced' WHERE id = ?",
                        body_plain,
                        body_html,
                        snippet,
                        headers_json,
                        sent_at,
                        has_attachments,
                        email_id_str
                    )
                    .execute(pool)
                    .await
                    .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

                    if !attachments.is_empty() {
                        log::debug!(
                            "[BackgroundBodyFetcher] Processing {} attachments for email {}",
                            attachments.len(),
                            email_id
                        );

                        let processed = attachment_handler
                            .process_attachments(email_id, account.id, &attachments)
                            .await?;

                        for (att_id, is_inline) in processed {
                            if is_inline {
                                let attachment =
                                    attachment_handler.get_attachment_metadata(att_id).await?;

                                if let Some(att_with_data) = attachments
                                    .iter()
                                    .find(|a| a.hash == attachment.hash && a.data.is_some())
                                {
                                    if let Some(data) = &att_with_data.data {
                                        attachment_handler
                                            .cache_attachment(
                                                att_id,
                                                account.id,
                                                email_id,
                                                data,
                                                &attachment.filename,
                                            )
                                            .await?;

                                        log::debug!(
                                            "[BackgroundBodyFetcher] Cached inline attachment: {} ({})",
                                            attachment.filename,
                                            att_id
                                        );
                                    }
                                }
                            }
                        }
                    }

                    log::info!(
                        "[BackgroundBodyFetcher] Successfully synced body for email {}",
                        email_id
                    );
                }
                Err(e) => {
                    log::error!(
                        "[BackgroundBodyFetcher] Failed to fetch body for email {}: {}",
                        email_id,
                        e
                    );

                    let attempts = email.body_fetch_attempts + 1;
                    let sync_status = if attempts >= MAX_FETCH_ATTEMPTS {
                        log::warn!(
                            "[BackgroundBodyFetcher] Max fetch attempts reached for email {}",
                            email_id
                        );
                        EmailSyncStatus::Error.as_str()
                    } else {
                        EmailSyncStatus::HeadersOnly.as_str()
                    };

                    let email_id_str = email_id.to_string();
                    sqlx::query!(
                        "UPDATE emails SET sync_status = ? WHERE id = ?",
                        sync_status,
                        email_id_str
                    )
                    .execute(pool)
                    .await
                    .map_err(|e| SyncError::DatabaseError(e.to_string()))?;
                }
            }
        }

        log::info!(
            "[BackgroundBodyFetcher] Completed body fetch for account {}",
            account.id
        );

        Ok(())
    }

    /// Load credentials for an account
    async fn load_credentials(
        credential_store: &Arc<CredentialStore>,
        account: &Account,
    ) -> SyncResult<ProviderCredentials> {
        if !credential_store.has_credentials(account.id).await {
            return Err(SyncError::InvalidConfiguration(format!(
                "No credentials found for account {} ({})",
                account.id, account.email
            )));
        }

        match account.account_type.as_str() {
            "gmail" | "office365" => {
                let oauth_creds = credential_store.get_oauth2(account.id).await?;
                Ok(ProviderCredentials::OAuth2(oauth_creds))
            }
            "imap" | "apple" => {
                let imap_creds = credential_store.get_imap(account.id).await?;
                Ok(ProviderCredentials::Imap(imap_creds))
            }
            _ => Err(SyncError::NotSupported(format!(
                "Account type {} not supported",
                account.account_type
            ))),
        }
    }
}
