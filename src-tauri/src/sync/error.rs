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

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(String),
}

impl SyncError {
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
