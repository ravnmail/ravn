use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, Decode, Encode};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Decode, Encode)]
pub struct EmailAddress {
    pub address: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Decode, Encode)]
pub struct EmailAddressList(pub Vec<EmailAddress>);
impl sqlx::Type<sqlx::Sqlite> for EmailAddressList {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl IntoIterator for EmailAddressList {
    type Item = EmailAddress;
    type IntoIter = std::vec::IntoIter<EmailAddress>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a EmailAddressList {
    type Item = &'a EmailAddress;
    type IntoIter = std::slice::Iter<'a, EmailAddress>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl std::ops::Deref for EmailAddressList {
    type Target = Vec<EmailAddress>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EmailSyncStatus {
    HeadersOnly,
    FetchingBody,
    Synced,
    Error,
}

impl EmailSyncStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmailSyncStatus::HeadersOnly => "headers_only",
            EmailSyncStatus::FetchingBody => "fetching_body",
            EmailSyncStatus::Synced => "synced",
            EmailSyncStatus::Error => "error",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "headers_only" => EmailSyncStatus::HeadersOnly,
            "fetching_body" => EmailSyncStatus::FetchingBody,
            "error" => EmailSyncStatus::Error,
            _ => EmailSyncStatus::Synced,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub id: Uuid,
    pub account_id: Uuid,
    pub folder_id: Uuid,
    pub message_id: String,
    pub conversation_id: Option<String>,
    pub remote_id: Option<String>,

    pub from: Json<EmailAddress>,
    pub to: Json<Vec<EmailAddress>>,
    pub cc: Json<Vec<EmailAddress>>,
    pub bcc: Json<Vec<EmailAddress>>,
    pub reply_to: Option<Json<EmailAddress>>,

    pub subject: Option<String>,
    pub snippet: Option<String>,
    pub body_plain: Option<String>,
    pub body_html: Option<String>,
    pub other_mails: Option<String>,
    pub category: Option<String>,
    pub ai_cache: Option<String>,
    pub received_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub scheduled_send_at: Option<DateTime<Utc>>,
    pub is_read: bool,
    pub is_flagged: bool,
    pub has_attachments: bool,
    pub is_draft: bool,
    pub is_deleted: bool,
    pub headers: Option<String>,
    pub sync_status: String,
    pub tracking_blocked: bool,
    pub images_blocked: bool,
    pub body_fetch_attempts: i64,
    pub last_body_fetch_attempt: Option<DateTime<Utc>>,
    pub change_key: Option<String>,
    pub last_modified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub size: i64,
}

impl Email {
    pub fn get_sync_status(&self) -> EmailSyncStatus {
        EmailSyncStatus::from_str(&self.sync_status)
    }

    pub fn set_sync_status(&mut self, status: EmailSyncStatus) {
        self.sync_status = status.as_str().to_string();
    }

    // Helper method to format recipients for display
    pub fn format_recipients(&self, recipients: &[EmailAddress]) -> String {
        recipients
            .iter()
            .map(|r| match &r.name {
                Some(name) => format!("{} <{}>", name, r.address),
                None => r.address.clone(),
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    // Get primary recipient display string
    pub fn primary_recipient(&self) -> String {
        self.to
            .0
            .first()
            .map(|r| match &r.name {
                Some(name) => format!("{} <{}>", name, r.address),
                None => r.address.clone(),
            })
            .unwrap_or_default()
    }

    pub fn recipient_count(&self) -> usize {
        self.to.0.len() + self.cc.0.len() + self.bcc.0.len()
    }

    pub fn from(&self) -> &EmailAddress {
        &self.from.0
    }

    pub fn to(&self) -> &[EmailAddress] {
        &self.to.0
    }

    pub fn cc(&self) -> &[EmailAddress] {
        &self.cc.0
    }

    pub fn bcc(&self) -> &[EmailAddress] {
        &self.bcc.0
    }

    pub fn reply_to(&self) -> &EmailAddress {
        self.reply_to.as_ref().map(|j| &j.0).unwrap_or(&self.from.0)
    }
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for Email {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        let id_str: String = row.try_get("id")?;
        let id = Uuid::parse_str(&id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        let account_id_str: String = row.try_get("account_id")?;
        let account_id =
            Uuid::parse_str(&account_id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        let folder_id_str: String = row.try_get("folder_id")?;
        let folder_id =
            Uuid::parse_str(&folder_id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        Ok(Email {
            id,
            account_id,
            folder_id,
            message_id: row.try_get("message_id")?,
            conversation_id: row.try_get("conversation_id")?,
            remote_id: row.try_get("remote_id")?,
            from: {
                let json_str: String = row.try_get("from")?;
                Json(
                    serde_json::from_str(&json_str).map_err(|e| sqlx::Error::ColumnDecode {
                        index: "from".into(),
                        source: Box::new(e),
                    })?,
                )
            },
            to: {
                let json_str: String = row.try_get("to")?;
                Json(
                    serde_json::from_str(&json_str).map_err(|e| sqlx::Error::ColumnDecode {
                        index: "to".into(),
                        source: Box::new(e),
                    })?,
                )
            },
            cc: {
                let json_str: String = row.try_get("cc")?;
                Json(
                    serde_json::from_str(&json_str).map_err(|e| sqlx::Error::ColumnDecode {
                        index: "cc".into(),
                        source: Box::new(e),
                    })?,
                )
            },
            bcc: {
                let json_str: String = row.try_get("bcc")?;
                Json(
                    serde_json::from_str(&json_str).map_err(|e| sqlx::Error::ColumnDecode {
                        index: "bcc".into(),
                        source: Box::new(e),
                    })?,
                )
            },
            reply_to: match row.try_get::<Option<String>, _>("reply_to")? {
                Some(json_str) => {
                    let s = json_str.trim();
                    if s.eq_ignore_ascii_case("null") || s.is_empty() {
                        None
                    } else {
                        Some(Json(serde_json::from_str(s).map_err(|e| {
                            sqlx::Error::ColumnDecode {
                                index: "reply_to".into(),
                                source: Box::new(e),
                            }
                        })?))
                    }
                }
                None => None,
            },
            subject: row.try_get("subject")?,
            snippet: row.try_get("snippet")?,
            body_plain: row.try_get("body_plain")?,
            body_html: row.try_get("body_html")?,
            other_mails: row.try_get("other_mails")?,
            category: row.try_get("category")?,
            ai_cache: row.try_get("ai_cache")?,
            received_at: row.try_get("received_at")?,
            sent_at: row.try_get("sent_at")?,
            scheduled_send_at: row.try_get("scheduled_send_at")?,
            is_read: row.try_get("is_read")?,
            is_flagged: row.try_get("is_flagged")?,
            has_attachments: row.try_get("has_attachments")?,
            is_draft: row.try_get("is_draft")?,
            is_deleted: row.try_get("is_deleted")?,
            headers: row.try_get("headers")?,
            sync_status: row.try_get("sync_status")?,
            tracking_blocked: row.try_get("tracking_blocked")?,
            images_blocked: row.try_get("images_blocked")?,
            body_fetch_attempts: row.try_get("body_fetch_attempts")?,
            last_body_fetch_attempt: row.try_get("last_body_fetch_attempt")?,
            change_key: row.try_get("change_key").ok(),
            last_modified_at: row.try_get("last_modified_at").ok(),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            size: row.try_get("size")?,
        })
    }
}
