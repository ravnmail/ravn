use chrono::Utc;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use uuid::Uuid;

use super::auth::CredentialStore;
use super::error::{SyncError, SyncResult};
use super::sync_manager::SyncManager;
use super::sync_queue::{SyncPriority, SyncQueue, SyncQueueItem, SyncQueueWorker};
use super::types::FolderType;
use crate::database::repositories::{
    AccountRepository, SqliteAccountRepository, SqliteSyncStateRepository, SyncStateRepository,
};

/// Background sync task handle
struct SyncTask {
    handle: JoinHandle<()>,
}

/// Manages background synchronization tasks for all accounts
pub struct BackgroundSyncManager {
    pool: SqlitePool,
    app_data_dir: String,
    credential_store: Arc<CredentialStore>,
    tasks: Arc<RwLock<HashMap<Uuid, SyncTask>>>,
    shutdown_tx: tokio::sync::broadcast::Sender<()>,
    app_handle: tauri::AppHandle,
}

impl BackgroundSyncManager {
    /// Create a new background sync manager
    pub fn new(
        pool: SqlitePool,
        app_data_dir: String,
        credential_store: Arc<CredentialStore>,
        app_handle: tauri::AppHandle,
    ) -> Self {
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(16);

        Self {
            pool,
            app_data_dir,
            credential_store,
            tasks: Arc::new(RwLock::new(HashMap::new())),
            shutdown_tx,
            app_handle,
        }
    }

    /// Start sync for all accounts with sync_enabled=true
    pub async fn start_all(&self) -> SyncResult<Vec<Uuid>> {
        log::info!("Starting background sync for all enabled accounts");

        let account_repo = SqliteAccountRepository::new(self.pool.clone());
        let accounts = account_repo
            .find_by_sync_enabled()
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        let mut started_accounts = Vec::new();

        for account in accounts {
            match self.start_account_sync(&account.id).await {
                Ok(_) => {
                    started_accounts.push(account.id);
                    log::info!(
                        "Started background sync for account {} ({})",
                        account.id,
                        account.email
                    );
                }
                Err(e) => {
                    log::error!("Failed to start sync for account {}: {}", account.id, e);
                }
            }
        }

        log::info!(
            "Started background sync for {} accounts",
            started_accounts.len()
        );
        Ok(started_accounts)
    }

    /// Start background sync for a specific account
    pub async fn start_account_sync(&self, account_id: &Uuid) -> SyncResult<()> {
        {
            let tasks = self.tasks.read().await;
            if tasks.contains_key(account_id) {
                log::debug!("Background sync already running for account {}", account_id);
                return Ok(());
            }
        }

        let account_repo = SqliteAccountRepository::new(self.pool.clone());
        let account = account_repo
            .find_by_id(*account_id)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
            .ok_or_else(|| {
                SyncError::DatabaseError(format!("Account not found: {}", account_id))
            })?;

        let settings_value = account.settings.clone();

        if !self.credential_store.has_credentials(*account_id).await {
            log::warn!(
                "Skipping background sync for account {} ({}): No credentials found. Complete account setup first.",
                account_id,
                account.email
            );
            return Err(SyncError::InvalidConfiguration(
                "No credentials found for this account. Please complete account setup first."
                    .to_string(),
            ));
        }

        let account_settings: super::types::AccountSettings =
            serde_json::from_value(settings_value).unwrap_or_default();

        if !account_settings.sync_enabled {
            log::debug!("Sync not enabled for account {}", account_id);
            return Err(SyncError::InvalidConfiguration(
                "Sync is not enabled for this account".to_string(),
            ));
        }

        let pool = self.pool.clone();
        let app_data_dir = self.app_data_dir.clone();
        let credential_store = Arc::clone(&self.credential_store);
        let app_handle = self.app_handle.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        let account_id_copy = *account_id;

        let handle = tokio::spawn(async move {
            log::info!(
                "Background sync task started for account {}",
                account_id_copy
            );

            if account_settings.sync_on_startup {
                log::info!("Running initial sync for account {}", account_id_copy);
                let sync_manager = SyncManager::new(
                    pool.clone(),
                    app_data_dir.clone(),
                    Arc::clone(&credential_store),
                )
                .with_app_handle(app_handle.clone());

                match sync_manager.sync_account(&account).await {
                    Ok(report) => {
                        log::info!(
                            "Initial sync completed for account {}: {} folders, {} emails",
                            account_id_copy,
                            report.folders_synced,
                            report.emails_synced
                        );
                    }
                    Err(e) => {
                        log::error!("Initial sync failed for account {}: {}", account_id_copy, e);
                    }
                }
            }

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        log::info!("Shutdown signal received for account {}", account_id_copy);
                        break;
                    }
                    _ = Self::sync_folders_periodic(&pool, &app_data_dir, Arc::clone(&credential_store), app_handle.clone(), account_id_copy) => {
                    }
                }
            }

