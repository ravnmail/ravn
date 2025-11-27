use keyring::Entry;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::encrypted_store::EncryptedCredentialStore;
use super::error::{SyncError, SyncResult};
use super::types::{ImapCredentials, OAuth2Credentials};

const KEYRING_SERVICE: &str = "com.ravn.email";

/// Detects if system keyring is using mock credentials
fn _is_keyring_mock() -> bool {
    let test_entry = Entry::new(KEYRING_SERVICE, "__ravn_keyring_test__");

    match test_entry {
        Ok(entry) => {
            if entry.set_password("test").is_ok() {
                if let Ok(password) = entry.get_password() {
                    let _ = entry.delete_credential();
                    return password.is_empty();
                }
            }
            true
        }
        Err(_) => true,
    }
}

/// Secure credential storage using OS keyring with encrypted database fallback
pub struct CredentialStore {
    encrypted_store: Option<Arc<RwLock<EncryptedCredentialStore>>>,
    use_encrypted_fallback: bool,
}

impl CredentialStore {
    /// Create a new credential store
    pub fn new(pool: Option<SqlitePool>, app_data_dir: Option<String>) -> Self {
        let use_encrypted_fallback = true; // cfg!(debug_assertions); // enabled in debug builds, disabled in release

        if use_encrypted_fallback {
            log::warn!("System keyring unavailable or using mock - falling back to encrypted database storage");
        }

        let encrypted_store = if let (Some(pool), Some(dir)) = (pool, app_data_dir) {
            match EncryptedCredentialStore::new(pool, &dir) {
                Ok(store) => Some(Arc::new(RwLock::new(store))),
                Err(e) => {
                    log::error!("Failed to initialize encrypted credential store: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Self {
            encrypted_store,
            use_encrypted_fallback,
        }
    }

    /// Store OAuth2 credentials securely
    pub async fn store_oauth2(
        &self,
        account_id: Uuid,
        credentials: &OAuth2Credentials,
    ) -> SyncResult<()> {
        if self.use_encrypted_fallback {
            if let Some(store) = &self.encrypted_store {
                let store = store.read().await;
                return store.store_oauth2(account_id, credentials).await;
            }
            return Err(SyncError::KeyringError(
                "No credential storage available".to_string(),
            ));
        }

        let key = format!("oauth2_account_{}", account_id.to_string());
        let entry = Entry::new(KEYRING_SERVICE, &key)?;
        let json = serde_json::to_string(credentials)?;
        entry.set_password(&json)?;
        log::info!(
            "Stored OAuth2 credentials in system keyring for account {}",
            account_id
        );
        Ok(())
    }

    /// Retrieve OAuth2 credentials
    pub async fn get_oauth2(&self, account_id: Uuid) -> SyncResult<OAuth2Credentials> {
        if self.use_encrypted_fallback {
            if let Some(store) = &self.encrypted_store {
                let store = store.read().await;
                return store.get_oauth2(account_id).await;
            }
            return Err(SyncError::KeyringError(
                "No credential storage available".to_string(),
            ));
        }

        let key = format!("oauth2_account_{}", account_id.to_string());
        let entry = Entry::new(KEYRING_SERVICE, &key)?;
        let json = entry.get_password()?;
        let credentials: OAuth2Credentials = serde_json::from_str(&json)?;
        Ok(credentials)
    }

    /// Store IMAP credentials securely
    pub async fn store_imap(
        &self,
        account_id: Uuid,
        credentials: &ImapCredentials,
    ) -> SyncResult<()> {
        if self.use_encrypted_fallback {
            if let Some(store) = &self.encrypted_store {
                let store = store.read().await;
                return store.store_imap(account_id, credentials).await;
            }
            return Err(SyncError::KeyringError(
                "No credential storage available".to_string(),
            ));
        }

        let key = format!("imap_account_{}", account_id.to_string());
        let entry = Entry::new(KEYRING_SERVICE, &key)?;
        let json = serde_json::to_string(credentials)?;
        entry.set_password(&json)?;
        log::info!(
            "Stored IMAP credentials in system keyring for account {}",
            account_id
        );
        Ok(())
    }

    /// Retrieve IMAP credentials
    pub async fn get_imap(&self, account_id: Uuid) -> SyncResult<ImapCredentials> {
        if self.use_encrypted_fallback {
            if let Some(store) = &self.encrypted_store {
                let store = store.read().await;
                return store.get_imap(account_id).await;
            }
            return Err(SyncError::KeyringError(
                "No credential storage available".to_string(),
            ));
        }

        let key = format!("imap_account_{}", account_id.to_string());
        let entry = Entry::new(KEYRING_SERVICE, &key)?;
        let json = entry.get_password()?;
        let credentials: ImapCredentials = serde_json::from_str(&json)?;
        Ok(credentials)
    }

    /// Delete credentials for an account
    pub async fn delete(&self, account_id: Uuid) -> SyncResult<()> {
        if self.use_encrypted_fallback {
            if let Some(store) = &self.encrypted_store {
                let store = store.read().await;
                return store.delete(account_id).await;
            }
            return Err(SyncError::KeyringError(
                "No credential storage available".to_string(),
            ));
        }

        let oauth2_key = format!("oauth2_account_{}", account_id.to_string());
        if let Ok(entry) = Entry::new(KEYRING_SERVICE, &oauth2_key) {
            let _ = entry.delete_credential();
        }

        let imap_key = format!("imap_account_{}", account_id.to_string());
        if let Ok(entry) = Entry::new(KEYRING_SERVICE, &imap_key) {
            let _ = entry.delete_credential();
        }

        log::info!("Deleted credentials for account {}", account_id);
        Ok(())
    }

    /// Check if credentials exist for an account
    pub async fn has_credentials(&self, account_id: Uuid) -> bool {
        if self.use_encrypted_fallback {
            if let Some(store) = &self.encrypted_store {
                let store = store.read().await;
                return store.has_credentials(account_id).await;
            }
            return false;
        }

        let oauth2_key = format!("oauth2_account_{}", account_id.to_string());
        if let Ok(entry) = Entry::new(KEYRING_SERVICE, &oauth2_key) {
            if entry.get_password().is_ok() {
                return true;
            }
        }

        let imap_key = format!("imap_account_{}", account_id.to_string());
        if let Ok(entry) = Entry::new(KEYRING_SERVICE, &imap_key) {
            if entry.get_password().is_ok() {
                return true;
            }
        }

        false
    }
}

/// OAuth2 authentication helper
pub struct OAuth2Helper;

impl OAuth2Helper {
    /// Start OAuth2 flow and return authorization URL, CSRF token, and PKCE verifier
    pub fn start_oauth2_flow(
        provider: &str,
        redirect_uri: &str,
    ) -> SyncResult<(String, String, String)> {
        match provider {
            "gmail" => Self::start_gmail_oauth2(redirect_uri),
            "office365" => Self::start_office365_oauth2(redirect_uri),
            _ => Err(SyncError::NotSupported(format!(
                "OAuth2 not supported for provider: {}",
                provider
            ))),
        }
    }

    fn start_gmail_oauth2(redirect_uri: &str) -> SyncResult<(String, String, String)> {
        use oauth2::basic::BasicClient;
        use oauth2::{
            AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope,
            TokenUrl,
        };

        let client_id = env!("GMAIL_CLIENT_ID").to_string();
        let client_secret = env!("GMAIL_CLIENT_SECRET").to_string();

        let client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
                .map_err(|e| SyncError::OAuth2Error(e.to_string()))?,
            Some(
                TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
                    .map_err(|e| SyncError::OAuth2Error(e.to_string()))?,
            ),
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect_uri.to_string())
                .map_err(|e| SyncError::OAuth2Error(e.to_string()))?,
        );

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/gmail.modify".to_string(),
            ))
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/gmail.settings.basic".to_string(),
            ))
            .set_pkce_challenge(pkce_challenge)
            .url();

        Ok((
            auth_url.to_string(),
            csrf_token.secret().to_string(),
            pkce_verifier.secret().to_string(),
        ))
    }

    fn start_office365_oauth2(redirect_uri: &str) -> SyncResult<(String, String, String)> {
        use oauth2::basic::BasicClient;
        use oauth2::{
            AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope,
            TokenUrl,
        };

        let client_id = env!("OFFICE365_CLIENT_ID").to_string();
        let client_secret = env!("OFFICE365_CLIENT_SECRET").to_string();
        let tenant = env!("OFFICE365_TENANT").to_string();

        let auth_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize",
            tenant
        );
        let token_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            tenant
        );

        let client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new(auth_url).map_err(|e| SyncError::OAuth2Error(e.to_string()))?,
            Some(TokenUrl::new(token_url).map_err(|e| SyncError::OAuth2Error(e.to_string()))?),
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect_uri.to_string())
                .map_err(|e| SyncError::OAuth2Error(e.to_string()))?,
        );

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(
                "https://graph.microsoft.com/Mail.ReadWrite".to_string(),
            ))
            .add_scope(Scope::new(
                "https://graph.microsoft.com/Mail.Send".to_string(),
            ))
            .add_scope(Scope::new("offline_access".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        Ok((
            auth_url.to_string(),
            csrf_token.secret().to_string(),
            pkce_verifier.secret().to_string(),
        ))
    }

    /// Exchange authorization code for access token with PKCE verifier
    pub async fn exchange_code(
        provider: &str,
        code: &str,
        redirect_uri: &str,
        pkce_verifier: &str,
    ) -> SyncResult<OAuth2Credentials> {
        match provider {
            "gmail" => Self::exchange_gmail_code(code, redirect_uri, pkce_verifier).await,
            "office365" => Self::exchange_office365_code(code, redirect_uri, pkce_verifier).await,
            _ => Err(SyncError::NotSupported(format!(
                "OAuth2 not supported for provider: {}",
                provider
            ))),
        }
    }

    async fn exchange_gmail_code(
        code: &str,
        redirect_uri: &str,
        pkce_verifier: &str,
    ) -> SyncResult<OAuth2Credentials> {
        use oauth2::basic::BasicClient;
        use oauth2::{
            AuthorizationCode, ClientId, ClientSecret, PkceCodeVerifier, RedirectUrl,
            TokenResponse, TokenUrl,
        };

        let client_id = env!("GMAIL_CLIENT_ID").to_string();
        let client_secret = env!("GMAIL_CLIENT_SECRET").to_string();

        let client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            oauth2::AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
                .map_err(|e| SyncError::OAuth2Error(e.to_string()))?,
            Some(
                TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
                    .map_err(|e| SyncError::OAuth2Error(e.to_string()))?,
            ),
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect_uri.to_string())
                .map_err(|e| SyncError::OAuth2Error(e.to_string()))?,
        );

        let token_result = client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier.to_string()))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|e| SyncError::OAuth2Error(e.to_string()))?;

        let expires_at = token_result
            .expires_in()
            .map(|d| chrono::Utc::now() + chrono::Duration::seconds(d.as_secs() as i64));

        Ok(OAuth2Credentials {
            access_token: token_result.access_token().secret().clone(),
            refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
            token_type: "Bearer".to_string(),
            expires_at,
            scopes: token_result
                .scopes()
                .map(|scopes| scopes.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default(),
        })
    }

    async fn exchange_office365_code(
        code: &str,
        redirect_uri: &str,
        pkce_verifier: &str,
    ) -> SyncResult<OAuth2Credentials> {
        use oauth2::basic::BasicClient;
        use oauth2::{
            AuthorizationCode, ClientId, ClientSecret, PkceCodeVerifier, RedirectUrl,
            TokenResponse, TokenUrl,
        };

        let client_id = env!("OFFICE365_CLIENT_ID").to_string();
        let client_secret = env!("OFFICE365_CLIENT_SECRET").to_string();
        let tenant = env!("OFFICE365_TENANT").to_string();

        let auth_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize",
            tenant
        );
        let token_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            tenant
        );

        let client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            oauth2::AuthUrl::new(auth_url).map_err(|e| SyncError::OAuth2Error(e.to_string()))?,
            Some(TokenUrl::new(token_url).map_err(|e| SyncError::OAuth2Error(e.to_string()))?),
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect_uri.to_string())
                .map_err(|e| SyncError::OAuth2Error(e.to_string()))?,
        );

        let token_result = client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier.to_string()))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|e| SyncError::OAuth2Error(e.to_string()))?;

        let expires_at = token_result
            .expires_in()
            .map(|d| chrono::Utc::now() + chrono::Duration::seconds(d.as_secs() as i64));

        Ok(OAuth2Credentials {
            access_token: token_result.access_token().secret().clone(),
            refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
            token_type: "Bearer".to_string(),
            expires_at,
            scopes: token_result
                .scopes()
                .map(|scopes| scopes.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default(),
        })
    }

    /// Refresh an expired access token
    pub async fn refresh_token(
        provider: &str,
        refresh_token: &str,
    ) -> SyncResult<OAuth2Credentials> {
        match provider {
            "gmail" => Self::refresh_gmail_token(refresh_token).await,
            "office365" => Self::refresh_office365_token(refresh_token).await,
            _ => Err(SyncError::NotSupported(format!(
                "OAuth2 not supported for provider: {}",
                provider
            ))),
        }
    }

    async fn refresh_gmail_token(refresh_token: &str) -> SyncResult<OAuth2Credentials> {
        use oauth2::basic::BasicClient;
        use oauth2::{ClientId, ClientSecret, RefreshToken, TokenResponse, TokenUrl};

        let client_id = env!("GMAIL_CLIENT_ID").to_string();
        let client_secret = env!("GMAIL_CLIENT_SECRET").to_string();

        let client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            oauth2::AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
                .map_err(|e| SyncError::OAuth2Error(e.to_string()))?,
            Some(
                TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
                    .map_err(|e| SyncError::OAuth2Error(e.to_string()))?,
            ),
        );

        let token_result = client
            .exchange_refresh_token(&RefreshToken::new(refresh_token.to_string()))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|e| SyncError::OAuth2Error(e.to_string()))?;

        let expires_at = token_result
            .expires_in()
            .map(|d| chrono::Utc::now() + chrono::Duration::seconds(d.as_secs() as i64));

        Ok(OAuth2Credentials {
            access_token: token_result.access_token().secret().clone(),
            refresh_token: Some(refresh_token.to_string()),
            token_type: "Bearer".to_string(),
            expires_at,
            scopes: token_result
                .scopes()
                .map(|scopes| scopes.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default(),
        })
    }

    async fn refresh_office365_token(refresh_token: &str) -> SyncResult<OAuth2Credentials> {
        use oauth2::basic::BasicClient;
        use oauth2::{ClientId, ClientSecret, RefreshToken, TokenResponse, TokenUrl};

        let client_id = env!("OFFICE365_CLIENT_ID").to_string();
        let client_secret = env!("OFFICE365_CLIENT_SECRET").to_string();
        let tenant = env!("OFFICE365_TENANT").to_string();

        let auth_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize",
            tenant
        );
        let token_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            tenant
        );

        let client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            oauth2::AuthUrl::new(auth_url).map_err(|e| SyncError::OAuth2Error(e.to_string()))?,
            Some(TokenUrl::new(token_url).map_err(|e| SyncError::OAuth2Error(e.to_string()))?),
        );

        let token_result = client
            .exchange_refresh_token(&RefreshToken::new(refresh_token.to_string()))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|e| SyncError::OAuth2Error(e.to_string()))?;

        let expires_at = token_result
            .expires_in()
            .map(|d| chrono::Utc::now() + chrono::Duration::seconds(d.as_secs() as i64));

        Ok(OAuth2Credentials {
            access_token: token_result.access_token().secret().clone(),
            refresh_token: Some(refresh_token.to_string()),
            token_type: "Bearer".to_string(),
            expires_at,
            scopes: token_result
                .scopes()
                .map(|scopes| scopes.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default(),
        })
    }
}
