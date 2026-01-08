use aes_gcm::{
    aead::{rand_core::RngCore, Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use sqlx::SqlitePool;
use uuid::Uuid;

use super::error::{SyncError, SyncResult};
use super::types::{ImapCredentials, OAuth2Credentials};

/// Encrypted credential storage using database with AES-256-GCM encryption
pub struct EncryptedCredentialStore {
    pool: SqlitePool,
    encryption_key: [u8; 32],
}

impl EncryptedCredentialStore {
    /// Create a new encrypted credential store
    pub fn new(pool: SqlitePool, app_data_dir: &str) -> SyncResult<Self> {
        let encryption_key = Self::get_or_create_encryption_key(app_data_dir)?;
        Ok(Self {
            pool,
            encryption_key,
        })
    }

    /// Get or create the encryption key (stored securely in app data directory)
    fn get_or_create_encryption_key(app_data_dir: &str) -> SyncResult<[u8; 32]> {
        let key_path = std::path::Path::new(app_data_dir).join(".ravn_key");

        if key_path.exists() {
            // Load existing key
            let key_bytes = std::fs::read(&key_path).map_err(|e| {
                SyncError::KeyringError(format!("Failed to read encryption key: {}", e))
            })?;

            if key_bytes.len() != 32 {
                return Err(SyncError::KeyringError(
                    "Invalid encryption key length".to_string(),
                ));
            }

            let mut key = [0u8; 32];
            key.copy_from_slice(&key_bytes);
            Ok(key)
        } else {
            // Generate new key
            let mut key = [0u8; 32];
            OsRng.fill_bytes(&mut key);

            // Create app data directory if it doesn't exist
            if let Some(parent) = key_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    SyncError::KeyringError(format!("Failed to create app data directory: {}", e))
                })?;
            }

            // Save key with restricted permissions
            std::fs::write(&key_path, &key).map_err(|e| {
                SyncError::KeyringError(format!("Failed to write encryption key: {}", e))
            })?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&key_path)
                    .map_err(|e| {
                        SyncError::KeyringError(format!("Failed to get key file metadata: {}", e))
                    })?
                    .permissions();
                perms.set_mode(0o600);
                std::fs::set_permissions(&key_path, perms).map_err(|e| {
                    SyncError::KeyringError(format!("Failed to set key file permissions: {}", e))
                })?;
            }

            log::info!("Generated new encryption key at {:?}", key_path);
            Ok(key)
        }
    }

    /// Encrypt data using AES-256-GCM
    fn encrypt(&self, plaintext: &[u8]) -> SyncResult<(Vec<u8>, Vec<u8>)> {
        let cipher = Aes256Gcm::new_from_slice(&self.encryption_key)
            .map_err(|e| SyncError::KeyringError(format!("Failed to create cipher: {}", e)))?;

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| SyncError::KeyringError(format!("Encryption failed: {}", e)))?;

        Ok((ciphertext, nonce_bytes.to_vec()))
    }

    /// Decrypt data using AES-256-GCM
    fn decrypt(&self, ciphertext: &[u8], nonce_bytes: &[u8]) -> SyncResult<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(&self.encryption_key)
            .map_err(|e| SyncError::KeyringError(format!("Failed to create cipher: {}", e)))?;

        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| SyncError::KeyringError(format!("Decryption failed: {}", e)))?;

        Ok(plaintext)
    }

    /// Store OAuth2 credentials
    pub async fn store_oauth2(
        &self,
        account_id: Uuid,
        credentials: &OAuth2Credentials,
    ) -> SyncResult<()> {
        let json = serde_json::to_string(credentials)?;
        let id = Uuid::now_v7().to_string();
        let (encrypted_data, nonce) = self.encrypt(json.as_bytes())?;

        let account_id_str = account_id.to_string();
        sqlx::query!(
            r#"
            INSERT INTO encrypted_credentials (id, account_id, credential_type, encrypted_data, nonce, updated_at)
            VALUES (?, ?, 'oauth2', ?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(account_id, credential_type) DO UPDATE SET
                encrypted_data = excluded.encrypted_data,
                nonce = excluded.nonce,
                updated_at = CURRENT_TIMESTAMP
            "#,
            id,
            account_id_str,
            encrypted_data,
            nonce
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        log::info!(
            "Stored encrypted OAuth2 credentials for account {}",
            account_id
        );
        Ok(())
    }

    /// Retrieve OAuth2 credentials
    pub async fn get_oauth2(&self, account_id: Uuid) -> SyncResult<OAuth2Credentials> {
        let account_id_str = account_id.to_string();
        let record = sqlx::query!(
            r#"
            SELECT encrypted_data, nonce
            FROM encrypted_credentials
            WHERE account_id = ? AND credential_type = 'oauth2'
            "#,
            account_id_str
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| SyncError::KeyringError("No OAuth2 credentials found".to_string()))?;

        let plaintext = self.decrypt(&record.encrypted_data, &record.nonce)?;
        let json = String::from_utf8(plaintext)
            .map_err(|e| SyncError::KeyringError(format!("Invalid UTF-8 in credentials: {}", e)))?;

        let credentials: OAuth2Credentials = serde_json::from_str(&json)?;
        Ok(credentials)
    }

    /// Store IMAP credentials
    pub async fn store_imap(
        &self,
        account_id: Uuid,
        credentials: &ImapCredentials,
    ) -> SyncResult<()> {
        let json = serde_json::to_string(credentials)?;
        let id = Uuid::now_v7().to_string();
        let (encrypted_data, nonce) = self.encrypt(json.as_bytes())?;

        let account_id_str = account_id.to_string();
        sqlx::query!(
            r#"
            INSERT INTO encrypted_credentials (id, account_id, credential_type, encrypted_data, nonce, updated_at)
            VALUES (?, ?, 'imap', ?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(account_id, credential_type) DO UPDATE SET
                encrypted_data = excluded.encrypted_data,
                nonce = excluded.nonce,
                updated_at = CURRENT_TIMESTAMP
            "#,
            id,
            account_id_str,
            encrypted_data,
            nonce
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        log::info!(
            "Stored encrypted IMAP credentials for account {}",
            account_id
        );
        Ok(())
    }

    /// Retrieve IMAP credentials
    pub async fn get_imap(&self, account_id: Uuid) -> SyncResult<ImapCredentials> {
        let account_id_str = account_id.to_string();
        let record = sqlx::query!(
            r#"
            SELECT encrypted_data, nonce
            FROM encrypted_credentials
            WHERE account_id = ? AND credential_type = 'imap'
            "#,
            account_id_str
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| SyncError::KeyringError("No IMAP credentials found".to_string()))?;

        let plaintext = self.decrypt(&record.encrypted_data, &record.nonce)?;
        let json = String::from_utf8(plaintext)
            .map_err(|e| SyncError::KeyringError(format!("Invalid UTF-8 in credentials: {}", e)))?;

        let credentials: ImapCredentials = serde_json::from_str(&json)?;
        Ok(credentials)
    }

    /// Delete credentials for an account
    pub async fn delete(&self, account_id: Uuid) -> SyncResult<()> {
        let account_id_str = account_id.to_string();
        sqlx::query!(
            "DELETE FROM encrypted_credentials WHERE account_id = ?",
            account_id_str
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        log::info!("Deleted encrypted credentials for account {}", account_id);
        Ok(())
    }

    /// Check if credentials exist for an account
    pub async fn has_credentials(&self, account_id: Uuid) -> bool {
        let account_id_str = account_id.to_string();
        let result = sqlx::query!(
            "SELECT COUNT(*) as count FROM encrypted_credentials WHERE account_id = ?",
            account_id_str
        )
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(record) => record.count > 0,
            Err(_) => false,
        }
    }
}
