use crate::database::models::email::{Email, EmailAddress};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use uuid::Uuid;

// Re-export FolderType from database models for consistency
pub use crate::database::models::folder::FolderType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncFolder {
    pub id: Option<Uuid>,
    pub account_id: Uuid,
    pub name: String,
    pub folder_type: FolderType,
    pub remote_id: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub parent_id: Option<Uuid>,
    pub attributes: Vec<String>,
    pub unread_count: i32,
    pub total_count: i32,
    pub expanded: bool,
    pub hidden: bool,
    pub synced_at: Option<DateTime<Utc>>,
    pub sync_interval: i64,
}

// Note: FolderType enum and its implementations have been moved to
// database::models::folder and are re-exported above for consistency

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEmail {
    pub id: Option<Uuid>,
    pub account_id: Uuid,
    pub folder_id: Uuid,
    pub message_id: String,
    pub conversation_id: Option<String>,
    pub remote_id: String,
    pub from: EmailAddress,
    pub to: Vec<EmailAddress>,
    pub cc: Vec<EmailAddress>,
    pub bcc: Vec<EmailAddress>,
    pub reply_to: Option<EmailAddress>,
    pub subject: Option<String>,
    pub snippet: Option<String>,
    pub body_plain: Option<String>,
    pub body_html: Option<String>,
    pub other_mails: Option<String>,
    pub category: Option<String>,
    pub ai_cache: Option<String>,
    pub received_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub flags: Vec<String>,
    pub headers: Option<serde_json::Value>,
    pub size: i64,
    pub has_attachments: bool,
    pub attachments: Vec<SyncAttachment>,
    pub change_key: Option<String>,
    pub last_modified_at: Option<DateTime<Utc>>,
}

