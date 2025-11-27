use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::email_dto::{AttachmentInfo, EmailDetail, EmailListItem};

/// Conversation model representing an email thread
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: Uuid,
    pub remote_id: String,
    pub message_count: i64,
    pub ai_cache: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for Conversation {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        let id_str: String = row.try_get("id")?;
        let id = Uuid::parse_str(&id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        Ok(Conversation {
            id,
            remote_id: row.try_get("remote_id")?,
            message_count: row.try_get("message_count")?,
            ai_cache: row.try_get("ai_cache")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

/// DTO for conversation list items with minimal email data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationListItem {
    pub id: String,
    pub message_count: i64,
    pub ai_cache: Option<String>,
    pub messages: Vec<EmailListItem>,
}

/// DTO for conversation detail with full email data and attachments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationDetail {
    pub id: String,
    pub message_count: i64,
    pub ai_cache: Option<String>,
    pub attachments: Vec<AttachmentInfo>,
    pub messages: Vec<EmailDetail>,
}

impl Conversation {
    /// Convert Conversation to ConversationListItem with associated emails
    pub fn to_list_item(self, messages: Vec<EmailListItem>) -> ConversationListItem {
        ConversationListItem {
            id: self.id.to_string(),
            message_count: self.message_count,
            ai_cache: self.ai_cache,
            messages,
        }
    }

    /// Convert Conversation to ConversationDetail with full email data and attachments
    pub fn to_detail(
        self,
        messages: Vec<EmailDetail>,
        attachments: Vec<AttachmentInfo>,
    ) -> ConversationDetail {
        ConversationDetail {
            id: self.id.to_string(),
            message_count: self.message_count,
            ai_cache: self.ai_cache,
            attachments,
            messages,
        }
    }
}
