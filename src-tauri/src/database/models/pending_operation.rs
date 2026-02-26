use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PendingOperationType {
    MarkRead,
    MarkUnread,
    Flag,
    Unflag,
    Move,
    Delete,
    PermanentDelete,
    CreateDraft,
    UpdateDraft,
    Send,
}

impl PendingOperationType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::MarkRead => "mark_read",
            Self::MarkUnread => "mark_unread",
            Self::Flag => "flag",
            Self::Unflag => "unflag",
            Self::Move => "move",
            Self::Delete => "delete",
            Self::PermanentDelete => "permanent_delete",
            Self::CreateDraft => "create_draft",
            Self::UpdateDraft => "update_draft",
            Self::Send => "send",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "mark_read" => Some(Self::MarkRead),
            "mark_unread" => Some(Self::MarkUnread),
            "flag" => Some(Self::Flag),
            "unflag" => Some(Self::Unflag),
            "move" => Some(Self::Move),
            "delete" => Some(Self::Delete),
            "permanent_delete" => Some(Self::PermanentDelete),
            "create_draft" => Some(Self::CreateDraft),
            "update_draft" => Some(Self::UpdateDraft),
            "send" => Some(Self::Send),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PendingOperationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

impl PendingOperationStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "in_progress" => Some(Self::InProgress),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingOperation {
    pub id: Uuid,
    pub account_id: Uuid,
    pub email_id: Option<Uuid>,
    pub folder_id: Option<Uuid>,
    pub operation_type: String,
    pub payload: String,
    pub status: String,
    pub retry_count: i64,
    pub max_retries: i64,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for PendingOperation {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        let parse_uuid = |s: &str| Uuid::parse_str(s).map_err(|e| sqlx::Error::Decode(Box::new(e)));

        let id: String = row.try_get("id")?;
        let account_id: String = row.try_get("account_id")?;
        let email_id: Option<String> = row.try_get("email_id")?;
        let folder_id: Option<String> = row.try_get("folder_id")?;

        Ok(Self {
            id: parse_uuid(&id)?,
            account_id: parse_uuid(&account_id)?,
            email_id: email_id.as_deref().map(parse_uuid).transpose()?,
            folder_id: folder_id.as_deref().map(parse_uuid).transpose()?,
            operation_type: row.try_get("operation_type")?,
            payload: row.try_get("payload")?,
            status: row.try_get("status")?,
            retry_count: row.try_get("retry_count")?,
            max_retries: row.try_get("max_retries")?,
            error_message: row.try_get("error_message")?,
            created_at: row.try_get("created_at")?,
            completed_at: row.try_get("completed_at")?,
            expires_at: row.try_get("expires_at")?,
        })
    }
}

impl PendingOperation {
    pub fn new(
        account_id: Uuid,
        email_id: Option<Uuid>,
        folder_id: Option<Uuid>,
        operation_type: PendingOperationType,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            account_id,
            email_id,
            folder_id,
            operation_type: operation_type.as_str().to_string(),
            payload: payload.to_string(),
            status: PendingOperationStatus::Pending.as_str().to_string(),
            retry_count: 0,
            max_retries: 3,
            error_message: None,
            created_at: Utc::now(),
            completed_at: None,
            expires_at: None,
        }
    }

    pub fn with_expires_at(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn parsed_operation_type(&self) -> Option<PendingOperationType> {
        PendingOperationType::from_str(&self.operation_type)
    }

    pub fn parsed_status(&self) -> Option<PendingOperationStatus> {
        PendingOperationStatus::from_str(&self.status)
    }

    pub fn parsed_payload(&self) -> serde_json::Value {
        serde_json::from_str(&self.payload).unwrap_or(serde_json::Value::Object(Default::default()))
    }
}
