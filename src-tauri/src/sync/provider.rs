use super::error::SyncResult;
use super::types::*;
use crate::database::models::account::Account;
use crate::sync::SyncError;
use async_trait::async_trait;
use std::format;

/// Provider-agnostic email synchronization interface
#[async_trait]
pub trait EmailProvider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &str;

    /// Allow downcasting to concrete provider types
    fn as_any(&self) -> &dyn std::any::Any;

    /// Authenticate with the provider
    async fn authenticate(&mut self, credentials: ProviderCredentials) -> SyncResult<()>;

    /// Test connection to the provider
    async fn test_connection(&self) -> SyncResult<bool>;

    /// Fetch all folders from the provider
    async fn fetch_folders(&self) -> SyncResult<Vec<SyncFolder>>;

    /// Sync emails from a folder with delta detection
    ///
    /// # Arguments
    /// * `folder` - The folder to sync
    /// * `sync_token` - Optional delta token for incremental sync. If None, performs full sync.
    ///
    /// # Returns
    /// `SyncDiff` containing added, modified, and deleted email IDs, plus next sync token.
    ///
    /// **Delta sync**: Provider returns only changed emails (new, modified, deleted)
    /// **Full sync**: Provider returns all emails, EmailSync will compute deletions
    async fn sync_messages(
        &self,
        folder: &SyncFolder,
        sync_token: Option<String>,
    ) -> SyncResult<SyncDiff>;

    /// Fetch a single email by its remote ID
    async fn fetch_email(&self, folder: &SyncFolder, remote_id: &str) -> SyncResult<SyncEmail>;

    /// Fetch attachment content
    async fn fetch_attachment(&self, attachment: &SyncAttachment) -> SyncResult<Vec<u8>>;

    /// Move an email to a different folder
    async fn move_email(
        &self,
        email_remote_id: &str,
        from_folder: &SyncFolder,
        to_folder: &SyncFolder,
    ) -> SyncResult<()>;

    /// Delete an email
    async fn delete_email(
        &self,
        email_remote_id: &str,
        folder: &SyncFolder,
        permanent: bool,
    ) -> SyncResult<()>;

    /// Mark email as read/unread
    async fn mark_as_read(
        &self,
        email_remote_id: &str,
        folder: &SyncFolder,
        is_read: bool,
    ) -> SyncResult<()>;

    /// Flag/unflag an email
    async fn set_flag(
        &self,
        email_remote_id: &str,
        folder: &SyncFolder,
        flagged: bool,
    ) -> SyncResult<()>;

    /// Rename a folder
    async fn rename_folder(&self, _folder: &SyncFolder, _new_name: &str) -> SyncResult<()> {
        Err(SyncError::NotSupported(
            "This provider does not support folder renaming".to_string(),
        ))
    }

    /// Move a folder to a new parent
    async fn move_folder(
        &self,
        _folder: &SyncFolder,
        _new_parent_path: Option<&str>,
    ) -> SyncResult<()> {
        Err(SyncError::NotSupported(
            "This provider does not support folder moving".to_string(),
        ))
    }

    /// Get the sync token for incremental sync (Gmail historyId, etc.)
    async fn get_sync_token(&self) -> SyncResult<Option<String>>;

    /// Sync changes since the last sync token
    async fn sync_since_token(&self, token: &str) -> SyncResult<Vec<SyncEmail>>;

    /// Send an email via the provider's API (optional, for providers that support API-based sending)
    /// Returns NotSupported error by default - providers that support API sending should override
    async fn send_email(
        &self,
        _to: Vec<super::types::EmailRecipient>,
        _cc: Vec<super::types::EmailRecipient>,
        _bcc: Vec<super::types::EmailRecipient>,
        _subject: String,
        _body_html: String,
        _attachments: Vec<super::types::EmailAttachmentData>,
    ) -> SyncResult<()> {
        Err(SyncError::NotSupported(
            "This provider does not support API-based email sending".to_string(),
        ))
    }
}

/// Factory for creating email provider instances
pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create(
        account: &Account,
        credential_store: std::sync::Arc<super::auth::CredentialStore>,
    ) -> SyncResult<Box<dyn EmailProvider>> {
        Self::create_with_app_handle(account, credential_store, None)
    }

    pub fn create_with_app_handle(
        account: &Account,
        credential_store: std::sync::Arc<super::auth::CredentialStore>,
        app_handle: Option<tauri::AppHandle>,
    ) -> SyncResult<Box<dyn EmailProvider>> {
        use crate::sync::providers;

        let settings: AccountSettings = match &account.settings {
            serde_json::Value::String(s) => serde_json::from_str(s)?,
            _ => serde_json::from_value(account.settings.clone())?,
        };

        match account.account_type.as_str() {
            "gmail" => {
                let provider = providers::gmail::GmailProvider::new(account.id, credential_store)?;
                Ok(Box::new(provider))
            }
            "office365" => {
                let mut provider =
                    providers::office365::Office365Provider::new(account.id, credential_store)?;
                if let Some(app_handle) = app_handle {
                    provider = provider.with_app_handle(app_handle);
                }
                Ok(Box::new(provider))
            }
            "imap" => {
                let provider = providers::imap::ImapProvider::new(account.id, credential_store)?
                    .with_settings(settings);
                Ok(Box::new(provider))
            }
            "apple" => {
                let provider = providers::imap::ImapProvider::new(account.id, credential_store)?
                    .with_settings(settings);
                Ok(Box::new(provider))
            }
            _ => Err(super::error::SyncError::NotSupported(format!(
                "Provider {} is not supported",
                account.account_type
            ))),
        }
    }
}