impl SyncEmail {
    pub fn from_email(email: &Email) -> Self {
        Self {
            id: Some(email.id),
            account_id: email.account_id,
            folder_id: email.folder_id,
            message_id: email.message_id.clone(),
            conversation_id: email.conversation_id.clone(),
            remote_id: email.remote_id.clone().unwrap_or_default(),
            from: email.from().clone(),
            to: email.to().to_vec(),
            cc: email.cc().to_vec(),
            bcc: email.bcc().to_vec(),
            reply_to: Some(email.reply_to().clone()),
            subject: email.subject.clone(),
            snippet: email.snippet.clone(),
            body_plain: email.body_plain.clone(),
            body_html: email.body_html.clone(),
            other_mails: email.other_mails.clone(),
            category: email.category.clone(),
            ai_cache: email.ai_cache.clone(),
            received_at: email.received_at,
            sent_at: email.sent_at,
            flags: Vec::new(),
            size: email.size,
            has_attachments: email.has_attachments,
            headers: Some({
                if let Some(hdrs) = &email.headers {
                    serde_json::to_value(hdrs).unwrap_or(serde_json::Value::Null)
                } else {
                    serde_json::Value::Null
                }
            }),
            attachments: Vec::new(),
            change_key: email.change_key.clone(),
            last_modified_at: email.last_modified_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncAttachment {
    pub id: Option<Uuid>,
    pub email_id: Option<Uuid>,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub hash: String,
    pub cache_path: Option<String>,
    pub remote_url: Option<String>,
    pub remote_path: Option<String>,
    pub is_inline: bool,
    pub is_cached: bool,
    pub content_id: Option<String>,
    /// Optional attachment data (for immediate caching during sync)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct SyncDiff {
    /// New emails to be inserted
    pub added: Vec<SyncEmail>,
    /// Existing emails that were modified
    pub modified: Vec<SyncEmail>,
    /// Remote IDs of emails that were deleted or moved out
    pub deleted: Vec<String>,
    /// Delta token for next incremental sync (provider-specific)
    pub next_sync_token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SyncState {
    pub account_id: Uuid,
    pub folder_id: Option<Uuid>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub next_sync_at: Option<DateTime<Utc>>,
    pub last_uid: Option<i64>,
    pub sync_token: Option<String>,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SyncStatus {
    Idle,
    Syncing,
    Error,
    Paused,
}

impl Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            SyncStatus::Idle => "idle".to_string(),
            SyncStatus::Syncing => "syncing".to_string(),
            SyncStatus::Error => "error".to_string(),
            SyncStatus::Paused => "paused".to_string(),
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug, Clone)]
pub struct AccountSettings {
    pub imap_host: Option<String>,
    pub imap_port: Option<u16>,
    pub imap_use_tls: Option<bool>,
    pub imap_username: Option<String>,

    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_use_tls: Option<bool>,
    pub smtp_username: Option<String>,

    pub sync_enabled: bool,
    pub sync_interval: Option<u64>,
    pub sync_on_startup: bool,

    pub cache_attachments: bool,
    pub max_attachment_cache_size: Option<i64>, // in bytes
    pub auto_download_inline: bool,

    pub provider_settings: Option<serde_json::Value>,
}

impl Default for AccountSettings {
    fn default() -> Self {
        Self {
            imap_host: None,
            imap_port: None,
            imap_use_tls: Some(true),
            imap_username: None,
            smtp_host: None,
            smtp_port: None,
            smtp_use_tls: Some(true),
            smtp_username: None,
            sync_enabled: true,
            sync_interval: Some(5 * 60),
            sync_on_startup: true,
            cache_attachments: true,
            max_attachment_cache_size: Some(1024 * 1024 * 1024),
            auto_download_inline: true,
            provider_settings: None,
        }
    }
}

impl serde::Serialize for AccountSettings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AccountSettings", 15)?;
        state.serialize_field("imap_host", &self.imap_host)?;
        state.serialize_field("imap_port", &self.imap_port)?;
        state.serialize_field("imap_use_tls", &self.imap_use_tls)?;
        state.serialize_field("imap_username", &self.imap_username)?;
        state.serialize_field("smtp_host", &self.smtp_host)?;
        state.serialize_field("smtp_port", &self.smtp_port)?;
        state.serialize_field("smtp_use_tls", &self.smtp_use_tls)?;
        state.serialize_field("smtp_username", &self.smtp_username)?;
        state.serialize_field("sync_enabled", &self.sync_enabled)?;
        state.serialize_field("sync_interval", &self.sync_interval)?;
        state.serialize_field("sync_on_startup", &self.sync_on_startup)?;
        state.serialize_field("cache_attachments", &self.cache_attachments)?;
        state.serialize_field("max_attachment_cache_size", &self.max_attachment_cache_size)?;
        state.serialize_field("auto_download_inline", &self.auto_download_inline)?;
        state.serialize_field("provider_settings", &self.provider_settings)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for AccountSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            ImapHost,
            ImapPort,
            ImapUseTls,
            ImapUsername,
            SmtpHost,
            SmtpPort,
            SmtpUseTls,
            SmtpUsername,
            SyncEnabled,
            SyncInterval,
            SyncOnStartup,
            CacheAttachments,
            MaxAttachmentCacheSize,
            AutoDownloadInline,
            ProviderSettings,
        }

        struct AccountSettingsVisitor;

        impl<'de> serde::de::Visitor<'de> for AccountSettingsVisitor {
            type Value = AccountSettings;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct AccountSettings")
            }

            fn visit_map<V>(self, mut map: V) -> Result<AccountSettings, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut imap_host = None;
                let mut imap_port = None;
                let mut imap_use_tls = None;
                let mut imap_username = None;
                let mut smtp_host = None;
                let mut smtp_port = None;
                let mut smtp_use_tls = None;
                let mut smtp_username = None;
                let mut sync_enabled = None;
                let mut sync_interval = None;
                let mut sync_on_startup = None;
                let mut cache_attachments = None;
                let mut max_attachment_cache_size = None;
                let mut auto_download_inline = None;
                let mut provider_settings = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::ImapHost => imap_host = map.next_value()?,
                        Field::ImapPort => imap_port = map.next_value()?,
                        Field::ImapUseTls => imap_use_tls = map.next_value()?,
                        Field::ImapUsername => imap_username = map.next_value()?,
                        Field::SmtpHost => smtp_host = map.next_value()?,
                        Field::SmtpPort => smtp_port = map.next_value()?,
                        Field::SmtpUseTls => smtp_use_tls = map.next_value()?,
                        Field::SmtpUsername => smtp_username = map.next_value()?,
                        Field::SyncEnabled => sync_enabled = map.next_value()?,
                        Field::SyncInterval => sync_interval = map.next_value()?,
                        Field::SyncOnStartup => sync_on_startup = map.next_value()?,
                        Field::CacheAttachments => cache_attachments = map.next_value()?,
                        Field::MaxAttachmentCacheSize => {
                            max_attachment_cache_size = map.next_value()?
                        }
                        Field::AutoDownloadInline => auto_download_inline = map.next_value()?,
                        Field::ProviderSettings => provider_settings = map.next_value()?,
                    }
                }

                Ok(AccountSettings {
                    imap_host,
                    imap_port,
                    imap_use_tls,
                    imap_username,
                    smtp_host,
                    smtp_port,
                    smtp_use_tls,
                    smtp_username,
                    sync_enabled: sync_enabled.unwrap_or(true),
                    sync_interval,
                    sync_on_startup: sync_on_startup.unwrap_or(true),
                    cache_attachments: cache_attachments.unwrap_or(true),
                    max_attachment_cache_size,
                    auto_download_inline: auto_download_inline.unwrap_or(true),
                    provider_settings,
                })
            }
        }

        const FIELDS: &[&str] = &[
            "imap_host",
            "imap_port",
            "imap_use_tls",
            "imap_username",
            "smtp_host",
            "smtp_port",
            "smtp_use_tls",
            "smtp_username",
            "sync_enabled",
            "sync_interval",
            "sync_on_startup",
            "cache_attachments",
            "max_attachment_cache_size",
            "auto_download_inline",
            "provider_settings",
        ];
        deserializer.deserialize_struct("AccountSettings", FIELDS, AccountSettingsVisitor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Credentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImapCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub enum ProviderCredentials {
    OAuth2(OAuth2Credentials),
    Imap(ImapCredentials),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailRecipient {
    pub address: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAttachmentData {
    pub filename: String,
    pub content: Vec<u8>,
    pub content_type: Option<String>,
}
