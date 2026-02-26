use thiserror::Error;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("OAuth2 error: {0}")]
    OAuth2Error(String),

    #[error("IMAP error: {0}")]
    ImapError(String),

    #[error("Gmail API error: {0}")]
    GmailError(String),

    #[error("Office365 API error: {0}")]
    Office365Error(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Record not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Keyring error: {0}")]
    KeyringError(String),

    #[error("Attachment error: {0}")]
    AttachmentError(String),

    #[error("Folder not found: {0}")]
    FolderNotFound(String),

    #[error("Email not found: {0}")]
    EmailNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Sync in progress: {0}")]
    SyncInProgress(String),

    #[error("Operation not supported: {0}")]
    NotSupported(String),

    #[error("Sync token expired: {0}")]
    SyncTokenExpired(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    /// Network timeouts, rate limits — safe to retry
    Transient,
    /// Provider-side errors — retry with backoff
    Provider,
    /// Invalid data from provider — skip item, continue sync
    DataCorruption,
    /// Missing credentials, bad settings — requires user action
    Configuration,
    /// Unrecoverable errors (e.g., database failure)
    Fatal,
}

impl SyncError {
    pub fn category(&self) -> ErrorCategory {
        match self {
            SyncError::NetworkError(_) | SyncError::ReqwestError(_) => ErrorCategory::Transient,
            SyncError::GmailError(_)
            | SyncError::Office365Error(_)
            | SyncError::ImapError(_) => ErrorCategory::Provider,
            SyncError::ParseError(_) | SyncError::JsonError(_) => ErrorCategory::DataCorruption,
            SyncError::AuthenticationError(_)
            | SyncError::OAuth2Error(_)
            | SyncError::InvalidConfiguration(_)
            | SyncError::KeyringError(_) => ErrorCategory::Configuration,
            SyncError::DatabaseError(_) | SyncError::IoError(_) => ErrorCategory::Fatal,
            SyncError::SyncInProgress(_) | SyncError::NotSupported(_) => ErrorCategory::Transient,
            SyncError::SyncTokenExpired(_) => ErrorCategory::Transient,
            _ => ErrorCategory::Transient,
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self.category(),
            ErrorCategory::Transient | ErrorCategory::Provider
        )
    }

    pub(crate) fn timeout(_p0: String) -> SyncError {
        todo!()
    }
}

impl From<async_imap::error::Error> for SyncError {
    fn from(err: async_imap::error::Error) -> Self {
        SyncError::ImapError(err.to_string())
    }
}

impl From<keyring::Error> for SyncError {
    fn from(err: keyring::Error) -> Self {
        SyncError::KeyringError(err.to_string())
    }
}

pub type SyncResult<T> = Result<T, SyncError>;
