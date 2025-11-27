use crate::database::models::email::EmailAddress;
use crate::sync::{
    auth::{CredentialStore, OAuth2Helper},
    error::{SyncError, SyncResult},
    provider::EmailProvider,
    types::*,
};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use mail_parser::{MessageParser, MimeHeaders};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

const GMAIL_API_BASE: &str = "https://gmail.googleapis.com/gmail/v1";

pub struct GmailProvider {
    account_id: Uuid,
    client: Client,
    access_token: Option<String>,
    credential_store: Arc<CredentialStore>,
}

#[derive(Debug, Deserialize)]
struct GmailLabelsResponse {
    labels: Vec<GmailLabel>,
}

#[derive(Debug, Deserialize)]
struct GmailLabel {
    id: String,
    name: String,
    // #[serde(rename = "type")]
    // label_type: Option<String>,
    #[serde(rename = "messagesTotal")]
    messages_total: Option<i32>,
    #[serde(rename = "messagesUnread")]
    messages_unread: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct GmailMessagesResponse {
    messages: Option<Vec<GmailMessageRef>>,
    // #[serde(rename = "nextPageToken")]
    // next_page_token: Option<String>,
    // #[serde(rename = "resultSizeEstimate")]
    // result_size_estimate: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct GmailMessageRef {
    id: String,
    // #[serde(rename = "threadId")]
    // thread_id: String,
}

#[derive(Debug, Deserialize)]
struct GmailMessage {
    id: String,
    #[serde(rename = "threadId")]
    thread_id: String,
    #[serde(rename = "labelIds")]
    label_ids: Option<Vec<String>>,
    snippet: Option<String>,
    // #[serde(rename = "historyId")]
    // history_id: Option<String>,
    #[serde(rename = "internalDate")]
    internal_date: Option<String>,
    payload: Option<GmailPayload>,
    #[serde(rename = "sizeEstimate")]
    size_estimate: Option<i64>,
    raw: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GmailPayload {
    // #[serde(rename = "partId")]
    // part_id: Option<String>,
    #[serde(rename = "mimeType")]
    mime_type: Option<String>,
    filename: Option<String>,
    headers: Option<Vec<GmailHeader>>,
    body: Option<GmailBody>,
    parts: Option<Vec<GmailPayload>>,
}

#[derive(Debug, Deserialize)]
struct GmailHeader {
    name: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct GmailBody {
    #[serde(rename = "attachmentId")]
    attachment_id: Option<String>,
    size: Option<i64>,
    data: Option<String>,
}

impl GmailProvider {
    pub fn new(account_id: Uuid, credential_store: Arc<CredentialStore>) -> SyncResult<Self> {
        Ok(Self {
            account_id,
            client: Client::new(),
            access_token: None,
            credential_store,
        })
    }

    async fn _ensure_token(&mut self) -> SyncResult<String> {
        if let Some(token) = &self.access_token {
            return Ok(token.clone());
        }

        let mut credentials = self.credential_store.get_oauth2(self.account_id).await?;

        if let Some(expires_at) = credentials.expires_at {
            if expires_at < Utc::now() {
                if let Some(refresh_token) = &credentials.refresh_token {
                    credentials = OAuth2Helper::refresh_token("gmail", refresh_token).await?;
                    self.credential_store
                        .store_oauth2(self.account_id, &credentials)
                        .await?;
                } else {
                    return Err(SyncError::AuthenticationError(
                        "Token expired and no refresh token available".to_string(),
                    ));
                }
            }
        }

        self.access_token = Some(credentials.access_token.clone());
        Ok(credentials.access_token)
    }

    fn map_label_to_folder_type(label_id: &str, label_name: &str) -> FolderType {
        match label_id {
            "INBOX" => FolderType::Inbox,
            "SENT" => FolderType::Sent,
            "DRAFT" => FolderType::Draft,
            "TRASH" => FolderType::Trash,
            "SPAM" => FolderType::Spam,
            "STARRED" => FolderType::Starred,
            _ => {
                if label_name.to_lowercase().contains("archive") {
                    FolderType::Archive
                } else {
                    FolderType::Custom
                }
            }
        }
    }

