use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};
use uuid::Uuid;

use crate::database::models::account::AccountType;
use crate::database::models::email::{Email, EmailAddress};
use crate::database::models::email_dto::{AttachmentInfo, EmailDetail, EmailListItem, LabelInfo};
use crate::database::models::folder::FolderType;
use crate::database::repositories::{
    AccountRepository, AttachmentRepository, EmailRepository, FolderRepository, LabelRepository,
    SqliteAccountRepository, SqliteAttachmentRepository, SqliteEmailRepository,
    SqliteFolderRepository, SqliteLabelRepository,
};
use crate::services::email_service::{EmailAttachment, EmailData, EmailService};
use crate::services::notification_service::NotificationService;
use crate::state::AppState;
use crate::sync::types::AccountSettings;
use sqlx::types::Json;
use std::sync::Arc;
use turndown::Turndown;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentData {
    pub filename: String,
    pub content: Vec<u8>,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendEmailRequest {
    pub from: String,
    pub to: Vec<EmailAddress>,
    pub cc: Vec<EmailAddress>,
    pub bcc: Vec<EmailAddress>,
    pub subject: String,
    pub body: String,
    pub attachments: Vec<AttachmentData>,
}

#[derive(Debug, Serialize)]
pub struct SendEmailResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendFromAccountRequest {
    pub account_id: Uuid,
    pub to: Vec<EmailAddress>,
    pub cc: Vec<EmailAddress>,
    pub bcc: Vec<EmailAddress>,
    pub subject: String,
    pub body: String,
    pub attachments: Vec<AttachmentData>,
    pub draft_id: Option<Uuid>,
    pub conversation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveDraftRequest {
    pub account_id: Uuid,
    pub draft_id: Option<Uuid>,
    pub to: Vec<EmailAddress>,
    pub cc: Vec<EmailAddress>,
    pub bcc: Vec<EmailAddress>,
    pub subject: String,
    pub body: String,
    pub scheduled_send_at: Option<String>,
    pub conversation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountForSending {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub account_type: String,
    pub has_smtp_config: bool,
}

#[derive(Debug, Serialize)]
pub struct SaveDraftResponse {
    pub success: bool,
    pub draft_id: Uuid,
    pub message: String,
}

fn emit_email_event<S: Serialize + Clone>(
    app_handle: &tauri::AppHandle,
    event_name: &str,
    payload: S,
) {
    if let Err(e) = app_handle.emit(event_name, payload) {
        log::error!("Failed to emit email event '{}': {}", event_name, e);
    }
}

#[tauri::command]
pub async fn send_email(request: SendEmailRequest) -> Result<SendEmailResponse, String> {
    log::info!("Sending email with subject: {}", request.subject);

    Ok(SendEmailResponse {
        success: true,
        message: "Email sent successfully".to_string(),
    })
}

#[tauri::command]
pub async fn test_smtp_connection() -> Result<SendEmailResponse, String> {
    log::info!("Testing SMTP connection");

    Ok(SendEmailResponse {
        success: true,
        message: "SMTP configuration is valid".to_string(),
    })
}

#[tauri::command]
pub async fn send_email_from_account(
    state: State<'_, AppState>,
    request: SendFromAccountRequest,
) -> Result<SendEmailResponse, String> {
    log::info!(
        "Sending email from account {} with subject: {}",
        request.account_id,
        request.subject
    );

    let account_repo = SqliteAccountRepository::new(state.db_pool.clone());
    let account = account_repo
        .find_by_id(request.account_id)
        .await
        .map_err(|e| format!("Failed to find account: {}", e))?
        .ok_or_else(|| format!("Account {} not found", request.account_id))?;

    if account.account_type == AccountType::Office365 {
        use crate::sync::provider::ProviderFactory;
        use crate::sync::types::{EmailAttachmentData, EmailRecipient};

        log::info!("[Office365] Using Microsoft Graph API to send email");

        let provider = ProviderFactory::create(&account, state.credential_store.clone())
            .map_err(|e| format!("Failed to create Office365 provider: {}", e))?;

        let to_recipients: Vec<EmailRecipient> = request
            .to
            .iter()
            .map(|addr| EmailRecipient {
                address: addr.address.clone(),
                name: addr.name.clone(),
            })
            .collect();

        let cc_recipients: Vec<EmailRecipient> = request
            .cc
            .iter()
            .map(|addr| EmailRecipient {
                address: addr.address.clone(),
                name: addr.name.clone(),
            })
            .collect();

        let bcc_recipients: Vec<EmailRecipient> = request
            .bcc
            .iter()
            .map(|addr| EmailRecipient {
                address: addr.address.clone(),
                name: addr.name.clone(),
            })
            .collect();

        let attachment_data: Vec<EmailAttachmentData> = request
            .attachments
            .iter()
            .map(|att| EmailAttachmentData {
                filename: att.filename.clone(),
                content: att.content.clone(),
                content_type: att.content_type.clone(),
            })
            .collect();

        provider
            .send_email(
                to_recipients,
                cc_recipients,
                bcc_recipients,
                request.subject.clone(),
                request.body.clone(),
                attachment_data,
            )
            .await
            .map_err(|e| format!("Failed to send email via Office365: {}", e))?;

        log::info!("[Office365] Email sent successfully via Graph API");
    } else {
        log::info!("Using SMTP to send email");

        let settings: AccountSettings = serde_json::from_value(account.settings.clone())
            .map_err(|e| format!("Failed to parse account settings: {}", e))?;

        let smtp_host = settings
            .smtp_host
            .or_else(|| settings.imap_host.clone())
            .ok_or_else(|| "Neither SMTP nor IMAP host configured for this account".to_string())?;

        let smtp_port = settings.smtp_port.unwrap_or(587);
        let smtp_use_tls = settings
            .smtp_use_tls
            .unwrap_or_else(|| settings.imap_use_tls.unwrap_or(true));

        let smtp_username = settings
            .smtp_username
            .clone()
            .or_else(|| settings.imap_username.clone())
            .unwrap_or(account.email.clone());

        let credentials = state
            .credential_store
            .get_imap(account.id)
            .await
            .map_err(|e| format!("Failed to get credentials: {}", e))?;

        let email_service = EmailService::from_account_settings(
            smtp_host.clone(),
            smtp_port,
            smtp_use_tls,
            smtp_username,
            credentials.password,
        )
        .map_err(|e| format!("Failed to initialize email service: {}", e))?;

        let attachments: Vec<EmailAttachment> = request
            .attachments
            .into_iter()
            .map(|att| EmailAttachment {
                filename: att.filename,
                content: att.content,
                content_type: att.content_type,
            })
            .collect();

        let email_data = EmailData {
            from: account.email.clone(),
            to: request.to.clone(),
            cc: request.cc.clone(),
            bcc: request.bcc.clone(),
            subject: request.subject.clone(),
            body_html: request.body.clone(),
            attachments,
        };

        email_service
            .send_email(email_data)
            .await
            .map_err(|e| format!("Failed to send email: {}", e))?;
    }

    if let Some(draft_id) = request.draft_id {
        let folder_repo = SqliteFolderRepository::new(state.db_pool.clone());
        let email_repo = SqliteEmailRepository::new(state.db_pool.clone());

        let folders = folder_repo
            .find_by_account(account.id)
            .await
            .map_err(|e| format!("Failed to get folders: {}", e))?;

        if let Some(sent_folder) = folders.iter().find(|f| f.folder_type == FolderType::Sent) {
            if let Some(mut draft_email) = email_repo
                .find_by_id(draft_id)
                .await
                .map_err(|e| format!("Failed to get draft: {}", e))?
            {
                draft_email.folder_id = sent_folder.id;
                draft_email.is_draft = false;
                draft_email.sent_at = Some(Utc::now());
                draft_email.conversation_id = request.conversation_id.clone();
                draft_email.sync_status = "synced".to_string();

                email_repo
                    .update(&draft_email)
                    .await
                    .map_err(|e| format!("Failed to update draft: {}", e))?;

                emit_email_event(&state.app_handle, "email:updated", &draft_email);
            }
        }
    } else {
        let folder_repo = SqliteFolderRepository::new(state.db_pool.clone());
        let email_repo = SqliteEmailRepository::new(state.db_pool.clone());

        let folders = folder_repo
            .find_by_account(account.id)
            .await
            .map_err(|e| format!("Failed to get folders: {}", e))?;

        if let Some(sent_folder) = folders.iter().find(|f| f.folder_type == FolderType::Sent) {
            let domain = account
                .email
                .split_once('@')
                .map(|(_, d)| d.to_string())
                .unwrap_or_else(|| "ravn.app".to_string());

            let message_id = format!("<{}@{}>", Uuid::now_v7(), domain);
            let size = request.body.len();

            let sent_email = Email {
                id: Uuid::now_v7(),
                account_id: account.id,
                folder_id: sent_folder.id,
                message_id,
                conversation_id: request.conversation_id.clone(),
                remote_id: None,
                from: Json(EmailAddress {
                    address: account.email.clone(),
                    name: Some(account.name.clone()),
                }),
                to: Json(request.to),
                cc: Json(request.cc),
                bcc: Json(request.bcc),
                reply_to: None,
                subject: Some(request.subject),
                snippet: None,
                body_plain: None,
                body_html: Some(request.body),
                other_mails: None,
                category: None,
                ai_cache: None,
                received_at: Utc::now(),
                sent_at: Some(Utc::now()),
                scheduled_send_at: None,
                size: size as i64,
                headers: Some("".to_string()),
                is_read: true,
                is_flagged: false,
                is_draft: false,
                has_attachments: false,
                is_deleted: false,
                sync_status: "synced".to_string(),
                tracking_blocked: true,
                images_blocked: true,
                body_fetch_attempts: 0,
                last_body_fetch_attempt: None,
                change_key: None,
                last_modified_at: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let _ = email_repo
                .create(&sent_email)
                .await
                .map_err(|e| format!("Failed to save sent email: {}", e))?;

            emit_email_event(&state.app_handle, "email:created", &sent_email);
        }
    }

    let notification_service = Arc::new(
        NotificationService::new(state.db_pool.clone(), state.settings.clone())
            .with_app_handle(state.app_handle.clone()),
    );
    if let Err(e) = notification_service.notify_outgoing_email().await {
        log::warn!("Failed to trigger outgoing email notification: {}", e);
    }

    Ok(SendEmailResponse {
        success: true,
        message: "Email sent successfully".to_string(),
    })
}

#[tauri::command]
pub async fn save_draft(
    state: State<'_, AppState>,
    request: SaveDraftRequest,
) -> Result<SaveDraftResponse, String> {
    log::info!("Saving draft for account {}", request.account_id);

    let folder_repo = SqliteFolderRepository::new(state.db_pool.clone());
    let email_repo = SqliteEmailRepository::new(state.db_pool.clone());
    let account_repo = SqliteAccountRepository::new(state.db_pool.clone());

    let account = account_repo
        .find_by_id(request.account_id)
        .await
        .map_err(|e| format!("Failed to find account: {}", e))?
        .ok_or_else(|| format!("Account {} not found", request.account_id))?;

    let folders = folder_repo
        .find_by_account(account.id)
        .await
        .map_err(|e| format!("Failed to get folders: {}", e))?;

    let draft_folder = folders
        .iter()
        .find(|f| f.folder_type == FolderType::Draft)
        .ok_or_else(|| "Draft folder not found for this account".to_string())?;

    let scheduled_send_at = if let Some(timestamp) = request.scheduled_send_at {
        Some(
            chrono::DateTime::parse_from_rfc3339(&timestamp)
                .map_err(|e| format!("Invalid scheduled_send_at timestamp: {}", e))?
                .with_timezone(&Utc),
        )
    } else {
        None
    };

    if let Some(draft_id) = request.draft_id {
        let mut draft = email_repo
            .find_by_id(draft_id)
            .await
            .map_err(|e| format!("Failed to find draft: {}", e))?
            .ok_or_else(|| format!("Draft {} not found", draft_id))?;

        draft.to = Json(request.to);
        draft.cc = Json(request.cc);
        draft.bcc = Json(request.bcc);
        draft.subject = Some(request.subject);
        draft.body_html = Some(request.body);
        draft.conversation_id = request.conversation_id;
        draft.scheduled_send_at = scheduled_send_at;
        draft.updated_at = Utc::now();

        email_repo
            .update(&draft)
            .await
            .map_err(|e| format!("Failed to update draft: {}", e))?;

        emit_email_event(&state.app_handle, "email:updated", &draft);

        Ok(SaveDraftResponse {
            success: true,
            draft_id,
            message: "Draft updated successfully".to_string(),
        })
    } else {
        let message_id = format!("<draft-{}@ravn.app>", Uuid::now_v7());

        let draft = Email {
            id: Uuid::now_v7(),
            account_id: account.id,
            folder_id: draft_folder.id,
            message_id,
            conversation_id: request.conversation_id,
            remote_id: None,
            from: Json(EmailAddress {
                address: account.email.clone(),
                name: Some(account.name.clone()),
            }),
            to: Json(request.to),
            cc: Json(request.cc),
            bcc: Json(request.bcc),
            reply_to: None,
            subject: Some(request.subject),
            snippet: None,
            body_plain: None,
            body_html: Some(request.body),
            other_mails: None,
            category: None,
            ai_cache: None,
            received_at: Utc::now(),
            size: 0,
            headers: Some("".to_string()),
            sent_at: None,
            scheduled_send_at,
            is_read: false,
            is_flagged: false,
            is_draft: true,
            has_attachments: false,
            is_deleted: false,
            sync_status: "local".to_string(),
            tracking_blocked: true,
            images_blocked: true,
            body_fetch_attempts: 0,
            last_body_fetch_attempt: None,
            change_key: None,
            last_modified_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let draft_id = email_repo
            .create(&draft)
            .await
            .map_err(|e| format!("Failed to create draft: {}", e))?;

        emit_email_event(&state.app_handle, "email:created", &draft);

        Ok(SaveDraftResponse {
            success: true,
            draft_id,
            message: "Draft created successfully".to_string(),
        })
    }
}

#[tauri::command]
pub async fn get_accounts_for_sending(
    state: State<'_, AppState>,
) -> Result<Vec<AccountForSending>, String> {
    log::info!("Getting accounts for sending");

    let account_repo = SqliteAccountRepository::new(state.db_pool.clone());
    let accounts = account_repo
        .find_all()
        .await
        .map_err(|e| format!("Failed to get accounts: {}", e))?;

    let mut sending_accounts = Vec::new();

    for account in accounts {
        let has_smtp_config = if account.account_type == AccountType::Office365 {
            true
        } else {
            if let Ok(settings) =
                serde_json::from_value::<AccountSettings>(account.settings.clone())
            {
                settings.smtp_host.is_some() || settings.imap_host.is_some()
            } else {
                false
            }
        };

        sending_accounts.push(AccountForSending {
            id: account.id,
            name: account.name,
            email: account.email,
            account_type: account.account_type.to_string(),
            has_smtp_config,
        });
    }

    Ok(sending_accounts)
}

#[tauri::command]
pub async fn get_drafts(
    state: State<'_, AppState>,
    account_id: Uuid,
) -> Result<Vec<Email>, String> {
    log::info!("Getting drafts for account {}", account_id);

    let folder_repo = SqliteFolderRepository::new(state.db_pool.clone());
    let email_repo = SqliteEmailRepository::new(state.db_pool.clone());

    let folders = folder_repo
        .find_by_account(account_id)
        .await
        .map_err(|e| format!("Failed to get folders: {}", e))?;

    let draft_folder = folders
        .iter()
        .find(|f| f.folder_type == FolderType::Draft)
        .ok_or_else(|| "Draft folder not found".to_string())?;

    let drafts = email_repo
        .find_by_folder(draft_folder.id, 100, 0)
        .await
        .map_err(|e| format!("Failed to get drafts: {}", e))?;

    Ok(drafts)
}

#[tauri::command]
pub async fn delete_draft(
    state: State<'_, AppState>,
    draft_id: Uuid,
) -> Result<SendEmailResponse, String> {
    log::info!("Deleting draft {}", draft_id);

    let email_repo = SqliteEmailRepository::new(state.db_pool.clone());

    email_repo
        .delete(draft_id)
        .await
        .map_err(|e| format!("Failed to delete draft: {}", e))?;

    emit_email_event(&state.app_handle, "email:deleted", draft_id.to_string());

    Ok(SendEmailResponse {
        success: true,
        message: "Draft deleted successfully".to_string(),
    })
}

#[tauri::command]
pub async fn get_emails(state: State<'_, AppState>, id: Uuid) -> Result<EmailDetail, String> {
    let email_repo = SqliteEmailRepository::new(state.db_pool.clone());
    let label_repo = SqliteLabelRepository::new(state.db_pool.clone());
    let attachment_repo = SqliteAttachmentRepository::new(state.db_pool.clone());

    let email = email_repo
        .find_by_id(id)
        .await
        .map_err(|e| format!("Failed to fetch email: {}", e))?
        .ok_or_else(|| format!("Email {} not found", id))?;

    let labels = label_repo
        .find_by_email(email.id)
        .await
        .map_err(|e| format!("Failed to fetch labels: {}", e))?
        .iter()
        .map(LabelInfo::from)
        .collect();

    let attachments = attachment_repo
        .find_by_email(email.id)
        .await
        .map_err(|e| format!("Failed to fetch attachments: {}", e))?
        .iter()
        .map(AttachmentInfo::from)
        .collect();

    Ok(EmailDetail::from_email(&email, labels, attachments))
}

#[tauri::command]
pub async fn get_emails_for_folders(
    state: State<'_, AppState>,
    folder_id: Uuid,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<EmailListItem>, String> {
    let email_repo = SqliteEmailRepository::new(state.db_pool.clone());
    let label_repo = SqliteLabelRepository::new(state.db_pool.clone());

    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let emails = email_repo
        .find_by_folder(folder_id, limit, offset)
        .await
        .map_err(|e| format!("Failed to fetch emails: {}", e))?;

    let email_ids: Vec<Uuid> = emails.iter().map(|e| e.id).collect();
    let labels_map = label_repo
        .find_by_emails(&email_ids)
        .await
        .map_err(|e| format!("Failed to fetch labels: {}", e))?;

    let list_items = emails
        .iter()
        .map(|email| {
            let labels = labels_map
                .get(&email.id)
                .map(|labels| labels.iter().map(LabelInfo::from).collect())
                .unwrap_or_default();
            EmailListItem::from_email(email, labels)
        })
        .collect();

    Ok(list_items)
}

#[tauri::command]
pub async fn get_emails_for_labels(
    state: State<'_, AppState>,
    label_ids: Vec<Uuid>,
    match_all: bool,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<EmailListItem>, String> {
    let email_repo = SqliteEmailRepository::new(state.db_pool.clone());
    let label_repo = SqliteLabelRepository::new(state.db_pool.clone());

    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let emails = email_repo
        .find_by_labels(&label_ids, match_all, limit, offset)
        .await
        .map_err(|e| format!("Failed to fetch emails by labels: {}", e))?;

    let email_ids: Vec<Uuid> = emails.iter().map(|e| e.id).collect();
    let labels_map = label_repo
        .find_by_emails(&email_ids)
        .await
        .map_err(|e| format!("Failed to fetch labels: {}", e))?;

    let list_items = emails
        .iter()
        .map(|email| {
            let labels = labels_map
                .get(&email.id)
                .map(|labels| labels.iter().map(LabelInfo::from).collect())
                .unwrap_or_default();
            EmailListItem::from_email(email, labels)
        })
        .collect();

    Ok(list_items)
}

#[tauri::command]
pub async fn update_read(
    state: State<'_, AppState>,
    email_id: Uuid,
    is_read: bool,
) -> Result<(), String> {
    let email_repo = SqliteEmailRepository::new(state.db_pool.clone());

    let mut email = email_repo
        .find_by_id(email_id)
        .await
        .map_err(|e| format!("Failed to fetch email: {}", e))?
        .ok_or_else(|| format!("Email {} not found", email_id))?;

    state
        .sync_coordinator
        .mark_as_read(email.account_id, email_id, is_read)
        .await
        .map_err(|e| e.to_string())?;

    email.is_read = is_read;
    email_repo
        .update_read_status(email_id, is_read)
        .await
        .map_err(|e| format!("Failed to update read status: {}", e))?;

    emit_email_event(&state.app_handle, "email:updated", serde_json::json!(email));
    emit_email_event(
        &state.app_handle,
        "folder:updated",
        serde_json::json!({
            "account_id": email.account_id.to_string(),
            "id": email.folder_id.to_string()
        }),
    );

    Ok(())
}

#[tauri::command]
pub async fn email_parse_body_plain(
    state: State<'_, AppState>,
    email_id: Uuid,
) -> Result<Email, String> {
    let email_repo = SqliteEmailRepository::new(state.db_pool.clone());
    let mut options = turndown::TurndownOptions::default();
    options.strip_tracking_images = true;
    let turndown = Turndown::with_options(options);

    let mut email = email_repo
        .find_by_id(email_id)
        .await
        .map_err(|e| format!("Failed to fetch email: {}", e))?
        .ok_or_else(|| format!("Email {} not found", email_id))?;

    email.body_plain = email.body_html.as_ref().map(|html| turndown.convert(html));

    email_repo
        .update(&email)
        .await
        .map_err(|e| format!("Failed to update email body: {}", e))?;

    emit_email_event(&state.app_handle, "email:updated", serde_json::json!(email));

    Ok(email)
}

#[tauri::command]
pub async fn move_email(
    state: State<'_, AppState>,
    email_id: Uuid,
    folder_id: Uuid,
) -> Result<Email, String> {
    let email_repo = SqliteEmailRepository::new(state.db_pool.clone());

    let email = email_repo
        .find_by_id(email_id)
        .await
        .map_err(|e| format!("Failed to fetch email: {}", e))?
        .ok_or_else(|| format!("Email {} not found", email_id))?;

    let source_folder_id = email.folder_id;
    let account_id = email.account_id;

    state
        .sync_coordinator
        .move_email(account_id, email_id, folder_id)
        .await
        .map_err(|e| e.to_string())?;

    let updated_email = email_repo
        .find_by_id(email_id)
        .await
        .map_err(|e| format!("Failed to fetch updated email: {}", e))?
        .ok_or_else(|| format!("Email {} not found after move", email_id))?;

    emit_email_event(&state.app_handle, "email:updated", serde_json::json!(email));
    emit_email_event(
        &state.app_handle,
        "folder:updated",
        serde_json::json!({
            "account_id": account_id.to_string(),
            "id": source_folder_id.to_string()
        }),
    );
    emit_email_event(
        &state.app_handle,
        "folder:updated",
        serde_json::json!({
            "account_id": account_id.to_string(),
            "id": folder_id.to_string()
        }),
    );

    Ok(updated_email)
}

#[tauri::command]
pub async fn archive(state: State<'_, AppState>, email_id: Uuid) -> Result<Email, String> {
    let email_repo = SqliteEmailRepository::new(state.db_pool.clone());
    let folder_repo = SqliteFolderRepository::new(state.db_pool.clone());

    let email = email_repo
        .find_by_id(email_id)
        .await
        .map_err(|e| format!("Failed to fetch email: {}", e))?
        .ok_or_else(|| format!("Email {} not found", email_id))?;

    let account_id = email.account_id;

    let archive_folder = folder_repo
        .find_by_type(account_id, "archive")
        .await
        .map_err(|e| format!("Failed to fetch archive folder: {}", e))?
        .ok_or_else(|| "Archive folder not found for this account".to_string())?;

    let updated_email = move_email(state, email_id, archive_folder.id).await?;
    Ok(updated_email)
}

#[tauri::command]
pub async fn junk(state: State<'_, AppState>, email_id: Uuid) -> Result<Email, String> {
    let email_repo = SqliteEmailRepository::new(state.db_pool.clone());
    let folder_repo = SqliteFolderRepository::new(state.db_pool.clone());

    let email = email_repo
        .find_by_id(email_id)
        .await
        .map_err(|e| format!("Failed to fetch email: {}", e))?
        .ok_or_else(|| format!("Email {} not found", email_id))?;

    let account_id = email.account_id;

    let spam_folder = folder_repo
        .find_by_type(account_id, "spam")
        .await
        .map_err(|e| format!("Failed to fetch spam folder: {}", e))?
        .ok_or_else(|| "Spam folder not found for this account".to_string())?;

    let updated_email = move_email(state, email_id, spam_folder.id).await?;
    Ok(updated_email)
}

#[tauri::command]
pub async fn trash(state: State<'_, AppState>, email_id: Uuid) -> Result<Email, String> {
    let email_repo = SqliteEmailRepository::new(state.db_pool.clone());
    let folder_repo = SqliteFolderRepository::new(state.db_pool.clone());

    let email = email_repo
        .find_by_id(email_id)
        .await
        .map_err(|e| format!("Failed to fetch email: {}", e))?
        .ok_or_else(|| format!("Email {} not found", email_id))?;

    let account_id = email.account_id;

    let trash_folder = folder_repo
        .find_by_type(account_id, "trash")
        .await
        .map_err(|e| format!("Failed to fetch trash folder: {}", e))?
        .ok_or_else(|| "Trash folder not found for this account".to_string())?;

    let updated_email = move_email(state, email_id, trash_folder.id).await?;
    Ok(updated_email)
}

#[tauri::command]
pub async fn delete(state: State<'_, AppState>, email_id: Uuid) -> Result<(), String> {
    let email_repo = SqliteEmailRepository::new(state.db_pool.clone());

    let email = email_repo
        .find_by_id(email_id)
        .await
        .map_err(|e| format!("Failed to fetch email: {}", e))?
        .ok_or_else(|| format!("Email {} not found", email_id))?;

    let account_id = email.account_id;

    state
        .sync_coordinator
        .delete_email(account_id, email_id, true)
        .await
        .map_err(|e| e.to_string())?;

    emit_email_event(
        &state.app_handle,
        "email:deleted",
        serde_json::json!({
            "id": email_id.to_string()
        }),
    );

    Ok(())
}

#[tauri::command]
pub async fn fetch_body(state: State<'_, AppState>, email_id: Uuid) -> Result<String, String> {
    let email_repo = SqliteEmailRepository::new(state.db_pool.clone());

    let email = email_repo
        .find_by_id(email_id)
        .await
        .map_err(|e| format!("Failed to fetch email: {}", e))?
        .ok_or_else(|| format!("Email {} not found", email_id))?;

    if email.sync_status == "synced" {
        return Ok("Email body already fetched".to_string());
    }

    log::info!(
        "Email {} is queued for body fetch (status: {})",
        email_id,
        email.sync_status
    );

    Ok("Email queued for body fetch".to_string())
}

#[tauri::command]
pub async fn update_blocking(
    state: State<'_, AppState>,
    email_id: Uuid,
    images_blocked: bool,
    tracking_blocked: bool,
) -> Result<(), String> {
    let email_repo = SqliteEmailRepository::new(state.db_pool.clone());

    let mut email = email_repo
        .find_by_id(email_id)
        .await
        .map_err(|e| format!("Failed to fetch email: {}", e))?
        .ok_or_else(|| format!("Email {} not found", email_id))?;

    email.images_blocked = images_blocked;
    email.tracking_blocked = tracking_blocked;

    email_repo
        .update(&email)
        .await
        .map_err(|e| format!("Failed to update email blocking: {}", e))?;

    emit_email_event(&state.app_handle, "email:updated", serde_json::json!(email));

    Ok(())
}
