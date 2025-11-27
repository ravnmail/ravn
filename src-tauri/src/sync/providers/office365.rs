use crate::database::models::email::EmailAddress;
use crate::sync::{
    auth::{CredentialStore, OAuth2Helper},
    error::{SyncError, SyncResult},
    provider::EmailProvider,
    types::*,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

const GRAPH_API_BASE: &str = "https://graph.microsoft.com/v1.0";

pub struct Office365Provider {
    account_id: Uuid,
    client: Client,
    access_token: Arc<RwLock<Option<String>>>,
    credential_store: Arc<CredentialStore>,
    app_handle: Option<tauri::AppHandle>,
}

#[derive(Debug, Deserialize)]
struct GraphFoldersResponse {
    value: Vec<GraphFolder>,
    #[serde(rename = "@odata.nextLink")]
    next_link: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GraphFolder {
    id: String,
    #[serde(rename = "displayName")]
    display_name: String,
    #[serde(rename = "parentFolderId")]
    parent_folder_id: Option<String>,
    #[serde(rename = "childFolderCount")]
    child_folder_count: Option<i32>,
    #[serde(rename = "unreadItemCount")]
    unread_item_count: Option<i32>,
    #[serde(rename = "totalItemCount")]
    total_item_count: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct GraphMessagesResponse {
    value: Vec<GraphMessage>,
    #[serde(rename = "@odata.nextLink")]
    next_link: Option<String>,
    #[serde(rename = "@odata.deltaLink")]
    delta_link: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GraphDeltaResponse {
    value: Vec<GraphMessage>,
    #[serde(rename = "@odata.nextLink")]
    next_link: Option<String>,
    #[serde(rename = "@odata.deltaLink")]
    delta_link: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GraphMessage {
    id: String,
    #[serde(rename = "conversationId")]
    conversation_id: Option<String>,
    #[serde(rename = "internetMessageId")]
    internet_message_id: Option<String>,
    #[serde(rename = "changeKey")]
    change_key: Option<String>,
    #[serde(rename = "lastModifiedDateTime")]
    last_modified_date_time: Option<String>,
    subject: Option<String>,
    #[serde(rename = "bodyPreview")]
    body_preview: Option<String>,
    body: Option<GraphBody>,
    from: Option<GraphRecipient>,
    #[serde(rename = "toRecipients")]
    to_recipients: Option<Vec<GraphRecipient>>,
    #[serde(rename = "ccRecipients")]
    cc_recipients: Option<Vec<GraphRecipient>>,
    #[serde(rename = "bccRecipients")]
    bcc_recipients: Option<Vec<GraphRecipient>>,
    #[serde(rename = "replyTo")]
    reply_to: Option<Vec<GraphRecipient>>,
    #[serde(rename = "receivedDateTime")]
    received_date_time: Option<String>,
    #[serde(rename = "sentDateTime")]
    sent_date_time: Option<String>,
    #[serde(rename = "isRead")]
    is_read: Option<bool>,
    #[serde(rename = "isDraft")]
    is_draft: Option<bool>,
    #[serde(rename = "hasAttachments")]
    has_attachments: Option<bool>,
    flag: Option<GraphFlag>,
    #[serde(rename = "@removed")]
    removed: Option<GraphRemoved>,
}

#[derive(Debug, Deserialize)]
struct GraphRemoved {
    reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GraphBody {
    #[serde(rename = "contentType")]
    content_type: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct GraphRecipient {
    #[serde(rename = "emailAddress")]
    email_address: GraphEmailAddress,
}

#[derive(Debug, Deserialize)]
struct GraphEmailAddress {
    name: Option<String>,
    address: String,
}

#[derive(Debug, Deserialize)]
struct GraphFlag {
    #[serde(rename = "flagStatus")]
    flag_status: String,
}

#[derive(Debug, Deserialize)]
struct GraphAttachmentsResponse {
    value: Vec<GraphAttachment>,
}

#[derive(Debug, Deserialize)]
struct GraphAttachment {
    #[serde(rename = "@odata.type")]
    odata_type: Option<String>,
    id: String,
    name: String,
    #[serde(rename = "contentType")]
    content_type: String,
    size: i64,
    #[serde(rename = "isInline")]
    is_inline: Option<bool>,
    #[serde(rename = "contentId")]
    content_id: Option<String>,
}

impl GraphAttachment {
    fn is_file_attachment(&self) -> bool {
        self.odata_type
            .as_ref()
            .map(|t| t.contains("fileAttachment"))
            .unwrap_or(true)
    }

    fn to_sync_attachment(&self, message_id: &str) -> SyncAttachment {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(format!("{}:{}", message_id, self.id).as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        SyncAttachment {
            id: None,
            email_id: None,
            filename: self.name.clone(),
            content_type: self.content_type.clone(),
            size: self.size,
            hash,
            cache_path: None,
            remote_url: None,
            remote_path: Some(format!("{}:{}", message_id, self.id)),
            is_inline: self.is_inline.unwrap_or(false),
            is_cached: false,
            content_id: self.content_id.clone(),
            data: None,
        }
    }
}

impl Office365Provider {
    pub fn new(account_id: Uuid, credential_store: Arc<CredentialStore>) -> SyncResult<Self> {
        Ok(Self {
            account_id,
            client: Client::new(),
            access_token: Arc::new(RwLock::new(None)),
            credential_store,
            app_handle: None,
        })
    }

    pub fn with_app_handle(mut self, app_handle: tauri::AppHandle) -> Self {
        self.app_handle = Some(app_handle);
        self
    }

    async fn handle_401_error(&self) -> SyncResult<()> {
        use tauri::Emitter;

        log::warn!(
            "[Office365] Received 401 error, attempting to refresh token for account {}",
            self.account_id
        );

        {
            let mut token = self.access_token.write().await;
            *token = None;
        }

        let credentials = self.credential_store.get_oauth2(self.account_id).await?;

        if let Some(refresh_token) = &credentials.refresh_token {
            match OAuth2Helper::refresh_token("office365", refresh_token).await {
                Ok(new_credentials) => {
                    log::info!(
                        "[Office365] Successfully refreshed token for account {}",
                        self.account_id
                    );

                    self.credential_store
                        .store_oauth2(self.account_id, &new_credentials)
                        .await?;

                    {
                        let mut token = self.access_token.write().await;
                        *token = Some(new_credentials.access_token.clone());
                    }

                    Ok(())
                }
                Err(e) => {
                    log::error!(
                        "[Office365] Failed to refresh token for account {}: {}",
                        self.account_id,
                        e
                    );

                    if let Some(app_handle) = &self.app_handle {
                        let _ = app_handle.emit(
                            "office365:auth-required",
                            serde_json::json!({
                                "account_id": self.account_id.to_string(),
                                "provider": "office365",
                                "reason": "Token refresh failed"
                            }),
                        );
                    }

                    Err(SyncError::AuthenticationError(format!(
                        "Token refresh failed: {}. Please re-authenticate.",
                        e
                    )))
                }
            }
        } else {
            log::error!(
                "[Office365] No refresh token available for account {}",
                self.account_id
            );

            if let Some(app_handle) = &self.app_handle {
                let _ = app_handle.emit(
                    "office365:auth-required",
                    serde_json::json!({
                        "account_id": self.account_id.to_string(),
                        "provider": "office365",
                        "reason": "No refresh token available"
                    }),
                );
            }

            Err(SyncError::AuthenticationError(
                "No refresh token available. Please re-authenticate.".to_string(),
            ))
        }
    }

    async fn ensure_token(&self) -> SyncResult<String> {
        {
            let token = self.access_token.read().await;
            if let Some(ref t) = *token {
                return Ok(t.clone());
            }
        }

        let mut credentials = self.credential_store.get_oauth2(self.account_id).await?;

        if let Some(expires_at) = credentials.expires_at {
            if expires_at < Utc::now() {
                if let Some(refresh_token) = &credentials.refresh_token {
                    credentials = OAuth2Helper::refresh_token("office365", refresh_token).await?;
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

        {
            let mut token = self.access_token.write().await;
            *token = Some(credentials.access_token.clone());
        }

        Ok(credentials.access_token)
    }

    async fn execute_with_401_retry<F, Fut>(&self, operation: F) -> SyncResult<reqwest::Response>
    where
        F: Fn(String) -> Fut,
        Fut: std::future::Future<Output = Result<reqwest::Response, reqwest::Error>>,
    {
        let token = self.ensure_token().await?;
        let response = operation(token)
            .await
            .map_err(|e| SyncError::NetworkError(e.to_string()))?;

        if response.status().as_u16() == 401 {
            log::warn!("[Office365] Got 401 Unauthorized, attempting token refresh");

            self.handle_401_error().await?;

            let new_token = self.ensure_token().await?;
            let retry_response = operation(new_token)
                .await
                .map_err(|e| SyncError::NetworkError(e.to_string()))?;

            Ok(retry_response)
        } else {
            Ok(response)
        }
    }

    fn map_folder_type(display_name: &str) -> FolderType {
        let name_lower = display_name.to_lowercase();
        if name_lower.contains("inbox") {
            FolderType::Inbox
        } else if name_lower.contains("sent") {
            FolderType::Sent
        } else if name_lower.contains("draft") {
            FolderType::Draft
        } else if name_lower.contains("deleted") || name_lower.contains("trash") {
            FolderType::Trash
        } else if name_lower.contains("junk") || name_lower.contains("spam") {
            FolderType::Spam
        } else if name_lower.contains("archive") {
            FolderType::Archive
        } else {
            FolderType::Custom
        }
    }

    fn convert_recipient(recipient: &GraphRecipient) -> EmailAddress {
        EmailAddress {
            name: recipient.email_address.name.clone(),
            address: recipient.email_address.address.clone(),
        }
    }

    async fn _fetch_attachments_metadata(
        &self,
        message_id: &str,
        existing_hashes: &std::collections::HashSet<String>,
    ) -> SyncResult<Vec<SyncAttachment>> {
        let token = self.ensure_token().await?;

        let response = self
            .client
            .get(format!(
                "{}/me/messages/{}/attachments",
                GRAPH_API_BASE, message_id
            ))
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| {
                SyncError::NetworkError(format!(
                    "Network error fetching attachments for message {}: {}",
                    message_id, e
                ))
            })?;

        if !response.status().is_success() {
            if response.status().as_u16() == 404 {
                log::warn!("Message {} not found when fetching attachments", message_id);
                return Ok(Vec::new());
            }

            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error".to_string());

            return Err(SyncError::Office365Error(format!(
                "Failed to fetch attachments for message {} (status {}): {}",
                message_id, status, error_text
            )));
        }

        let attachments_response: GraphAttachmentsResponse =
            response.json().await.map_err(|e| {
                SyncError::Office365Error(format!(
                    "Failed to parse attachments response for message {}: {}",
                    message_id, e
                ))
            })?;

        let sync_attachments: Vec<SyncAttachment> = attachments_response
            .value
            .into_iter()
            .filter(|att| {
                if !att.is_file_attachment() {
                    log::debug!(
                        "Skipping non-file attachment {} (type: {:?})",
                        att.name,
                        att.odata_type
                    );
                    return false;
                }
                true
            })
            .map(|att| {
                let sync_att = att.to_sync_attachment(message_id);
                let is_cached = existing_hashes.contains(&sync_att.hash);

                SyncAttachment {
                    is_cached,
                    ..sync_att
                }
            })
            .collect();

        Ok(sync_attachments)
    }

    pub async fn fetch_attachments_for_message(
        &self,
        message_id: &str,
    ) -> SyncResult<Vec<SyncAttachment>> {
        let token = self.ensure_token().await?;

        let response = self
            .client
            .get(format!(
                "{}/me/messages/{}/attachments",
                GRAPH_API_BASE, message_id
            ))
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| {
                SyncError::NetworkError(format!(
                    "Network error fetching attachments for message {}: {}",
                    message_id, e
                ))
            })?;

        if !response.status().is_success() {
            if response.status().as_u16() == 404 {
                log::debug!("Message {} not found when fetching attachments", message_id);
                return Ok(Vec::new());
            }

            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error".to_string());

            return Err(SyncError::Office365Error(format!(
                "Failed to fetch attachments for message {} (status {}): {}",
                message_id, status, error_text
            )));
        }

        let attachments_response: GraphAttachmentsResponse =
            response.json().await.map_err(|e| {
                SyncError::Office365Error(format!(
                    "Failed to parse attachments response for message {}: {}",
                    message_id, e
                ))
            })?;

        let sync_attachments: Vec<SyncAttachment> = attachments_response
            .value
            .into_iter()
            .filter(|att| att.is_file_attachment())
            .map(|att| att.to_sync_attachment(message_id))
            .collect();

        Ok(sync_attachments)
    }

    /// Enrich emails with attachment metadata and data
    /// - Downloads all attachments (inline + regular)
    /// - Inline attachments needed for cid: links in HTML
    /// - Regular attachments downloaded for complete email sync
    pub async fn enrich_emails_with_attachments(
        &self,
        emails: &mut [SyncEmail],
        download_all: bool,
    ) -> SyncResult<()> {
        // Process attachments for emails that have them
        for email in emails.iter_mut() {
            if email.has_attachments {
                match self.fetch_attachments_for_message(&email.remote_id).await {
                    Ok(mut attachments) => {
                        if !attachments.is_empty() {
                            log::debug!(
                                "[Office365] Fetched {} attachment metadata for email {}",
                                attachments.len(),
                                email.remote_id
                            );

                            // Download all attachments if enabled (sequential with logging)
                            if download_all {
                                for attachment in &mut attachments {
                                    match self.fetch_attachment(attachment).await {
                                        Ok(data) => {
                                            log::debug!(
                                                "[Office365] Downloaded attachment {} ({} bytes)",
                                                attachment.filename,
                                                data.len()
                                            );
                                            attachment.data = Some(data);
                                        }
                                        Err(e) => {
                                            log::warn!(
                                                "[Office365] Failed to download attachment {}: {}",
                                                attachment.filename,
                                                e
                                            );
                                            // Continue - email can still be synced without this attachment
                                        }
                                    }
                                }
                            }

                            email.attachments = attachments;
                        }
                    }
                    Err(e) => {
                        log::warn!(
                            "[Office365] Failed to fetch attachments for email {}: {}",
                            email.remote_id,
                            e
                        );
                        // Don't fail the entire sync if attachment fetching fails
                        // The email can still be synced without attachments
                    }
                }
            }
        }
        Ok(())
    }

    fn parse_graph_message(
        msg: &GraphMessage,
        folder_id: Uuid,
        account_id: Uuid,
        should_include_body: bool,
    ) -> SyncResult<SyncEmail> {
        let from = msg
            .from
            .as_ref()
            .map(Self::convert_recipient)
            .unwrap_or_else(|| EmailAddress {
                name: None,
                address: "unknown@unknown.com".to_string(),
            });

        let to = msg
            .to_recipients
            .as_ref()
            .map(|recipients| recipients.iter().map(Self::convert_recipient).collect())
            .unwrap_or_default();

        let cc = msg
            .cc_recipients
            .as_ref()
            .map(|recipients| recipients.iter().map(Self::convert_recipient).collect())
            .unwrap_or_default();

        let bcc = msg
            .bcc_recipients
            .as_ref()
            .map(|recipients| recipients.iter().map(Self::convert_recipient).collect())
            .unwrap_or_default();

        let reply_to = msg
            .reply_to
            .as_ref()
            .and_then(|recipients| recipients.first())
            .map(Self::convert_recipient);

        let (body_html, body_plain) = if should_include_body {
            match &msg.body {
                Some(body) if body.content_type == "html" => (Some(body.content.clone()), None),
                Some(body) => (None, Some(body.content.clone())),
                None => (None, None),
            }
        } else {
            (None, None)
        };

        let received_at = msg
            .received_date_time
            .as_ref()
            .and_then(|dt| DateTime::parse_from_rfc3339(dt).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|| Utc::now());

        let sent_at = msg
            .sent_date_time
            .as_ref()
            .and_then(|dt| DateTime::parse_from_rfc3339(dt).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let last_modified_at = msg
            .last_modified_date_time
            .as_ref()
            .and_then(|dt| DateTime::parse_from_rfc3339(dt).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let mut flags = Vec::new();
        if msg.is_read.unwrap_or(false) {
            flags.push("\\Seen".to_string());
        }
        if msg.is_draft.unwrap_or(false) {
            flags.push("\\Draft".to_string());
        }
        if let Some(flag) = &msg.flag {
            if flag.flag_status == "flagged" {
                flags.push("\\Flagged".to_string());
            }
        }

        let message_id = msg
            .internet_message_id
            .clone()
            .unwrap_or_else(|| msg.id.clone());

        let size = body_html
            .as_ref()
            .or(body_plain.as_ref())
            .map(|b| b.len() as i64)
            .unwrap_or(0);

        Ok(SyncEmail {
            id: None,
            account_id,
            folder_id,
            message_id,
            conversation_id: msg.conversation_id.clone(),
            remote_id: msg.id.clone(),
            from,
            to,
            cc,
            bcc,
            reply_to,
            subject: msg.subject.clone(),
            snippet: msg.body_preview.clone(),
            body_plain,
            body_html,
            other_mails: None,
            category: None,
            ai_cache: None,
            received_at,
            sent_at,
            flags,
            headers: None,
            size,
            has_attachments: msg.has_attachments.unwrap_or(false),
            attachments: Vec::new(),
            change_key: msg.change_key.clone(),
            last_modified_at,
        })
    }

    async fn _retry_with_backoff<F, Fut, T>(&self, operation: F, max_retries: u32) -> SyncResult<T>
    where
        F: Fn() -> Fut + Send,
        Fut: std::future::Future<Output = SyncResult<T>> + Send,
        T: Send,
    {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    let is_retryable = match &e {
                        SyncError::Office365Error(msg) => {
                            msg.contains("429") || msg.contains("503") || msg.contains("504")
                        }
                        SyncError::NetworkError(_) => true,
                        _ => false,
                    };

                    if !is_retryable || attempt >= max_retries {
                        return Err(e);
                    }

                    last_error = Some(e);

                    let delay = std::time::Duration::from_secs(2u64.pow(attempt));
                    log::warn!(
                        "Retrying after {} seconds (attempt {}/{})",
                        delay.as_secs(),
                        attempt + 1,
                        max_retries
                    );
                    tokio::time::sleep(delay).await;
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| SyncError::Office365Error("Max retries exceeded".to_string())))
    }

    async fn fetch_emails_delta(
        &self,
        folder: &SyncFolder,
        delta_link: &str,
    ) -> SyncResult<(Vec<SyncEmail>, Option<String>)> {
        log::info!(
            "[Office365] Starting delta sync for folder {} using delta link",
            folder.name
        );

        let folder_id = folder
            .id
            .ok_or_else(|| SyncError::DatabaseError("Folder ID is required".to_string()))?;

        let mut all_emails = Vec::new();
        let mut current_link = delta_link.to_string();
        let mut page_count = 0;
        let mut final_delta_link: Option<String> = None;
        const MAX_DELTA_PAGES: usize = 1000;

        loop {
            page_count += 1;

            // Prevent infinite loops from malformed pagination
            if page_count > MAX_DELTA_PAGES {
                log::warn!(
                    "[Office365] Delta sync for folder {} exceeded max pages ({}), stopping",
                    folder.name,
                    MAX_DELTA_PAGES
                );
                break;
            }

            log::debug!(
                "[Office365] Fetching delta page {} for folder {}",
                page_count,
                folder.name
            );

            let url_clone = current_link.clone();
            let response = self
                .execute_with_401_retry(|token| {
                    let client = self.client.clone();
                    let url = url_clone.clone();
                    async move { client.get(url).bearer_auth(token).send().await }
                })
                .await?;

            if !response.status().is_success() {
                return Err(SyncError::Office365Error(format!(
                    "Failed to fetch delta messages: {}",
                    response.status()
                )));
            }

            let delta_response: GraphDeltaResponse = response.json().await?;
            let messages_count = delta_response.value.len();
            let removed_count = delta_response
                .value
                .iter()
                .filter(|m| m.removed.is_some())
                .count();

            for msg in delta_response.value.iter() {
                if msg.removed.is_some() {
                    continue;
                }

                match Self::parse_graph_message(msg, folder_id, self.account_id, true) {
                    Ok(email) => {
                        all_emails.push(email);
                    }
                    Err(e) => {
                        log::error!("Failed to parse delta message: {}", e);
                    }
                }
            }

            log::info!(
                "[Office365] Fetched delta page {}: {} messages, {} removed ({} total so far)",
                page_count,
                messages_count - removed_count,
                removed_count,
                all_emails.len()
            );

            if let Some(delta_link) = delta_response.delta_link {
                final_delta_link = Some(delta_link);
                log::info!(
                    "[Office365] Delta sync complete for folder {}: {} emails from {} pages",
                    folder.name,
                    all_emails.len(),
                    page_count
                );
                break;
            } else if let Some(next_link) = delta_response.next_link {
                current_link = next_link;
            } else {
                log::warn!("[Office365] No deltaLink or nextLink in response, stopping pagination");
                break;
            }
        }

        Ok((all_emails, final_delta_link))
    }

    pub async fn fetch_emails_full(
        &self,
        folder: &SyncFolder,
    ) -> SyncResult<(Vec<SyncEmail>, Option<String>)> {
        let page_size = 100;
        let folder_remote_id = folder.remote_id.clone();
        let folder_id = folder
            .id
            .ok_or_else(|| SyncError::DatabaseError("Folder ID is required".to_string()))?;

        let mut all_emails = Vec::new();
        let mut next_link: Option<String> = None;
        let mut delta_link: Option<String> = None;
        let mut page_count = 0;
        const MAX_FULL_SYNC_PAGES: usize = 1000;

        loop {
            page_count += 1;

            // Prevent infinite loops from malformed pagination
            if page_count > MAX_FULL_SYNC_PAGES {
                log::warn!(
                    "[Office365] Full sync for folder {} exceeded max pages ({}), stopping",
                    folder.name,
                    MAX_FULL_SYNC_PAGES
                );
                break;
            }

            let url = if let Some(ref link) = next_link {
                link.clone()
            } else {
                // Use regular /messages endpoint for full sync (supports proper pagination)
                format!(
                    "{}/me/mailFolders/{}/messages",
                    GRAPH_API_BASE, folder_remote_id
                )
            };

            log::debug!(
                "[Office365] Fetching page {} for folder {} (full sync)",
                page_count,
                folder.name
            );

            let response = if next_link.is_some() {
                // nextLink already includes all query parameters
                let url_clone = url.clone();
                self.execute_with_401_retry(|token| {
                    let client = self.client.clone();
                    let url = url_clone.clone();
                    async move { client.get(url).bearer_auth(token).send().await }
                })
                .await?
            } else {
                // Initial request with pagination parameters
                let remote_id = folder_remote_id.clone();
                self.execute_with_401_retry(|token| {
                    let client = self.client.clone();
                    let remote_id = remote_id.clone();
                    async move {
                        client
                            .get(format!(
                                "{}/me/mailFolders/{}/messages",
                                GRAPH_API_BASE, remote_id
                            ))
                            .bearer_auth(token)
                            .query(&[("$top", page_size.to_string())])
                            .send()
                            .await
                    }
                })
                .await?
            };

            if !response.status().is_success() {
                return Err(SyncError::Office365Error(format!(
                    "Failed to fetch messages: {}",
                    response.status()
                )));
            }

            let messages_response: GraphMessagesResponse = response.json().await?;
            let messages_count = messages_response.value.len();

            for msg in messages_response.value.iter() {
                match Self::parse_graph_message(msg, folder_id, self.account_id, true) {
                    Ok(email) => {
                        all_emails.push(email);
                    }
                    Err(e) => {
                        log::error!("Failed to parse message: {}", e);
                    }
                }
            }

            log::info!(
                "[Office365] Fetched page {}: {} messages ({} total so far)",
                page_count,
                messages_count,
                all_emails.len()
            );

            next_link = messages_response.next_link;

            // Capture delta link from the response (returned on last page)
            if messages_response.delta_link.is_some() {
                delta_link = messages_response.delta_link;
            }

            if next_link.is_none() {
                log::info!(
                    "[Office365] Completed full fetch for folder {}: {} emails in {} pages (has delta link: {})",
                    folder.name,
                    all_emails.len(),
                    page_count,
                    delta_link.is_some()
                );
                break;
            }
        }

        Ok((all_emails, delta_link))
    }

    /// Fetch and process delta emails page by page
    /// Processes each page immediately as it's fetched (not buffering all pages)
    /// Returns (delta_link, deleted_remote_ids)
    pub async fn fetch_emails_delta_paged<F, Fut>(
        &self,
        folder: &SyncFolder,
        delta_link: &str,
        mut on_page: F,
    ) -> SyncResult<(Option<String>, Vec<String>)>
    where
        F: FnMut(Vec<SyncEmail>) -> Fut + Send,
        Fut: std::future::Future<Output = SyncResult<()>> + Send,
    {
        log::info!(
            "[Office365] Starting delta sync for folder {} (paged processing)",
            folder.name
        );

        let folder_id = folder
            .id
            .ok_or_else(|| SyncError::DatabaseError("Folder ID is required".to_string()))?;

        let mut current_link = delta_link.to_string();
        let mut page_count = 0;
        let mut final_delta_link: Option<String> = None;
        let mut deleted_email_ids: Vec<String> = Vec::new();
        const MAX_DELTA_PAGES: usize = 1000;
        let mut total_emails = 0;

        loop {
            page_count += 1;

            if page_count > MAX_DELTA_PAGES {
                log::warn!(
                    "[Office365] Delta sync exceeded max pages ({}), stopping",
                    MAX_DELTA_PAGES
                );
                break;
            }

            let _token = self.ensure_token().await?;

            let response = self
                .execute_with_401_retry(|token| {
                    let client = self.client.clone();
                    let url = current_link.clone();
                    async move { client.get(url).bearer_auth(token).send().await }
                })
                .await?;

            if !response.status().is_success() {
                return Err(SyncError::Office365Error(format!(
                    "Failed to fetch delta messages: {}",
                    response.status()
                )));
            }

            let delta_response: GraphDeltaResponse = response.json().await?;
            let messages_count = delta_response.value.len();
            let removed_count = delta_response
                .value
                .iter()
                .filter(|m| m.removed.is_some())
                .count();

            // Parse emails from this page and collect deleted email IDs
            let mut page_emails = Vec::new();
            for msg in delta_response.value.iter() {
                if let Some(removed) = &msg.removed {
                    // Only treat as deleted if reason is "deleted" or "moved"
                    // "changed" means property update (like marking as read), not actual deletion
                    let reason = removed.reason.as_deref().unwrap_or("");
                    if reason == "deleted" || reason == "moved" {
                        deleted_email_ids.push(msg.id.clone());
                    }
                    // For "changed" items, they should be processed as updates (below)
                    // For "deleted" and "moved", skip processing
                    if reason != "changed" {
                        continue;
                    }
                }

                match Self::parse_graph_message(msg, folder_id, self.account_id, true) {
                    Ok(email) => page_emails.push(email),
                    Err(e) => log::error!("Failed to parse delta message: {}", e),
                }
            }

            // Enrich this page with attachments immediately
            self.enrich_emails_with_attachments(&mut page_emails, true)
                .await
                .ok();

            total_emails += page_emails.len();
            log::info!(
                "[Office365] Fetched delta page {}: {} messages, {} removed ({} total so far)",
                page_count,
                messages_count - removed_count,
                removed_count,
                total_emails
            );

            // Process this page immediately (not buffering)
            on_page(page_emails).await?;

            // Check for next page or delta link
            if let Some(delta_link) = delta_response.delta_link {
                final_delta_link = Some(delta_link);
                log::info!(
                    "[Office365] Delta sync complete for folder {}: {} emails in {} pages",
                    folder.name,
                    total_emails,
                    page_count
                );
                break;
            } else if let Some(next_link) = delta_response.next_link {
                current_link = next_link;
            } else {
                log::warn!("[Office365] No deltaLink or nextLink in response, stopping pagination");
                break;
            }
        }

        Ok((final_delta_link, deleted_email_ids))
    }

    /// Fetch and process full emails page by page
    /// Processes each page immediately as it's fetched (not buffering all pages)
    pub async fn fetch_emails_full_paged<F, Fut>(
        &self,
        folder: &SyncFolder,
        mut on_page: F,
    ) -> SyncResult<Option<String>>
    where
        F: FnMut(Vec<SyncEmail>) -> Fut + Send,
        Fut: std::future::Future<Output = SyncResult<()>> + Send,
    {
        log::info!(
            "[Office365] Starting full sync for folder {} (paged processing)",
            folder.name
        );

        let folder_id = folder
            .id
            .ok_or_else(|| SyncError::DatabaseError("Folder ID is required".to_string()))?;

        let page_size = 100;
        let folder_remote_id = folder.remote_id.clone();
        let mut next_link: Option<String> = None;
        let mut delta_link: Option<String> = None;
        let mut page_count = 0;
        const MAX_FULL_SYNC_PAGES: usize = 1000;
        let mut total_emails = 0;

        loop {
            page_count += 1;

            if page_count > MAX_FULL_SYNC_PAGES {
                log::warn!(
                    "[Office365] Full sync exceeded max pages ({}), stopping",
                    MAX_FULL_SYNC_PAGES
                );
                break;
            }

            let url = if let Some(ref link) = next_link {
                link.clone()
            } else {
                format!(
                    "{}/me/mailFolders/{}/messages",
                    GRAPH_API_BASE, folder_remote_id
                )
            };

            let response = if next_link.is_some() {
                let url_clone = url.clone();
                self.execute_with_401_retry(|token| {
                    let client = self.client.clone();
                    let url = url_clone.clone();
                    async move { client.get(url).bearer_auth(token).send().await }
                })
                .await?
            } else {
                let remote_id = folder_remote_id.clone();
                self.execute_with_401_retry(|token| {
                    let client = self.client.clone();
                    let remote_id = remote_id.clone();
                    async move {
                        client
                            .get(format!(
                                "{}/me/mailFolders/{}/messages/delta",
                                GRAPH_API_BASE, remote_id
                            ))
                            .bearer_auth(token)
                            // .query(&[
                            //     ("$top", page_size.to_string()),
                            // ])
                            .send()
                            .await
                    }
                })
                .await?
            };

            if !response.status().is_success() {
                return Err(SyncError::Office365Error(format!(
                    "Failed to fetch messages: {}",
                    response.status()
                )));
            }

            let messages_response: GraphMessagesResponse = response.json().await?;
            let messages_count = messages_response.value.len();

            // Parse emails from this page
            let mut page_emails = Vec::new();
            for msg in messages_response.value.iter() {
                match Self::parse_graph_message(msg, folder_id, self.account_id, true) {
                    Ok(email) => page_emails.push(email),
                    Err(e) => log::error!("Failed to parse message: {}", e),
                }
            }

            // Enrich this page with attachments immediately
            self.enrich_emails_with_attachments(&mut page_emails, true)
                .await
                .ok();

            total_emails += page_emails.len();
            log::info!(
                "[Office365] Fetched full page {}: {} messages ({} total so far)",
                page_count,
                messages_count,
                total_emails
            );

            // Process this page immediately (not buffering)
            on_page(page_emails).await?;

            next_link = messages_response.next_link;

            // Capture delta link from the response
            if messages_response.delta_link.is_some() {
                delta_link = messages_response.delta_link;
            }

            if next_link.is_none() {
                log::info!(
                    "[Office365] Completed full fetch for folder {}: {} emails in {} pages (has delta link: {})",
                    folder.name,
                    total_emails,
                    page_count,
                    delta_link.is_some()
                );
                break;
            }
        }

        Ok(delta_link)
    }
}

#[async_trait]
impl EmailProvider for Office365Provider {
    fn name(&self) -> &str {
        "Office365"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn authenticate(&mut self, credentials: ProviderCredentials) -> SyncResult<()> {
        match credentials {
            ProviderCredentials::OAuth2(creds) => {
                {
                    let mut token = self.access_token.write().await;
                    *token = Some(creds.access_token.clone());
                }
                self.credential_store
                    .store_oauth2(self.account_id, &creds)
                    .await?;
                Ok(())
            }
            _ => Err(SyncError::InvalidConfiguration(
                "Office365 provider requires OAuth2 credentials".to_string(),
            )),
        }
    }

    async fn test_connection(&self) -> SyncResult<bool> {
        let token = {
            let token_guard = self.access_token.read().await;
            if token_guard.is_none() {
                return Ok(false);
            }
            token_guard.as_ref().unwrap().clone()
        };

        let response = self
            .client
            .get(format!("{}/me", GRAPH_API_BASE))
            .bearer_auth(&token)
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    async fn fetch_folders(&self) -> SyncResult<Vec<SyncFolder>> {
        let response = self
            .execute_with_401_retry(|token| {
                let client = self.client.clone();
                async move {
                    client
                        .get(format!("{}/me/mailFolders", GRAPH_API_BASE))
                        .bearer_auth(token)
                        .send()
                        .await
                }
            })
            .await?;

        if !response.status().is_success() {
            return Err(SyncError::Office365Error(format!(
                "Failed to fetch folders: {}",
                response.status()
            )));
        }

        let folders_response: GraphFoldersResponse = response.json().await?;

        let mut all_folders = Vec::new();

        for folder in folders_response.value {
            let folder_type = Self::map_folder_type(&folder.display_name);
            let has_children = folder.child_folder_count.unwrap_or(0) > 0;
            let folder_remote_id = folder.id.clone();

            let mut attributes = Vec::new();
            if let Some(ref parent_id) = folder.parent_folder_id {
                attributes.push(format!("parent_remote_id:{}", parent_id));
            }

            let sync_folder = SyncFolder {
                id: None,
                account_id: self.account_id,
                name: folder.display_name,
                folder_type,
                remote_id: folder_remote_id.clone(),
                icon: None,
                color: None,
                parent_id: None,
                sync_interval: 0,
                synced_at: None,
                attributes,
                unread_count: folder.unread_item_count.unwrap_or(0),
                total_count: folder.total_item_count.unwrap_or(0),
                expanded: false,
                hidden: false,
            };

            all_folders.push(sync_folder);

            if has_children {
                fetch_child_folders_recursive(self, folder_remote_id, &mut all_folders, 0).await?;
            }
        }

        Ok(all_folders)
    }

    async fn sync_messages(
        &self,
        folder: &SyncFolder,
        sync_token: Option<String>,
    ) -> SyncResult<crate::sync::types::SyncDiff> {
        if let Some(token) = sync_token {
            // Delta sync: fetch only changes
            let (emails, next_token) = self.fetch_emails_delta(folder, &token).await?;

            // Classify delta messages as added, modified, or deleted
            let mut added = Vec::new();
            let mut modified = Vec::new();
            let mut deleted = Vec::new();

            for email in emails {
                if email.id.is_none() {
                    // No local ID = new email
                    added.push(email);
                } else if email.remote_id.is_empty() {
                    // Empty remote_id indicates deletion
                    deleted.push(email.remote_id);
                } else {
                    // Has ID and remote_id = modified
                    modified.push(email);
                }
            }

            // Enrich added and modified emails with attachment data (download ALL)
            self.enrich_emails_with_attachments(&mut added, true)
                .await
                .ok();
            self.enrich_emails_with_attachments(&mut modified, true)
                .await
                .ok();

            Ok(crate::sync::types::SyncDiff {
                added,
                modified,
                deleted,
                next_sync_token: next_token,
            })
        } else {
            // Full sync: fetch all emails
            let (mut emails, next_token) = self.fetch_emails_full(folder).await?;

            // Enrich emails with attachment data (download ALL)
            self.enrich_emails_with_attachments(&mut emails, true)
                .await
                .ok();

            Ok(crate::sync::types::SyncDiff {
                added: emails,
                modified: Vec::new(),
                deleted: Vec::new(),
                next_sync_token: next_token,
            })
        }
    }

    async fn fetch_email(&self, folder: &SyncFolder, remote_id: &str) -> SyncResult<SyncEmail> {
        let remote_id_owned = remote_id.to_string();

        let response = self
            .execute_with_401_retry(|token| {
                let client = self.client.clone();
                let remote_id = remote_id_owned.clone();
                async move {
                    client
                        .get(format!("{}/me/messages/{}", GRAPH_API_BASE, remote_id))
                        .bearer_auth(token)
                        .send()
                        .await
                }
            })
            .await?;

        if !response.status().is_success() {
            return Err(SyncError::Office365Error(format!(
                "Failed to fetch message: {}",
                response.status()
            )));
        }

        let message: GraphMessage = response.json().await?;
        let folder_id = folder
            .id
            .ok_or_else(|| SyncError::DatabaseError("Folder ID is required".to_string()))?;

        let email = Self::parse_graph_message(&message, folder_id, self.account_id, true)?;

        // Enrich with attachment data (download ALL)
        let mut emails = vec![email];
        self.enrich_emails_with_attachments(&mut emails, true)
            .await
            .ok();
        let email = emails.into_iter().next().unwrap();

        Ok(email)
    }

    async fn fetch_attachment(&self, attachment: &SyncAttachment) -> SyncResult<Vec<u8>> {
        let remote_path = attachment.remote_path.as_ref().ok_or_else(|| {
            SyncError::AttachmentError("No remote path for attachment".to_string())
        })?;

        let (message_id, attachment_id) = remote_path.rsplit_once(':').ok_or_else(|| {
            SyncError::AttachmentError(format!(
                "Invalid remote path format: {}. Expected 'message_id:attachment_id'",
                remote_path
            ))
        })?;

        log::debug!(
            "Downloading attachment {} ({} bytes)",
            attachment.filename,
            attachment.size
        );

        let message_id_owned = message_id.to_string();
        let attachment_id_owned = attachment_id.to_string();
        let filename = attachment.filename.clone();
        let expected_size = attachment.size;

        let response = self
            .execute_with_401_retry(|token| {
                let client = self.client.clone();
                let msg_id = message_id_owned.clone();
                let att_id = attachment_id_owned.clone();
                async move {
                    client
                        .get(format!(
                            "{}/me/messages/{}/attachments/{}/$value",
                            GRAPH_API_BASE, msg_id, att_id
                        ))
                        .bearer_auth(token)
                        .timeout(std::time::Duration::from_secs(300))
                        .send()
                        .await
                }
            })
            .await?;

        let status = response.status();
        if !status.is_success() {
            return Err(SyncError::Office365Error(format!(
                "Failed to download attachment {} (status {}): {}",
                filename,
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown error")
            )));
        }

        if let Some(content_length) = response.content_length() {
            if content_length as i64 != expected_size {
                log::warn!(
                    "Attachment size mismatch for {}: expected {}, got {}",
                    filename,
                    expected_size,
                    content_length
                );
            }
        }

        let bytes = response.bytes().await.map_err(|e| {
            SyncError::NetworkError(format!(
                "Failed to read attachment bytes for {}: {}",
                filename, e
            ))
        })?;

        log::debug!(
            "Successfully downloaded attachment {} ({} bytes)",
            filename,
            bytes.len()
        );

        Ok(bytes.to_vec())
    }

    async fn move_email(
        &self,
        email_remote_id: &str,
        _from_folder: &SyncFolder,
        to_folder: &SyncFolder,
    ) -> SyncResult<()> {
        #[derive(Serialize, Clone)]
        struct MoveRequest {
            #[serde(rename = "destinationId")]
            destination_id: String,
        }

        let request = MoveRequest {
            destination_id: to_folder.remote_id.clone(),
        };

        let email_remote_id_owned = email_remote_id.to_string();

        let response = self
            .execute_with_401_retry(|token| {
                let client = self.client.clone();
                let remote_id = email_remote_id_owned.clone();
                let req = request.clone();
                async move {
                    client
                        .post(format!("{}/me/messages/{}/move", GRAPH_API_BASE, remote_id))
                        .bearer_auth(token)
                        .json(&req)
                        .send()
                        .await
                }
            })
            .await?;

        if !response.status().is_success() {
            return Err(SyncError::Office365Error(format!(
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
        _permanent: bool,
    ) -> SyncResult<()> {
        let email_remote_id_owned = email_remote_id.to_string();

        let response = self
            .execute_with_401_retry(|token| {
                let client = self.client.clone();
                let remote_id = email_remote_id_owned.clone();
                async move {
                    client
                        .delete(format!("{}/me/messages/{}", GRAPH_API_BASE, remote_id))
                        .bearer_auth(token)
                        .send()
                        .await
                }
            })
            .await?;

        if !response.status().is_success() {
            return Err(SyncError::Office365Error(format!(
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
        #[derive(Serialize, Clone)]
        struct UpdateRequest {
            #[serde(rename = "isRead")]
            is_read: bool,
        }

        let request = UpdateRequest { is_read };
        let email_remote_id_owned = email_remote_id.to_string();

        let response = self
            .execute_with_401_retry(|token| {
                let client = self.client.clone();
                let remote_id = email_remote_id_owned.clone();
                let req = request.clone();
                async move {
                    client
                        .patch(format!("{}/me/messages/{}", GRAPH_API_BASE, remote_id))
                        .bearer_auth(token)
                        .json(&req)
                        .send()
                        .await
                }
            })
            .await?;

        if !response.status().is_success() {
            return Err(SyncError::Office365Error(format!(
                "Failed to update message: {}",
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
        #[derive(Serialize, Clone)]
        struct FlagRequest {
            flag: FlagValue,
        }

        #[derive(Serialize, Clone)]
        struct FlagValue {
            #[serde(rename = "flagStatus")]
            flag_status: String,
        }

        let request = FlagRequest {
            flag: FlagValue {
                flag_status: if flagged {
                    "flagged".to_string()
                } else {
                    "notFlagged".to_string()
                },
            },
        };

        let email_remote_id_owned = email_remote_id.to_string();

        let response = self
            .execute_with_401_retry(|token| {
                let client = self.client.clone();
                let remote_id = email_remote_id_owned.clone();
                let req = request.clone();
                async move {
                    client
                        .patch(format!("{}/me/messages/{}", GRAPH_API_BASE, remote_id))
                        .bearer_auth(token)
                        .json(&req)
                        .send()
                        .await
                }
            })
            .await?;

        if !response.status().is_success() {
            return Err(SyncError::Office365Error(format!(
                "Failed to update message: {}",
                response.status()
            )));
        }

        Ok(())
    }

    async fn rename_folder(&self, folder: &SyncFolder, new_name: &str) -> SyncResult<()> {
        #[derive(Serialize, Clone)]
        struct RenameFolderRequest {
            #[serde(rename = "displayName")]
            display_name: String,
        }

        let request = RenameFolderRequest {
            display_name: new_name.to_string(),
        };

        let folder_remote_id = folder.remote_id.clone();
        let folder_name = folder.name.clone();
        let new_name_owned = new_name.to_string();

        let response = self
            .execute_with_401_retry(|token| {
                let client = self.client.clone();
                let remote_id = folder_remote_id.clone();
                let req = request.clone();
                async move {
                    client
                        .patch(format!("{}/me/mailFolders/{}", GRAPH_API_BASE, remote_id))
                        .bearer_auth(token)
                        .json(&req)
                        .send()
                        .await
                }
            })
            .await?;

        if !response.status().is_success() {
            return Err(SyncError::Office365Error(format!(
                "Failed to rename folder: {}",
                response.status()
            )));
        }

        log::info!(
            "Renamed Office365 folder '{}' to '{}'",
            folder_name,
            new_name_owned
        );
        Ok(())
    }

    async fn move_folder(
        &self,
        folder: &SyncFolder,
        new_parent_path: Option<&str>,
    ) -> SyncResult<()> {
        let parent_folder_id = new_parent_path.unwrap_or("msgfolderroot").to_string();

        #[derive(Serialize, Clone)]
        struct MoveFolderRequest {
            #[serde(rename = "destinationId")]
            destination_id: String,
        }

        let request = MoveFolderRequest {
            destination_id: parent_folder_id.clone(),
        };

        let folder_remote_id = folder.remote_id.clone();
        let folder_name = folder.name.clone();
        let parent_id_for_log = parent_folder_id.clone();

        let response = self
            .execute_with_401_retry(|token| {
                let client = self.client.clone();
                let remote_id = folder_remote_id.clone();
                let req = request.clone();
                async move {
                    client
                        .post(format!(
                            "{}/me/mailFolders/{}/move",
                            GRAPH_API_BASE, remote_id
                        ))
                        .bearer_auth(token)
                        .json(&req)
                        .send()
                        .await
                }
            })
            .await?;

        if !response.status().is_success() {
            return Err(SyncError::Office365Error(format!(
                "Failed to move folder: {}",
                response.status()
            )));
        }

        log::info!(
            "Moved Office365 folder '{}' to parent '{}'",
            folder_name,
            parent_id_for_log
        );
        Ok(())
    }

    async fn get_sync_token(&self) -> SyncResult<Option<String>> {
        Ok(None)
    }

    async fn sync_since_token(&self, _token: &str) -> SyncResult<Vec<SyncEmail>> {
        log::warn!("Office365 sync_since_token is deprecated, use fetch_emails_delta instead");
        Ok(Vec::new())
    }

    async fn send_email(
        &self,
        to: Vec<crate::sync::types::EmailRecipient>,
        cc: Vec<crate::sync::types::EmailRecipient>,
        bcc: Vec<crate::sync::types::EmailRecipient>,
        subject: String,
        body_html: String,
        attachments: Vec<crate::sync::types::EmailAttachmentData>,
    ) -> SyncResult<()> {
        log::info!("[Office365] Sending email with subject: {}", subject);

        #[derive(Serialize)]
        struct SendMailRequest {
            message: Message,
            #[serde(rename = "saveToSentItems")]
            save_to_sent_items: bool,
        }

        #[derive(Serialize)]
        struct Message {
            subject: String,
            body: Body,
            #[serde(rename = "toRecipients")]
            to_recipients: Vec<Recipient>,
            #[serde(rename = "ccRecipients", skip_serializing_if = "Vec::is_empty")]
            cc_recipients: Vec<Recipient>,
            #[serde(rename = "bccRecipients", skip_serializing_if = "Vec::is_empty")]
            bcc_recipients: Vec<Recipient>,
            #[serde(skip_serializing_if = "Vec::is_empty")]
            attachments: Vec<Attachment>,
        }

        #[derive(Serialize)]
        struct Body {
            #[serde(rename = "contentType")]
            content_type: String,
            content: String,
        }

        #[derive(Serialize)]
        struct Recipient {
            #[serde(rename = "emailAddress")]
            email_address: EmailAddr,
        }

        #[derive(Serialize)]
        struct EmailAddr {
            address: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<String>,
        }

        #[derive(Serialize)]
        struct Attachment {
            #[serde(rename = "@odata.type")]
            odata_type: String,
            name: String,
            #[serde(rename = "contentType")]
            content_type: String,
            #[serde(rename = "contentBytes")]
            content_bytes: String,
        }

        let to_recipients: Vec<Recipient> = to
            .into_iter()
            .map(|r| Recipient {
                email_address: EmailAddr {
                    address: r.address,
                    name: r.name,
                },
            })
            .collect();

        let cc_recipients: Vec<Recipient> = cc
            .into_iter()
            .map(|r| Recipient {
                email_address: EmailAddr {
                    address: r.address,
                    name: r.name,
                },
            })
            .collect();

        let bcc_recipients: Vec<Recipient> = bcc
            .into_iter()
            .map(|r| Recipient {
                email_address: EmailAddr {
                    address: r.address,
                    name: r.name,
                },
            })
            .collect();

        let graph_attachments: Vec<Attachment> = attachments
            .into_iter()
            .map(|att| {
                use base64::{engine::general_purpose, Engine as _};
                Attachment {
                    odata_type: "#microsoft.graph.fileAttachment".to_string(),
                    name: att.filename,
                    content_type: att
                        .content_type
                        .unwrap_or_else(|| "application/octet-stream".to_string()),
                    content_bytes: general_purpose::STANDARD.encode(&att.content),
                }
            })
            .collect();

        let request_body = SendMailRequest {
            message: Message {
                subject,
                body: Body {
                    content_type: "HTML".to_string(),
                    content: body_html,
                },
                to_recipients,
                cc_recipients,
                bcc_recipients,
                attachments: graph_attachments,
            },
            save_to_sent_items: true,
        };

        let response = self
            .execute_with_401_retry(|token| {
                let client = self.client.clone();
                let body = serde_json::to_value(&request_body).unwrap();
                async move {
                    client
                        .post(format!("{}/me/sendMail", GRAPH_API_BASE))
                        .bearer_auth(token)
                        .json(&body)
                        .send()
                        .await
                }
            })
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error".to_string());
            return Err(SyncError::Office365Error(format!(
                "Failed to send email (status {}): {}",
                status, error_text
            )));
        }

        log::info!("[Office365] Email sent successfully");
        Ok(())
    }
}

fn fetch_child_folders_recursive<'a>(
    provider: &'a Office365Provider,
    parent_remote_id: String,
    all_folders: &'a mut Vec<SyncFolder>,
    depth: usize,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = SyncResult<()>> + Send + 'a>> {
    Box::pin(async move {
        const MAX_FOLDER_DEPTH: usize = 50;

        // Prevent unbounded recursion - stop at max depth
        if depth > MAX_FOLDER_DEPTH {
            log::warn!(
                "[Office365] Skipping child folders for {} at depth {} (max: {})",
                parent_remote_id,
                depth,
                MAX_FOLDER_DEPTH
            );
            return Ok(());
        }

        let mut next_link: Option<String> = None;
        let mut page = 0;

        loop {
            page += 1;
            let token = provider.ensure_token().await?;

            let url = if let Some(ref link) = next_link {
                link.clone()
            } else {
                format!(
                    "{}/me/mailFolders/{}/childFolders",
                    GRAPH_API_BASE, parent_remote_id
                )
            };

            let response = provider.client.get(&url).bearer_auth(&token).send().await?;

            if !response.status().is_success() {
                log::error!(
                    "Failed to fetch child folders page {} for {}: {}",
                    page,
                    parent_remote_id,
                    response.status()
                );
                return Ok(());
            }

            let folders_response: GraphFoldersResponse = response.json().await?;

            for folder in folders_response.value {
                let folder_type = Office365Provider::map_folder_type(&folder.display_name);
                let has_children = folder.child_folder_count.unwrap_or(0) > 0;
                let folder_remote_id = folder.id.clone();

                let mut attributes = Vec::new();
                if let Some(ref parent_id) = folder.parent_folder_id {
                    attributes.push(format!("parent_remote_id:{}", parent_id));
                }

                let sync_folder = SyncFolder {
                    id: None,
                    account_id: provider.account_id,
                    name: folder.display_name,
                    folder_type,
                    remote_id: folder_remote_id.clone(),
                    icon: None,
                    color: None,
                    parent_id: None,
                    attributes,
                    sync_interval: 0,
                    synced_at: None,
                    unread_count: folder.unread_item_count.unwrap_or(0),
                    total_count: folder.total_item_count.unwrap_or(0),
                    expanded: false,
                    hidden: false,
                };

                all_folders.push(sync_folder);

                if has_children {
                    fetch_child_folders_recursive(
                        provider,
                        folder_remote_id,
                        all_folders,
                        depth + 1,
                    )
                    .await?;
                }
            }

            next_link = folders_response.next_link;
            if next_link.is_none() {
                break;
            }
        }

        Ok(())
    })
}