    fn parse_gmail_message(
        msg: &GmailMessage,
        folder_id: Uuid,
        account_id: Uuid,
    ) -> SyncResult<SyncEmail> {
        if let Some(raw) = &msg.raw {
            let decoded = general_purpose::URL_SAFE
                .decode(raw)
                .map_err(|e| SyncError::ParseError(format!("Base64 decode error: {}", e)))?;

            let parser = MessageParser::default();
            let message = parser
                .parse(&decoded)
                .ok_or_else(|| SyncError::ParseError("Failed to parse email".to_string()))?;

            return Self::parse_mail_message(&message, msg, folder_id, account_id);
        }

        Self::parse_from_payload(msg, folder_id, account_id)
    }

    fn parse_mail_message(
        message: &mail_parser::Message,
        gmail_msg: &GmailMessage,
        folder_id: Uuid,
        account_id: Uuid,
    ) -> SyncResult<SyncEmail> {
        let from = message
            .from()
            .and_then(|addr| addr.first())
            .map(|addr| EmailAddress {
                name: addr.name().map(|n| n.to_string()),
                address: addr.address().unwrap_or("").to_string(),
            })
            .unwrap_or_else(|| EmailAddress {
                name: None,
                address: "unknown@unknown.com".to_string(),
            });

        let to: Vec<EmailAddress> = message
            .to()
            .map(|addrs| {
                addrs
                    .iter()
                    .map(|addr| EmailAddress {
                        name: addr.name().map(|n| n.to_string()),
                        address: addr.address().unwrap_or("").to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let cc: Vec<EmailAddress> = message
            .cc()
            .map(|addrs| {
                addrs
                    .iter()
                    .map(|addr| EmailAddress {
                        name: addr.name().map(|n| n.to_string()),
                        address: addr.address().unwrap_or("").to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let bcc: Vec<EmailAddress> = message
            .bcc()
            .map(|addrs| {
                addrs
                    .iter()
                    .map(|addr| EmailAddress {
                        name: addr.name().map(|n| n.to_string()),
                        address: addr.address().unwrap_or("").to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let reply_to = message
            .reply_to()
            .and_then(|addrs| addrs.first())
            .map(|addr| EmailAddress {
                name: addr.name().map(|n| n.to_string()),
                address: addr.address().unwrap_or("").to_string(),
            });

        let subject = message.subject().map(|s| s.to_string());
        let body_html = message.body_html(0).map(|s| s.to_string());
        let body_plain = message.body_text(0).map(|s| s.to_string());

        let message_id = message
            .message_id()
            .map(|id| id.to_string())
            .unwrap_or_else(|| gmail_msg.id.clone());

        let received_at = if let Some(date_str) = &gmail_msg.internal_date {
            let millis: i64 = date_str.parse().unwrap_or(0);
            DateTime::from_timestamp_millis(millis).unwrap_or_else(|| Utc::now())
        } else {
            message
                .date()
                .and_then(|ts| DateTime::from_timestamp(ts.to_timestamp(), 0))
                .unwrap_or_else(|| Utc::now())
        };

        let flags: Vec<String> = gmail_msg
            .label_ids
            .as_ref()
            .map(|labels| labels.iter().map(|l| l.clone()).collect())
            .unwrap_or_default();

        let attachments: Vec<SyncAttachment> = message
            .attachments()
            .enumerate()
            .map(|(idx, att)| {
                let content = att.contents();
                let hash = format!("{:x}", md5::compute(content));
                let content_id = att.content_id().map(|s| s.to_string());

                let is_inline = if let (Some(cid), Some(html)) = (&content_id, &body_html) {
                    crate::sync::cid_utils::is_cid_referenced(html, cid)
                } else {
                    false
                };

                SyncAttachment {
                    id: None,
                    email_id: None,
                    filename: att
                        .attachment_name()
                        .unwrap_or(&format!("attachment_{}", idx))
                        .to_string(),
                    content_type: att
                        .content_type()
                        .map(|ct| ct.ctype())
                        .unwrap_or("application/octet-stream")
                        .to_string(),
                    size: content.len() as i64,
                    hash,
                    cache_path: None,
                    remote_url: None,
                    remote_path: Some(format!("{}:{}", gmail_msg.id, idx)),
                    is_inline,
                    is_cached: false,
                    content_id,
                    data: None,
                }
            })
            .collect();

        Ok(SyncEmail {
            id: None,
            account_id,
            folder_id,
            message_id,
            conversation_id: Some(gmail_msg.thread_id.clone()),
            remote_id: gmail_msg.id.clone(),
            from,
            to,
            cc,
            bcc,
            reply_to,
            subject,
            snippet: gmail_msg.snippet.clone(),
            body_plain,
            body_html,
            other_mails: None,
            category: None,
            ai_cache: None,
            received_at,
            sent_at: None,
            flags,
            headers: None,
            size: gmail_msg.size_estimate.unwrap_or(0),
            has_attachments: !attachments.is_empty(),
            attachments,
            change_key: None,
            last_modified_at: None,
        })
    }

    fn parse_from_payload(
        msg: &GmailMessage,
        folder_id: Uuid,
        account_id: Uuid,
    ) -> SyncResult<SyncEmail> {
        let payload = msg
            .payload
            .as_ref()
            .ok_or_else(|| SyncError::ParseError("No payload in message".to_string()))?;

        let mut from_addr = EmailAddress {
            name: None,
            address: "unknown@unknown.com".to_string(),
        };
        let mut to_addrs = Vec::new();
        let mut cc_addrs = Vec::new();
        let mut subject = None;
        let mut message_id = msg.id.clone();

        if let Some(headers) = &payload.headers {
            for header in headers {
                match header.name.to_lowercase().as_str() {
                    "from" => {
                        from_addr = Self::parse_email_address(&header.value);
                    }
                    "to" => {
                        to_addrs = Self::parse_email_addresses(&header.value);
                    }
                    "cc" => {
                        cc_addrs = Self::parse_email_addresses(&header.value);
                    }
                    "subject" => {
                        subject = Some(header.value.clone());
                    }
                    "message-id" => {
                        message_id = header.value.clone();
                    }
                    _ => {}
                }
            }
        }

        let received_at = if let Some(date_str) = &msg.internal_date {
            let millis: i64 = date_str.parse().unwrap_or(0);
            DateTime::from_timestamp_millis(millis).unwrap_or_else(|| Utc::now())
        } else {
            Utc::now()
        };

        let flags: Vec<String> = msg
            .label_ids
            .as_ref()
            .map(|labels| labels.iter().map(|l| l.clone()).collect())
            .unwrap_or_default();

        let (body_plain, body_html, attachments) = Self::extract_parts(payload);

        Ok(SyncEmail {
            id: None,
            account_id,
            folder_id,
            message_id,
            conversation_id: Some(msg.thread_id.clone()),
            remote_id: msg.id.clone(),
            from: from_addr,
            to: to_addrs,
            cc: cc_addrs,
            bcc: Vec::new(),
            reply_to: None,
            subject,
            snippet: msg.snippet.clone(),
            body_plain,
            body_html,
            other_mails: None,
            category: None,
            ai_cache: None,
            received_at,
            sent_at: None,
            flags,
            headers: None,
            size: msg.size_estimate.unwrap_or(0),
            has_attachments: !attachments.is_empty(),
            attachments,
            change_key: None,
            last_modified_at: None,
        })
    }

    fn parse_email_address(value: &str) -> EmailAddress {
        if let Some(start) = value.find('<') {
            if let Some(end) = value.find('>') {
                let name = value[..start].trim().trim_matches('"');
                let address = value[start + 1..end].trim();
                return EmailAddress {
                    name: if name.is_empty() {
                        None
                    } else {
                        Some(name.to_string())
                    },
                    address: address.to_string(),
                };
            }
        }
        EmailAddress {
            name: None,
            address: value.trim().to_string(),
        }
    }

    fn parse_email_addresses(value: &str) -> Vec<EmailAddress> {
        value
            .split(',')
            .map(|s| Self::parse_email_address(s.trim()))
            .collect()
    }

    fn extract_parts(
        payload: &GmailPayload,
    ) -> (Option<String>, Option<String>, Vec<SyncAttachment>) {
        let mut body_plain = None;
        let mut body_html = None;
        let mut attachments = Vec::new();

        if let Some(parts) = &payload.parts {
            for (_idx, part) in parts.iter().enumerate() {
                if let Some(mime_type) = &part.mime_type {
                    if mime_type == "text/plain" {
                        if let Some(body) = &part.body {
                            if let Some(data) = &body.data {
                                if let Ok(decoded) = general_purpose::URL_SAFE.decode(data) {
                                    body_plain = String::from_utf8(decoded).ok();
                                }
                            }
                        }
                    } else if mime_type == "text/html" {
                        if let Some(body) = &part.body {
                            if let Some(data) = &body.data {
                                if let Ok(decoded) = general_purpose::URL_SAFE.decode(data) {
                                    body_html = String::from_utf8(decoded).ok();
                                }
                            }
                        }
                    } else if let Some(filename) = &part.filename {
                        if !filename.is_empty() {
                            if let Some(body) = &part.body {
                                // Check if attachment is inline by examining headers
                                let is_inline = part
                                    .headers
                                    .as_ref()
                                    .map(|headers| {
                                        headers.iter().any(|h| {
                                            h.name.eq_ignore_ascii_case("Content-Disposition")
                                                && h.value.to_lowercase().contains("inline")
                                        }) || headers
                                            .iter()
                                            .any(|h| h.name.eq_ignore_ascii_case("Content-ID"))
                                    })
                                    .unwrap_or(false);

                                let content_id = part.headers.as_ref().and_then(|headers| {
                                    headers
                                        .iter()
                                        .find(|h| h.name.eq_ignore_ascii_case("Content-ID"))
                                        .map(|h| h.value.clone())
                                });

                                let temp_hash = if let Some(att_id) = &body.attachment_id {
                                    use sha2::{Digest, Sha256};
                                    let mut hasher = Sha256::new();
                                    hasher.update(att_id.as_bytes());
                                    format!("{:x}", hasher.finalize())
                                } else {
                                    String::new()
                                };

                                attachments.push(SyncAttachment {
                                    id: None,
                                    email_id: None,
                                    filename: filename.clone(),
                                    content_type: mime_type.clone(),
                                    size: body.size.unwrap_or(0),
                                    hash: temp_hash,
                                    cache_path: None,
                                    remote_url: None,
                                    remote_path: body.attachment_id.clone(),
                                    is_inline,
                                    is_cached: false,
                                    content_id,
                                    data: None,
                                });
                            }
                        }
                    }
                }

                if let Some(_nested_parts) = &part.parts {
                    let (nested_plain, nested_html, nested_attachments) = Self::extract_parts(part);
                    if body_plain.is_none() && nested_plain.is_some() {
                        body_plain = nested_plain;
                    }
                    if body_html.is_none() && nested_html.is_some() {
                        body_html = nested_html;
                    }
                    attachments.extend(nested_attachments);
                }
            }
        } else if let Some(body) = &payload.body {
            if let Some(data) = &body.data {
                if let Ok(decoded) = general_purpose::URL_SAFE.decode(data) {
                    if let Some(mime_type) = &payload.mime_type {
                        if mime_type == "text/plain" {
                            body_plain = String::from_utf8(decoded).ok();
                        } else if mime_type == "text/html" {
                            body_html = String::from_utf8(decoded).ok();
                        }
                    }
                }
            }
        }

        (body_plain, body_html, attachments)
    }
}

#[async_trait]
impl EmailProvider for GmailProvider {
    fn name(&self) -> &str {
        "Gmail"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn authenticate(&mut self, credentials: ProviderCredentials) -> SyncResult<()> {
        match credentials {
            ProviderCredentials::OAuth2(creds) => {
                self.access_token = Some(creds.access_token.clone());
                self.credential_store
                    .store_oauth2(self.account_id, &creds)
                    .await?;
                Ok(())
            }
            _ => Err(SyncError::InvalidConfiguration(
                "Gmail provider requires OAuth2 credentials".to_string(),
            )),
        }
    }

    async fn test_connection(&self) -> SyncResult<bool> {
        if self.access_token.is_none() {
            return Ok(false);
        }

        let token = self.access_token.as_ref().unwrap();
        let response = self
            .client
            .get(format!("{}/users/me/profile", GMAIL_API_BASE))
            .bearer_auth(token)
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    async fn fetch_folders(&self) -> SyncResult<Vec<SyncFolder>> {
        let token = self
            .access_token
            .as_ref()
            .ok_or_else(|| SyncError::AuthenticationError("Not authenticated".to_string()))?;

        let response = self
            .client
            .get(format!("{}/users/me/labels", GMAIL_API_BASE))
            .bearer_auth(token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(SyncError::GmailError(format!(
                "Failed to fetch labels: {}",
                response.status()
            )));
        }

        let labels: GmailLabelsResponse = response.json().await?;

        let folders = labels
            .labels
            .into_iter()
            .filter(|label| {
                !matches!(
                    label.id.as_str(),
                    "CATEGORY_PERSONAL"
                        | "CATEGORY_SOCIAL"
                        | "CATEGORY_UPDATES"
                        | "CATEGORY_FORUMS"
                        | "CATEGORY_PROMOTIONS"
                )
            })
            .map(|label| {
                let folder_type = Self::map_label_to_folder_type(&label.id, &label.name);

                SyncFolder {
                    id: None,
                    account_id: self.account_id,
                    name: label.name,
                    folder_type,
                    remote_id: label.id,
                    parent_id: None,
                    icon: None,
                    color: None,
                    sync_interval: 0,
                    synced_at: None,
                    attributes: Vec::new(),
                    unread_count: label.messages_unread.unwrap_or(0),
                    total_count: label.messages_total.unwrap_or(0),
                    expanded: false,
                    hidden: false,
                }
            })
            .collect();

        Ok(folders)
    }

    async fn sync_messages(
        &self,
        folder: &SyncFolder,
        _sync_token: Option<String>,
    ) -> SyncResult<crate::sync::types::SyncDiff> {
        let token = self
            .access_token
            .as_ref()
            .ok_or_else(|| SyncError::AuthenticationError("Not authenticated".to_string()))?;

        let max_results = 100;

        let response = self
            .client
            .get(format!("{}/users/me/messages", GMAIL_API_BASE))
            .bearer_auth(token)
            .query(&[
                ("labelIds", &folder.remote_id),
                ("maxResults", &max_results.to_string()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(SyncError::GmailError(format!(
                "Failed to fetch messages: {}",
                response.status()
            )));
        }

        let messages_response: GmailMessagesResponse = response.json().await?;

        let message_refs = messages_response.messages.unwrap_or_default();
        let mut emails = Vec::new();

        for msg_ref in message_refs {
            match self.fetch_email(folder, &msg_ref.id).await {
                Ok(email) => emails.push(email),
                Err(e) => log::error!("Failed to fetch email {}: {}", msg_ref.id, e),
            }
        }

        Ok(crate::sync::types::SyncDiff {
            added: emails,
            modified: Vec::new(),
            deleted: Vec::new(),
            next_sync_token: None,
        })
    }

    async fn fetch_email(&self, folder: &SyncFolder, remote_id: &str) -> SyncResult<SyncEmail> {
        let token = self
            .access_token
            .as_ref()
            .ok_or_else(|| SyncError::AuthenticationError("Not authenticated".to_string()))?;

        let response = self
            .client
            .get(format!(
                "{}/users/me/messages/{}",
                GMAIL_API_BASE, remote_id
            ))
            .bearer_auth(token)
            .query(&[("format", "full")])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(SyncError::GmailError(format!(
                "Failed to fetch message: {}",
                response.status()
            )));
        }

        let message: GmailMessage = response.json().await?;
        let folder_id = folder
            .id
            .ok_or_else(|| SyncError::DatabaseError("Folder ID is required".to_string()))?;
        Self::parse_gmail_message(&message, folder_id, self.account_id)
    }

    async fn fetch_attachment(&self, attachment: &SyncAttachment) -> SyncResult<Vec<u8>> {
        let token = self
            .access_token
            .as_ref()
            .ok_or_else(|| SyncError::AuthenticationError("Not authenticated".to_string()))?;

        let remote_path = attachment.remote_path.as_ref().ok_or_else(|| {
            SyncError::AttachmentError("No remote path for attachment".to_string())
        })?;

        let (message_id, attachment_id) = remote_path.rsplit_once(':').ok_or_else(|| {
            SyncError::AttachmentError(format!(
                "Invalid remote path format: {}. Expected 'message_id:attachment_id'",
                remote_path
            ))
        })?;

        let response = self
            .client
            .get(format!(
                "{}/users/me/messages/{}/attachments/{}",
                GMAIL_API_BASE, message_id, attachment_id
            ))
            .bearer_auth(token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(SyncError::GmailError(format!(
                "Failed to fetch attachment: {}",
                response.status()
            )));
        }

        #[derive(Deserialize)]
        struct AttachmentData {
            data: String,
        }

        let attachment_data: AttachmentData = response.json().await?;
        let decoded = general_purpose::URL_SAFE
            .decode(&attachment_data.data)
            .map_err(|e| SyncError::ParseError(format!("Base64 decode error: {}", e)))?;

        Ok(decoded)
    }

    async fn move_email(
        &self,
        email_remote_id: &str,
        from_folder: &SyncFolder,
        to_folder: &SyncFolder,
    ) -> SyncResult<()> {
        let token = self
            .access_token
            .as_ref()
            .ok_or_else(|| SyncError::AuthenticationError("Not authenticated".to_string()))?;

        #[derive(Serialize)]
        struct ModifyRequest {
            #[serde(rename = "addLabelIds")]
            add_label_ids: Vec<String>,
            #[serde(rename = "removeLabelIds")]
            remove_label_ids: Vec<String>,
        }

        let request = ModifyRequest {
            add_label_ids: vec![to_folder.remote_id.clone()],
            remove_label_ids: vec![from_folder.remote_id.clone()],
        };

        let response = self
            .client
            .post(format!(
                "{}/users/me/messages/{}/modify",
                GMAIL_API_BASE, email_remote_id
            ))
            .bearer_auth(token)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(SyncError::GmailError(format!(
                "Failed to move message: {}",
                response.status()
            )));
        }

        Ok(())
    }

    async fn delete_email(
        &self,
        email_remote_id: &str,
        _folder: &SyncFolder,
        permanent: bool,
    ) -> SyncResult<()> {
        let token = self
            .access_token
            .as_ref()
            .ok_or_else(|| SyncError::AuthenticationError("Not authenticated".to_string()))?;

        let endpoint = if permanent {
            format!("{}/users/me/messages/{}", GMAIL_API_BASE, email_remote_id)
        } else {
            format!(
                "{}/users/me/messages/{}/trash",
                GMAIL_API_BASE, email_remote_id
            )
        };

        let response = self
            .client
            .delete(&endpoint)
            .bearer_auth(token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(SyncError::GmailError(format!(
                "Failed to delete message: {}",
                response.status()
            )));
        }

        Ok(())
    }

    async fn mark_as_read(
        &self,
        email_remote_id: &str,
        _folder: &SyncFolder,
        is_read: bool,
    ) -> SyncResult<()> {
        let token = self
            .access_token
            .as_ref()
            .ok_or_else(|| SyncError::AuthenticationError("Not authenticated".to_string()))?;

        #[derive(Serialize)]
        struct ModifyRequest {
            #[serde(rename = "addLabelIds")]
            add_label_ids: Vec<String>,
            #[serde(rename = "removeLabelIds")]
            remove_label_ids: Vec<String>,
        }

        let request = if is_read {
            ModifyRequest {
                add_label_ids: Vec::new(),
                remove_label_ids: vec!["UNREAD".to_string()],
            }
        } else {
            ModifyRequest {
                add_label_ids: vec!["UNREAD".to_string()],
                remove_label_ids: Vec::new(),
            }
        };

        let response = self
            .client
            .post(format!(
                "{}/users/me/messages/{}/modify",
                GMAIL_API_BASE, email_remote_id
            ))
            .bearer_auth(token)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(SyncError::GmailError(format!(
                "Failed to modify message: {}",
                response.status()
            )));
        }

        Ok(())
    }

    async fn set_flag(
        &self,
        email_remote_id: &str,
        _folder: &SyncFolder,
        flagged: bool,
    ) -> SyncResult<()> {
        let token = self
            .access_token
            .as_ref()
            .ok_or_else(|| SyncError::AuthenticationError("Not authenticated".to_string()))?;

        #[derive(Serialize)]
        struct ModifyRequest {
            #[serde(rename = "addLabelIds")]
            add_label_ids: Vec<String>,
            #[serde(rename = "removeLabelIds")]
            remove_label_ids: Vec<String>,
        }

        let request = if flagged {
            ModifyRequest {
                add_label_ids: vec!["STARRED".to_string()],
                remove_label_ids: Vec::new(),
            }
        } else {
            ModifyRequest {
                add_label_ids: Vec::new(),
                remove_label_ids: vec!["STARRED".to_string()],
            }
        };

        let response = self
            .client
            .post(format!(
                "{}/users/me/messages/{}/modify",
                GMAIL_API_BASE, email_remote_id
            ))
            .bearer_auth(token)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(SyncError::GmailError(format!(
                "Failed to modify message: {}",
                response.status()
            )));
        }

        Ok(())
    }

    async fn rename_folder(&self, folder: &SyncFolder, new_name: &str) -> SyncResult<()> {
        let token = self
            .access_token
            .as_ref()
            .ok_or_else(|| SyncError::AuthenticationError("Not authenticated".to_string()))?;

        // Gmail uses labels, not folders
        // Update the label name via PATCH request
        #[derive(Serialize)]
        struct UpdateLabelRequest {
            name: String,
        }

        let request = UpdateLabelRequest {
            name: new_name.to_string(),
        };

        let response = self
            .client
            .patch(format!(
                "{}/users/me/labels/{}",
                GMAIL_API_BASE, folder.remote_id
            ))
            .bearer_auth(token)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(SyncError::GmailError(format!(
                "Failed to rename label: {}",
                response.status()
            )));
        }

        log::info!("Renamed Gmail label '{}' to '{}'", folder.name, new_name);
        Ok(())
    }

    async fn move_folder(
        &self,
        _folder: &SyncFolder,
        _new_parent_path: Option<&str>,
    ) -> SyncResult<()> {
        Err(SyncError::NotSupported(
            "Gmail does not support moving labels (use rename to change hierarchy)".to_string(),
        ))
    }

    async fn get_sync_token(&self) -> SyncResult<Option<String>> {
        let token = self
            .access_token
            .as_ref()
            .ok_or_else(|| SyncError::AuthenticationError("Not authenticated".to_string()))?;

        let response = self
            .client
            .get(format!("{}/users/me/profile", GMAIL_API_BASE))
            .bearer_auth(token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(SyncError::GmailError(format!(
                "Failed to get profile: {}",
                response.status()
            )));
        }

        #[derive(Deserialize)]
        struct Profile {
            #[serde(rename = "historyId")]
            history_id: String,
        }

        let profile: Profile = response.json().await?;
        Ok(Some(profile.history_id))
    }

    async fn sync_since_token(&self, _token: &str) -> SyncResult<Vec<SyncEmail>> {
        let _access_token = self
            .access_token
            .as_ref()
            .ok_or_else(|| SyncError::AuthenticationError("Not authenticated".to_string()))?;

        // TODO: Implement Gmail history sync
        log::warn!("Gmail history sync not yet fully implemented");
        Ok(Vec::new())
    }
}
