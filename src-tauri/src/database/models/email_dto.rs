/// DTOs for email data transfer to frontend
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::attachment::Attachment;
use super::email::{Email, EmailAddress};
use super::label::Label;

/// Minimal email data for list views
/// Optimized for performance with only essential fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailListItem {
    pub id: Uuid,
    pub account_id: Uuid,
    pub folder_id: Uuid,
    pub message_id: String,
    pub conversation_id: Option<String>,

    pub from: EmailAddress,
    pub to: Vec<EmailAddress>,
    pub cc: Vec<EmailAddress>,
    pub bcc: Vec<EmailAddress>,

    pub subject: Option<String>,
    pub snippet: Option<String>,
    pub category: Option<String>,

    pub received_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,

    pub is_read: bool,
    pub is_draft: bool,
    pub is_flagged: bool,
    pub sync_status: String,
    pub has_attachments: bool,
    pub size: i64,

    pub labels: Vec<LabelInfo>,
}

impl EmailListItem {
    pub fn from_email(email: &Email, labels: Vec<LabelInfo>) -> Self {
        Self {
            id: email.id,
            account_id: email.account_id,
            folder_id: email.folder_id,
            message_id: email.message_id.clone(),
            conversation_id: email.conversation_id.clone(),
            from: email.from.0.clone(),
            to: email.to.0.clone(),
            cc: email.cc.0.clone(),
            bcc: email.bcc.0.clone(),
            subject: email.subject.clone(),
            snippet: email.snippet.clone(),
            category: email.category.clone(),
            received_at: email.received_at,
            sent_at: email.sent_at,
            is_read: email.is_read,
            is_draft: email.is_draft,
            is_flagged: email.is_flagged,
            sync_status: email.sync_status.clone(),
            has_attachments: email.has_attachments,
            size: email.size,
            labels,
        }
    }
}

/// Full email data for detail view
/// Includes all fields and related data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailDetail {
    pub id: Uuid,
    pub account_id: Uuid,
    pub folder_id: Uuid,
    pub message_id: String,
    pub conversation_id: Option<String>,
    pub remote_id: Option<String>,

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

    pub headers: Option<String>,
    pub size: i64,

    pub received_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub scheduled_send_at: Option<DateTime<Utc>>,

    pub is_read: bool,
    pub is_flagged: bool,
    pub is_draft: bool,
    pub has_attachments: bool,
    pub is_deleted: bool,

    pub sync_status: String,
    pub body_fetch_attempts: i64,
    pub last_body_fetch_attempt: Option<DateTime<Utc>>,

    pub tracking_blocked: bool,
    pub images_blocked: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub labels: Vec<LabelInfo>,
    pub attachments: Vec<AttachmentInfo>,
}

impl EmailDetail {
    pub fn from_email(
        email: &Email,
        labels: Vec<LabelInfo>,
        attachments: Vec<AttachmentInfo>,
    ) -> Self {
        Self {
            id: email.id,
            account_id: email.account_id,
            folder_id: email.folder_id,
            message_id: email.message_id.clone(),
            conversation_id: email.conversation_id.clone(),
            remote_id: email.remote_id.clone(),
            from: email.from.0.clone(),
            to: email.to.0.clone(),
            cc: email.cc.0.clone(),
            bcc: email.bcc.0.clone(),
            reply_to: email.reply_to.as_ref().map(|r| r.0.clone()),
            subject: email.subject.clone(),
            snippet: email.snippet.clone(),
            body_plain: email.body_plain.clone(),
            body_html: email.body_html.clone(),
            other_mails: email.other_mails.clone(),
            category: email.category.clone(),
            ai_cache: email.ai_cache.clone(),
            headers: email.headers.clone(),
            size: email.size,
            received_at: email.received_at,
            sent_at: email.sent_at,
            scheduled_send_at: email.scheduled_send_at,
            is_read: email.is_read,
            is_flagged: email.is_flagged,
            is_draft: email.is_draft,
            has_attachments: email.has_attachments,
            is_deleted: email.is_deleted,
            sync_status: email.sync_status.clone(),
            body_fetch_attempts: email.body_fetch_attempts,
            last_body_fetch_attempt: email.last_body_fetch_attempt,
            tracking_blocked: email.tracking_blocked,
            images_blocked: email.images_blocked,
            created_at: email.created_at,
            updated_at: email.updated_at,
            labels,
            attachments,
        }
    }
}

/// Lightweight label information for email DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelInfo {
    pub id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub icon: Option<String>,
}

impl From<&Label> for LabelInfo {
    fn from(label: &Label) -> Self {
        Self {
            id: label.id,
            name: label.name.clone(),
            color: label.color.clone(),
            icon: label.icon.clone(),
        }
    }
}

/// Attachment information for email DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentInfo {
    pub id: Uuid,
    pub email_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub is_inline: bool,
    pub is_cached: bool,
    pub content_id: Option<String>,
    pub cache_path: Option<String>,
    pub hash: String,
}

impl From<&Attachment> for AttachmentInfo {
    fn from(attachment: &Attachment) -> Self {
        Self {
            id: attachment.id,
            email_id: attachment.email_id,
            filename: attachment.filename.clone(),
            content_type: attachment.content_type.clone(),
            size: attachment.size,
            is_inline: attachment.is_inline,
            is_cached: attachment.is_cached,
            content_id: attachment.content_id.clone(),
            cache_path: attachment.cache_path.clone(),
            hash: attachment.hash.clone(),
        }
    }
}
