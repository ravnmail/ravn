use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::auth::CredentialStore;
use super::error::{SyncError, SyncResult};
use super::SyncManager;
use crate::config::settings::Settings;
use crate::database::models::account::Account;
use crate::database::repositories::{AccountRepository, RepositoryFactory};
use crate::search::SearchManager;

/// Central coordinator for managing per-account SyncManager instances
///
/// This ensures each account has its own SyncManager with pre-loaded credentials
/// and proper state management. Instances are cached and reused for efficiency.
pub struct SyncCoordinator {
    pool: SqlitePool,
    app_data_dir: String,
    credential_store: Arc<CredentialStore>,
    search_manager: Option<Arc<SearchManager>>,
    settings: Arc<Settings>,
    app_handle: Option<tauri::AppHandle>,
    /// Cache of account_id -> SyncManager instances
    managers: Arc<RwLock<HashMap<Uuid, Arc<SyncManager>>>>,
}

impl SyncCoordinator {
    pub fn new(
        pool: SqlitePool,
        app_data_dir: String,
        credential_store: Arc<CredentialStore>,
        settings: Arc<Settings>,
    ) -> Self {
        Self {
            pool,
            app_data_dir,
            credential_store,
            search_manager: None,
            settings,
            app_handle: None,
            managers: Arc::new(RwLock::new(HashMap::new())),
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

    /// Get or create a SyncManager for a specific account
    ///
    /// This method ensures:
    /// - One SyncManager per account
    /// - Credentials are validated before creating the manager
    /// - Managers are cached for reuse
    pub async fn get_manager_for_account(&self, account: &Account) -> SyncResult<Arc<SyncManager>> {
        let account_id = account.id;

        {
            let managers = self.managers.read().await;
            if let Some(manager) = managers.get(&account_id) {
                log::debug!(
                    "[SyncCoordinator] Reusing existing SyncManager for account {}",
                    account_id
                );
                return Ok(Arc::clone(manager));
            }
        }

        log::info!(
            "[SyncCoordinator] Creating new SyncManager for account {}",
            account_id
        );

        let has_credentials = self.credential_store.has_credentials(account_id).await;

        if !has_credentials {
            log::warn!(
                "[SyncCoordinator] No credentials found for account {}",
                account_id
            );

            if let Some(app_handle) = &self.app_handle {
                if let Ok(account) = self.get_account(account_id).await {
                    let event_payload = super::events::CredentialsRequiredEvent {
                        account_id,
                        provider: account.account_type.to_string(),
                        reason: "Credentials not configured".to_string(),
                    };
                    super::events::emit_event(app_handle, "credentials:required", event_payload);
                }
            }

            return Err(SyncError::InvalidConfiguration(format!(
                "Credentials not configured for account {}",
                account_id
            )));
        }

        let mut manager = SyncManager::new(
            self.pool.clone(),
            self.app_data_dir.clone(),
            Arc::clone(&self.credential_store),
            Arc::clone(&self.settings),
        );

        if let Some(search_manager) = &self.search_manager {
            manager = manager.with_search_manager(Arc::clone(search_manager));
        }

        if let Some(app_handle) = &self.app_handle {
            manager = manager.with_app_handle(app_handle.clone());
        }

        let manager = Arc::new(manager);

        {
            let mut managers = self.managers.write().await;
            managers.insert(account_id, Arc::clone(&manager));
        }

        log::info!(
            "[SyncCoordinator] SyncManager created and cached for account {}",
            account_id
        );
        Ok(manager)
    }

    /// Clear cached manager for an account
    pub async fn invalidate_account(&self, account_id: Uuid) {
        let mut managers = self.managers.write().await;
        if managers.remove(&account_id).is_some() {
            log::info!(
                "[SyncCoordinator] Invalidated SyncManager cache for account {}",
                account_id
            );
        }
    }

    /// Clear all cached managers
    pub async fn clear_cache(&self) {
        let mut managers = self.managers.write().await;
        managers.clear();
        log::info!("[SyncCoordinator] Cleared all SyncManager caches");
    }

    /// Get account by ID
    async fn get_account(&self, account_id: Uuid) -> SyncResult<Account> {
        let repo_factory = RepositoryFactory::new(self.pool.clone());
        let account_repo = repo_factory.account_repository();

        account_repo
            .find_by_id(account_id)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
            .ok_or_else(|| SyncError::DatabaseError(format!("Account not found: {}", account_id)))
    }

    /// Execute an operation with the appropriate SyncManager for an account
    pub async fn with_manager<F, T>(&self, account_id: Uuid, operation: F) -> SyncResult<T>
    where
        F: FnOnce(
            Arc<SyncManager>,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = SyncResult<T>> + Send>>,
    {
        let account = self.get_account(account_id).await?;
        let manager = self.get_manager_for_account(&account).await?;
        operation(manager).await
    }
}

// Convenience methods that delegate to the cached SyncManager
impl SyncCoordinator {
    pub async fn sync_account(
        &self,
        account_id: Uuid,
    ) -> SyncResult<super::sync_manager::SyncReport> {
        let account = self.get_account(account_id).await?;
        let manager = self.get_manager_for_account(&account).await?;
        manager.sync_account(&account).await
    }

    pub async fn sync_folder(
        &self,
        account_id: Uuid,
        folder: &super::types::SyncFolder,
        full: bool,
    ) -> SyncResult<usize> {
        let account = self.get_account(account_id).await?;
        let manager = self.get_manager_for_account(&account).await?;
        manager.sync_folder(&account, folder, full).await
    }

    pub async fn move_email(
        &self,
        account_id: Uuid,
        email_id: Uuid,
        to_folder_id: Uuid,
    ) -> SyncResult<()> {
        let account = self.get_account(account_id).await?;
        let manager = self.get_manager_for_account(&account).await?;
        manager.move_email(&account, email_id, to_folder_id).await
    }

    pub async fn delete_email(
        &self,
        account_id: Uuid,
        email_id: Uuid,
        permanent: bool,
    ) -> SyncResult<()> {
        let account = self.get_account(account_id).await?;
        let manager = self.get_manager_for_account(&account).await?;
        manager.delete_email(&account, email_id, permanent).await
    }

    pub async fn mark_as_read(
        &self,
        account_id: Uuid,
        email_id: Uuid,
        is_read: bool,
    ) -> SyncResult<()> {
        log::info!(
            "[SyncCoordinator] mark_as_read: account={}, email={}, is_read={}",
            account_id,
            email_id,
            is_read
        );

        let account = self.get_account(account_id).await?;
        let manager = self.get_manager_for_account(&account).await?;
        manager.mark_as_read(&account, email_id, is_read).await
    }

    pub async fn set_flag(
        &self,
        account_id: Uuid,
        email_id: Uuid,
        flagged: bool,
    ) -> SyncResult<()> {
        let account = self.get_account(account_id).await?;
        let manager = self.get_manager_for_account(&account).await?;
        manager.set_flag(&account, email_id, flagged).await
    }

    pub async fn rename_folder(
        &self,
        account_id: Uuid,
        folder_id: Uuid,
        old_name: &str,
        new_name: &str,
    ) -> SyncResult<()> {
        let account = self.get_account(account_id).await?;
        let manager = self.get_manager_for_account(&account).await?;
        manager
            .rename_folder(&account, folder_id, old_name, new_name)
            .await
    }

    pub async fn move_folder(
        &self,
        account_id: Uuid,
        folder_id: Uuid,
        old_parent_id: Option<Uuid>,
        new_parent_id: Option<Uuid>,
    ) -> SyncResult<()> {
        let account = self.get_account(account_id).await?;
        let manager = self.get_manager_for_account(&account).await?;
        manager
            .move_folder(&account, folder_id, old_parent_id, new_parent_id)
            .await
    }
}
