use super::auth::CredentialStore;
use super::error::{SyncError, SyncResult};
use super::provider::ProviderFactory;
use super::types::{ProviderCredentials, SyncFolder};
use crate::database::models::account::Account;
use crate::database::repositories::{FolderRepository, SqliteFolderRepository};
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

pub struct FolderSync {
    pool: SqlitePool,
    credential_store: Arc<CredentialStore>,
    app_handle: Option<tauri::AppHandle>,
}

impl FolderSync {
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

    /// Synchronize folders for an account
    pub async fn sync_folders(&self, account: &Account) -> SyncResult<Vec<SyncFolder>> {
        log::info!("Syncing folders for account {}", account.id);

        let mut provider = ProviderFactory::create_with_app_handle(
            account,
            Arc::clone(&self.credential_store),
            self.app_handle.clone(),
        )?;

        let credentials = self.load_credentials(account).await?;
        provider.authenticate(credentials).await?;

        let mut remote_folders = provider.fetch_folders().await?;

        for folder in remote_folders.iter_mut() {
            let folder_id = self.upsert_folder(folder).await?;
            folder.id = Some(Uuid::parse_str(&folder_id).unwrap());
        }

        log::info!(
            "Successfully synced {} folders for account {}",
            remote_folders.len(),
            account.id
        );

        Ok(remote_folders)
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

    /// Upsert a folder into the database
    async fn upsert_folder(&self, folder: &SyncFolder) -> SyncResult<String> {
        let (parent_remote_opt, _base_name_raw) = extract_base_name(&folder.remote_id.as_str());
        let (_, base_name) = extract_base_name(&folder.name.as_str());

        let icon = folder.folder_type.default_icon();
        let sync_interval = folder.folder_type.default_sync_interval() as i64;
        let folder_type_str = folder.folder_type.to_string();

        // Lookup parent id - check multiple sources in priority order:
        // 1. parent_remote_id from attributes (for Office365)
        // 2. parent from path-based remote_id (for IMAP)
        // 3. folder.parent_id if already set
        let parent_id_str: Option<String> = {
            let parent_from_attributes = folder.attributes.iter().find_map(|attr| {
                if attr.starts_with("parent_remote_id:") {
                    Some(attr.trim_start_matches("parent_remote_id:"))
                } else {
                    None
                }
            });

            if let Some(parent_remote) = parent_from_attributes.or(parent_remote_opt) {
                let account_id_str = folder.account_id.to_string();
                match sqlx::query!(
                    "SELECT id FROM folders WHERE account_id = ? AND remote_id = ?",
                    account_id_str,
                    parent_remote
                )
                .fetch_optional(&self.pool)
                .await
                {
                    Ok(Some(rec)) => Some(rec.id),
                    _ => None,
                }
            } else {
                folder.parent_id.map(|id| id.to_string())
            }
        };

        let account_id_str = folder.account_id.to_string();
        let existing = sqlx::query!(
            "SELECT id FROM folders WHERE account_id = ? AND remote_id = ?",
            account_id_str,
            folder.remote_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| super::error::SyncError::DatabaseError(e.to_string()))?;

        if let Some(record) = existing {
            sqlx::query!(
                r#"
                UPDATE folders
                SET name = ?, folder_type = ?, sync_interval = ?,
                    parent_id = ?, synced_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
                WHERE id = ?
                "#,
                base_name,
                folder_type_str,
                sync_interval,
                parent_id_str,
                record.id
            )
            .execute(&self.pool)
            .await
            .map_err(|e| super::error::SyncError::DatabaseError(e.to_string()))?;

            let folder_id = record.id;
            log::debug!("Updated folder: {} (id: {})", folder.name, folder_id);
            Ok(folder_id)
        } else {
            let folder_id = Uuid::now_v7();
            let folder_id_str = folder_id.to_string();
            let account_id_str = folder.account_id.to_string();
            sqlx::query!(
                r#"
                INSERT INTO folders (id, account_id, name, folder_type, remote_id, icon,
                                   sync_interval, synced_at, parent_id)
                VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, ?)
                "#,
                folder_id_str,
                account_id_str,
                base_name,
                folder_type_str,
                folder.remote_id,
                icon,
                sync_interval,
                parent_id_str
            )
            .execute(&self.pool)
            .await
            .map_err(|e| super::error::SyncError::DatabaseError(e.to_string()))?;

            log::debug!("Inserted folder: {} (id: {})", folder.name, folder_id);
            Ok(folder_id_str)
        }
    }

    /// Get folders for an account from database
    pub async fn get_folders(&self, account_id: Uuid) -> SyncResult<Vec<SyncFolder>> {
        let folder_repo = SqliteFolderRepository::new(self.pool.clone());
        let folders = folder_repo
            .find_by_account(account_id)
            .await
            .map_err(|e| super::error::SyncError::DatabaseError(e.to_string()))?;

        let sync_folders = folders
            .into_iter()
            .filter(|folder| !folder.hidden)
            .map(|folder| SyncFolder {
                id: Some(folder.id),
                account_id: folder.account_id,
                name: folder.name,
                folder_type: folder.folder_type,
                remote_id: folder.remote_id.unwrap_or_default(),
                icon: folder.icon,
                color: folder.color,
                parent_id: folder.parent_id,
                attributes: Vec::new(),
                unread_count: folder.unread_count as i32,
                total_count: folder.total_count as i32,
                expanded: folder.expanded,
                hidden: folder.hidden,
                synced_at: Some(folder.synced_at),
                sync_interval: folder.sync_interval,
            })
            .collect();

        Ok(sync_folders)
    }
}

fn extract_base_name(remote: &str) -> (Option<&str>, &str) {
    let sep_pos = match (remote.rfind('/'), remote.rfind('.')) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        _ => None,
    };

    let (parent_remote_opt, base_name_raw) = if let Some(pos) = sep_pos {
        (Some(&remote[..pos]), &remote[pos + 1..])
    } else {
        (Some(""), remote.as_ref())
    };
    (parent_remote_opt, base_name_raw)
}
