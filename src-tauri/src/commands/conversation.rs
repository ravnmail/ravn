use std::collections::HashSet;
/// Conversation/thread query commands using repository pattern and DTOs
use tauri::State;
use uuid::Uuid;

use crate::database::models::conversation::{ConversationDetail, ConversationListItem};
use crate::database::models::email_dto::{AttachmentInfo, EmailDetail, EmailListItem, LabelInfo};
use crate::database::repositories::{
    AttachmentRepository, ConversationRepository, EmailRepository, LabelRepository,
    SqliteAttachmentRepository, SqliteConversationRepository, SqliteEmailRepository,
    SqliteLabelRepository,
};
use crate::state::AppState;

/// Get conversations for a folder with minimal email data
#[tauri::command]
pub async fn get_conversations_for_folder(
    state: State<'_, AppState>,
    folder_id: Uuid,
    limit: Option<i64>,
    offset: Option<i64>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    filter_read: Option<bool>,
    filter_has_attachments: Option<bool>,
) -> Result<Vec<ConversationListItem>, String> {
    let email_repo = SqliteEmailRepository::new(state.db_pool.clone());
    let conversation_repo = SqliteConversationRepository::new(state.db_pool.clone());
    let label_repo = SqliteLabelRepository::new(state.db_pool.clone());

    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    let sort_by = sort_by.unwrap_or_else(|| "received_at".to_string());
    let sort_order = sort_order.unwrap_or_else(|| "desc".to_string());

    let emails = email_repo
        .find_by_folder_with_filters(
            folder_id,
            limit * 10,
            offset,
            &sort_by,
            &sort_order,
            filter_read,
            filter_has_attachments,
        )
        .await
        .map_err(|e| format!("Failed to fetch emails: {}", e))?;

    let conversation_ids: Vec<Uuid> = emails
        .iter()
        .filter_map(|email| email.conversation_id.as_ref())
        .filter_map(|id| Uuid::parse_str(id).ok())
        .collect::<HashSet<_>>()
        .into_iter()
        .take(limit as usize)
        .collect();

    if conversation_ids.is_empty() {
        return Ok(Vec::new());
    }

    let conversations = conversation_repo
        .find_by_ids(conversation_ids.clone())
        .await
        .map_err(|e| format!("Failed to fetch conversations: {}", e))?;

    let mut result = Vec::new();
    for conversation in conversations {
        let conversation_emails = email_repo
            .find_by_conversation_id(conversation.id)
            .await
            .map_err(|e| format!("Failed to fetch conversation emails: {}", e))?;

        let mut email_list_items = Vec::new();
        for email in conversation_emails {
            let labels = label_repo
                .find_by_email(email.id)
                .await
                .map_err(|e| format!("Failed to fetch labels: {}", e))?
                .iter()
                .map(LabelInfo::from)
                .collect();

            email_list_items.push(EmailListItem::from_email(&email, labels));
        }

        result.push(conversation.to_list_item(email_list_items));
    }

    Ok(result)
}

/// Get conversation by email message ID
#[tauri::command]
pub async fn get_conversation_for_message_id(
    state: State<'_, AppState>,
    message_id: String,
) -> Result<ConversationListItem, String> {
    let email_repo = SqliteEmailRepository::new(state.db_pool.clone());
    let conversation_repo = SqliteConversationRepository::new(state.db_pool.clone());
    let label_repo = SqliteLabelRepository::new(state.db_pool.clone());

    let email = email_repo
        .find_by_message_id(&message_id)
        .await
        .map_err(|e| format!("Failed to fetch email: {}", e))?
        .ok_or_else(|| format!("Email with message_id {} not found", message_id))?;

    let conversation_id_str = email
        .conversation_id
        .ok_or_else(|| format!("Email {} has no conversation_id", email.id))?;

    let conversation_id = Uuid::parse_str(&conversation_id_str)
        .map_err(|e| format!("Invalid conversation_id: {}", e))?;

    let conversation = conversation_repo
        .find_by_id(conversation_id)
        .await
        .map_err(|e| format!("Failed to fetch conversation: {}", e))?
        .ok_or_else(|| format!("Conversation {} not found", conversation_id))?;

    let conversation_emails = email_repo
        .find_by_conversation_id(conversation_id)
        .await
        .map_err(|e| format!("Failed to fetch conversation emails: {}", e))?;

    let mut email_list_items = Vec::new();
    for email in conversation_emails {
        let labels = label_repo
            .find_by_email(email.id)
            .await
            .map_err(|e| format!("Failed to fetch labels: {}", e))?
            .iter()
            .map(LabelInfo::from)
            .collect();

        email_list_items.push(EmailListItem::from_email(&email, labels));
    }

    Ok(conversation.to_list_item(email_list_items))
}

/// Get full conversation details by conversation ID
#[tauri::command]
pub async fn get_conversation_by_id(
    state: State<'_, AppState>,
    conversation_id: Uuid,
) -> Result<ConversationDetail, String> {
    let email_repo = SqliteEmailRepository::new(state.db_pool.clone());
    let conversation_repo = SqliteConversationRepository::new(state.db_pool.clone());
    let label_repo = SqliteLabelRepository::new(state.db_pool.clone());
    let attachment_repo = SqliteAttachmentRepository::new(state.db_pool.clone());

    let conversation = conversation_repo
        .find_by_id(conversation_id)
        .await
        .map_err(|e| format!("Failed to fetch conversation: {}", e))?
        .ok_or_else(|| format!("Conversation {} not found", conversation_id))?;

    let conversation_emails = email_repo
        .find_by_conversation_id(conversation_id)
        .await
        .map_err(|e| format!("Failed to fetch conversation emails: {}", e))?;

    let mut email_details = Vec::new();
    for email in conversation_emails {
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

        email_details.push(EmailDetail::from_email(&email, labels, attachments));
    }

    let all_attachments = attachment_repo
        .find_by_conversation_id(conversation_id)
        .await
        .map_err(|e| format!("Failed to fetch conversation attachments: {}", e))?
        .iter()
        .map(AttachmentInfo::from)
        .collect();

    Ok(conversation.to_detail(email_details, all_attachments))
}