            log::info!(
                "Background sync task stopped for account {}",
                account_id_copy
            );
        });

        let mut tasks = self.tasks.write().await;
        tasks.insert(*account_id, SyncTask { handle });

        Ok(())
    }

    /// Stop background sync for a specific account
    pub async fn stop_account_sync(&self, account_id: &Uuid) -> SyncResult<()> {
        let mut tasks = self.tasks.write().await;

        if let Some(task) = tasks.remove(account_id) {
            log::info!("Stopping background sync for account {}", account_id);

            let _ = self.shutdown_tx.send(());

            match tokio::time::timeout(Duration::from_secs(10), task.handle).await {
                Ok(_) => {
                    log::info!("Background sync stopped for account {}", account_id);
                    Ok(())
                }
                Err(_) => {
                    log::warn!(
                        "Timeout waiting for sync task to stop for account {}",
                        account_id
                    );
                    Err(SyncError::timeout(
                        "Sync task did not stop in time".to_string(),
                    ))
                }
            }
        } else {
            log::debug!("No background sync running for account {}", account_id);
            Ok(())
        }
    }

    /// Stop all background sync tasks
    pub async fn stop_all(&self) -> SyncResult<()> {
        log::info!("Stopping all background sync tasks");

        let account_ids: Vec<Uuid> = {
            let tasks = self.tasks.read().await;
            tasks.keys().copied().collect()
        };

        for account_id in account_ids {
            if let Err(e) = self.stop_account_sync(&account_id).await {
                log::error!("Failed to stop sync for account {}: {}", account_id, e);
            }
        }

        log::info!("All background sync tasks stopped");
        Ok(())
    }

    /// Get list of accounts currently syncing
    pub async fn get_active_syncs(&self) -> Vec<Uuid> {
        let tasks = self.tasks.read().await;
        tasks.keys().copied().collect()
    }

    /// Check if sync is running for an account
    pub async fn is_syncing(&self, account_id: &Uuid) -> bool {
        let tasks = self.tasks.read().await;
        tasks.contains_key(account_id)
    }

    /// Periodic sync loop for all folders of an account
    /// Uses a priority queue with configurable concurrent workers
    async fn sync_folders_periodic(
        pool: &SqlitePool,
        app_data_dir: &str,
        credential_store: Arc<CredentialStore>,
        app_handle: tauri::AppHandle,
        account_id: Uuid,
    ) {
        let sync_manager = Arc::new(
            SyncManager::new(pool.clone(), app_data_dir.to_string(), credential_store)
                .with_app_handle(app_handle),
        );

        let account_repo = SqliteAccountRepository::new(pool.clone());
        let account = match account_repo.find_by_id(account_id).await {
            Ok(Some(account)) => account,
            Ok(None) => {
                log::error!("Account {} not found", account_id);
                sleep(Duration::from_secs(60)).await;
                return;
            }
            Err(e) => {
                log::error!("Failed to fetch account {}: {}", account_id, e);
                sleep(Duration::from_secs(60)).await;
                return;
            }
        };

        let folders = match sync_manager.get_folders(account_id).await {
            Ok(folders) => folders,
            Err(e) => {
                log::error!("Failed to fetch folders for account {}: {}", account_id, e);
                sleep(Duration::from_secs(60)).await;
                return;
            }
        };

        if folders.is_empty() {
            log::debug!(
                "No folders found for account {}, retrying in 60s",
                account_id
            );
            sleep(Duration::from_secs(60)).await;
            return;
        }

        let sync_queue = Arc::new(SyncQueue::new(3));
        let mut worker_handles = vec![];

        for worker_id in 0..sync_queue.workers_limit() {
            let queue = Arc::clone(&sync_queue);
            let manager = Arc::clone(&sync_manager);
            let handle = tokio::spawn(async move {
                let worker = SyncQueueWorker::new(queue, manager);
                let _ = worker.run(worker_id).await;
            });
            worker_handles.push(handle);
        }

        log::info!(
            "Started {} sync queue workers for account {}",
            sync_queue.workers_limit(),
            account_id
        );

        loop {
            let folders = match sync_manager.get_folders(account_id).await {
                Ok(folders) => folders,
                Err(e) => {
                    log::error!("Failed to fetch folders for account {}: {}", account_id, e);
                    sleep(Duration::from_secs(60)).await;
                    continue;
                }
            };

            if folders.is_empty() {
                log::debug!(
                    "No folders found for account {}, retrying in 60s",
                    account_id
                );
                sleep(Duration::from_secs(60)).await;
                continue;
            }

            let now = Utc::now();
            let mut enqueued = 0;

            for folder in &folders {
                if matches!(folder.folder_type, FolderType::Trash | FolderType::Spam) {
                    continue;
                }

                let folder_id = match folder.id {
                    Some(id) => id,
                    None => continue,
                };

                if sync_queue.is_processing(folder_id).await {
                    continue;
                }

                let should_sync = match folder.synced_at {
                    Some(synced_at) => {
                        let next_sync_time =
                            synced_at + chrono::Duration::seconds(folder.sync_interval);
                        now >= next_sync_time
                    }
                    None => true,
                };

                if should_sync {
                    let queue_item = SyncQueueItem {
                        account_id: account.id,
                        folder_id,
                        folder: folder.clone(),
                        account: account.clone(),
                        priority: SyncPriority::Normal,
                        last_synced_at: folder.synced_at,
                        enqueued_at: Utc::now(),
                    };

                    if let Err(e) = sync_queue.enqueue(queue_item).await {
                        log::warn!(
                            "Failed to enqueue folder {} for account {}: {}",
                            folder.name,
                            account_id,
                            e
                        );
                    } else {
                        enqueued += 1;
                    }
                }
            }

            if enqueued > 0 {
                log::debug!(
                    "Enqueued {} folders for account {} (queue size: {}, active: {})",
                    enqueued,
                    account_id,
                    sync_queue.size().await,
                    sync_queue.active_count().await
                );
            }

            sleep(Duration::from_secs(10)).await;
        }
    }

    /// Update sync state in database
    async fn _update_sync_state(
        pool: &SqlitePool,
        account_id: Uuid,
        folder_id: Uuid,
        status: &str,
        error_message: Option<&str>,
    ) -> SyncResult<()> {
        let sync_state_repo = SqliteSyncStateRepository::new(pool.clone());
        sync_state_repo
            .upsert(account_id, folder_id, status, error_message)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

impl Drop for BackgroundSyncManager {
    fn drop(&mut self) {
        let _ = self.shutdown_tx.send(());
    }
}
