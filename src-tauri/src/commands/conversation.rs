/// Conversation/thread query commands using repository pattern and DTOs
use std::collections::{HashMap, HashSet};
use tauri::State;
use uuid::Uuid;

use crate::database::models::conversation::{ConversationDetail, ConversationListItem};
use crate::database::models::email_dto::{AttachmentInfo, EmailDetail, EmailListItem, LabelInfo};
use crate::database::repositories::{
    AttachmentRepository, ConversationRepository, EmailRepository, LabelRepository,
    SqliteAttachmentRepository, SqliteConversationRepository, SqliteEmailRepository,
    SqliteLabelRepository,
};
use crate::services::notification_service::NotificationService;
use crate::state::AppState;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeFilterConditionRequest {
    pub r#type: String,
    #[serde(default)]
    pub folder_ids: Vec<Uuid>,
    #[serde(default)]
    pub label_ids: Vec<Uuid>,
    pub operator: Option<String>,
    #[serde(default)]
    pub negated: bool,
    pub match_all_labels: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeFilterGroupRequest {
    pub id: Option<String>,
    pub operator: Option<String>,
    #[serde(default)]
    pub negated: bool,
    #[serde(default)]
    pub filters: Vec<ScopeFilterConditionRequest>,
}

fn matches_scope_condition(
    email: &crate::database::models::email::Email,
    email_label_ids: &HashSet<Uuid>,
    condition: &ScopeFilterConditionRequest,
) -> bool {
    let operator = condition
        .operator
        .clone()
        .unwrap_or_else(|| "or".to_string())
        .to_lowercase();

    let base_match = match condition.r#type.as_str() {
        "folder" => {
            if condition.folder_ids.is_empty() {
                true
            } else if operator == "and" {
                condition
                    .folder_ids
                    .iter()
                    .all(|folder_id| email.folder_id == *folder_id)
            } else {
                condition
                    .folder_ids
                    .iter()
                    .any(|folder_id| email.folder_id == *folder_id)
            }
        }
        "label" => {
            if condition.label_ids.is_empty() {
                true
            } else if condition.match_all_labels.unwrap_or(false) || operator == "and" {
                condition
                    .label_ids
                    .iter()
                    .all(|label_id| email_label_ids.contains(label_id))
            } else {
                condition
                    .label_ids
                    .iter()
                    .any(|label_id| email_label_ids.contains(label_id))
            }
        }
        _ => true,
    };

    if condition.negated {
        !base_match
    } else {
        base_match
    }
}

async fn reminder_notification_map(
    state: &State<'_, AppState>,
    email_ids: &[Uuid],
) -> Result<HashMap<Uuid, chrono::DateTime<chrono::Utc>>, String> {
    NotificationService::new(state.db_pool.clone(), state.settings.clone())
        .latest_reminder_notification_map(email_ids)
        .await
}

fn matches_scope_condition_across_emails(
    emails_with_labels: &[(crate::database::models::email::Email, HashSet<Uuid>)],
    condition: &ScopeFilterConditionRequest,
) -> bool {
    if emails_with_labels.is_empty() {
        return false;
    }

    let operator = condition
        .operator
        .clone()
        .unwrap_or_else(|| "or".to_string())
        .to_lowercase();

    let base_match = match condition.r#type.as_str() {
        "folder" => {
            if condition.folder_ids.is_empty() {
                true
            } else if operator == "and" {
                condition.folder_ids.iter().all(|folder_id| {
                    emails_with_labels
                        .iter()
                        .any(|(email, _)| email.folder_id == *folder_id)
                })
            } else {
                emails_with_labels.iter().any(|(email, _)| {
                    condition
                        .folder_ids
                        .iter()
                        .any(|folder_id| email.folder_id == *folder_id)
                })
            }
        }
        "label" => {
            if condition.label_ids.is_empty() {
                true
            } else if condition.match_all_labels.unwrap_or(false) || operator == "and" {
                condition.label_ids.iter().all(|label_id| {
                    emails_with_labels
                        .iter()
                        .any(|(_, email_label_ids)| email_label_ids.contains(label_id))
                })
            } else {
                emails_with_labels.iter().any(|(_, email_label_ids)| {
                    condition
                        .label_ids
                        .iter()
                        .any(|label_id| email_label_ids.contains(label_id))
                })
            }
        }
        _ => true,
    };

    if condition.negated {
        !base_match
    } else {
        base_match
    }
}

fn matches_scope_group_across_emails(
    emails_with_labels: &[(crate::database::models::email::Email, HashSet<Uuid>)],
    group: &ScopeFilterGroupRequest,
) -> bool {
    if group.filters.is_empty() {
        return false;
    }

    let operator = group
        .operator
        .clone()
        .unwrap_or_else(|| "and".to_string())
        .to_lowercase();

    let base_match =
        if operator == "or" {
            group.filters.iter().any(|condition| {
                matches_scope_condition_across_emails(emails_with_labels, condition)
            })
        } else {
            group.filters.iter().all(|condition| {
                matches_scope_condition_across_emails(emails_with_labels, condition)
            })
        };

    if group.negated {
        !base_match
    } else {
        base_match
    }
}

fn matches_scope_group(
    email: &crate::database::models::email::Email,
    email_label_ids: &HashSet<Uuid>,
    group: &ScopeFilterGroupRequest,
) -> bool {
    if group.filters.is_empty() {
        return false;
    }

    let operator = group
        .operator
        .clone()
        .unwrap_or_else(|| "and".to_string())
        .to_lowercase();

    let base_match = if operator == "or" {
        group
            .filters
            .iter()
            .any(|condition| matches_scope_condition(email, email_label_ids, condition))
    } else {
        group
            .filters
            .iter()
            .all(|condition| matches_scope_condition(email, email_label_ids, condition))
    };

    if group.negated {
        !base_match
    } else {
        base_match
    }
}

fn matches_root_filters(
    emails_with_labels: &[(crate::database::models::email::Email, HashSet<Uuid>)],
    groups: &[ScopeFilterGroupRequest],
) -> bool {
    if groups.is_empty() {
        return true;
    }

    groups
        .iter()
        .all(|group| matches_scope_group_across_emails(emails_with_labels, group))
}

/// Get conversations for a label with minimal email data, supporting sort/filter/pagination
#[tauri::command]
pub async fn get_conversations_for_label(
    state: State<'_, AppState>,
    label_id: Uuid,
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

    // Fetch enough raw emails to fill `limit` unique conversations.
    let emails = email_repo
        .find_by_label_with_filters(
            label_id,
            limit * 10,
            offset,
            &sort_by,
            &sort_order,
            filter_read,
            filter_has_attachments,
        )
        .await
        .map_err(|e| format!("Failed to fetch emails for label: {}", e))?;

    // Deduplicate conversation IDs while preserving the sort order from the email query.
    let mut seen = HashSet::new();
    let conversation_ids: Vec<Uuid> = emails
        .iter()
        .filter_map(|email| email.conversation_id.as_ref())
        .filter_map(|id| Uuid::parse_str(id).ok())
        .filter(|id| seen.insert(*id))
        .take(limit as usize)
        .collect();

    if conversation_ids.is_empty() {
        return Ok(Vec::new());
    }

    let conversations = conversation_repo
        .find_by_ids(conversation_ids.clone())
        .await
        .map_err(|e| format!("Failed to fetch conversations: {}", e))?;

    let mut conversation_map: HashMap<Uuid, _> = HashMap::new();
    for conversation in conversations {
        let conversation_emails = email_repo
            .find_by_conversation_id(conversation.id)
            .await
            .map_err(|e| format!("Failed to fetch conversation emails: {}", e))?;
        let conversation_email_ids: Vec<Uuid> =
            conversation_emails.iter().map(|email| email.id).collect();
        let notified_at_by_email =
            reminder_notification_map(&state, &conversation_email_ids).await?;

        let mut email_list_items = Vec::new();
        for email in conversation_emails {
            let labels = label_repo
                .find_by_email(email.id)
                .await
                .map_err(|e| format!("Failed to fetch labels: {}", e))?
                .iter()
                .map(LabelInfo::from)
                .collect();

            let mut email_list_item = EmailListItem::from_email(&email, labels);
            email_list_item.notified_at = notified_at_by_email.get(&email.id).copied();
            email_list_items.push(email_list_item);
        }

        conversation_map.insert(conversation.id, conversation.to_list_item(email_list_items));
    }

    let result = conversation_ids
        .iter()
        .filter_map(|id| conversation_map.remove(id))
        .collect();

    Ok(result)
}

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

    // Fetch enough raw emails to fill `limit` unique conversations.
    // We use limit*10 as a heuristic for heavily-threaded folders.
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

    // Deduplicate conversation IDs while preserving the sort order from the email query.
    // A HashSet tracks what we've seen; the Vec preserves insertion order.
    let mut seen = HashSet::new();
    let conversation_ids: Vec<Uuid> = emails
        .iter()
        .filter_map(|email| email.conversation_id.as_ref())
        .filter_map(|id| Uuid::parse_str(id).ok())
        .filter(|id| seen.insert(*id))
        .take(limit as usize)
        .collect();

    if conversation_ids.is_empty() {
        return Ok(Vec::new());
    }

    // Build a map of conversation_id -> list item so we can restore sort order afterwards.
    let conversations = conversation_repo
        .find_by_ids(conversation_ids.clone())
        .await
        .map_err(|e| format!("Failed to fetch conversations: {}", e))?;

    let mut conversation_map: HashMap<Uuid, _> = HashMap::new();
    for conversation in conversations {
        let conversation_emails = email_repo
            .find_by_conversation_id(conversation.id)
            .await
            .map_err(|e| format!("Failed to fetch conversation emails: {}", e))?;
        let conversation_email_ids: Vec<Uuid> =
            conversation_emails.iter().map(|email| email.id).collect();
        let notified_at_by_email =
            reminder_notification_map(&state, &conversation_email_ids).await?;

        let mut email_list_items = Vec::new();
        for email in conversation_emails {
            let labels = label_repo
                .find_by_email(email.id)
                .await
                .map_err(|e| format!("Failed to fetch labels: {}", e))?
                .iter()
                .map(LabelInfo::from)
                .collect();

            let mut email_list_item = EmailListItem::from_email(&email, labels);
            email_list_item.notified_at = notified_at_by_email.get(&email.id).copied();
            email_list_items.push(email_list_item);
        }

        conversation_map.insert(conversation.id, conversation.to_list_item(email_list_items));
    }

    // Return conversations in the original sorted order derived from the email query.
    let result = conversation_ids
        .iter()
        .filter_map(|id| conversation_map.remove(id))
        .collect();

    Ok(result)
}

/// Get conversations for a combined folder/label scope with minimal email data
#[tauri::command]
pub async fn get_conversations_for_scope(
    state: State<'_, AppState>,
    folder_ids: Vec<Uuid>,
    label_ids: Vec<Uuid>,
    match_all_labels: Option<bool>,
    filters: Option<Vec<ScopeFilterConditionRequest>>,
    filter_groups: Option<Vec<ScopeFilterGroupRequest>>,
    root_operator: Option<String>,
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
    let match_all_labels = match_all_labels.unwrap_or(false);
    let _root_operator = root_operator.unwrap_or_else(|| "and".to_string());

    let normalized_groups = if let Some(groups) = filter_groups {
        groups
            .into_iter()
            .filter(|group| !group.filters.is_empty())
            .collect::<Vec<_>>()
    } else if let Some(flat_filters) = filters {
        if flat_filters.is_empty() {
            Vec::new()
        } else {
            vec![ScopeFilterGroupRequest {
                id: None,
                operator: Some("and".to_string()),
                negated: false,
                filters: flat_filters,
            }]
        }
    } else {
        Vec::new()
    };

    if folder_ids.is_empty() && label_ids.is_empty() && normalized_groups.is_empty() {
        return Ok(Vec::new());
    }

    let mut seed_emails = Vec::new();

    if !folder_ids.is_empty() {
        for folder_id in &folder_ids {
            let mut emails = email_repo
                .find_by_folder_with_filters(
                    *folder_id,
                    limit * 20,
                    0,
                    &sort_by,
                    &sort_order,
                    filter_read,
                    filter_has_attachments,
                )
                .await
                .map_err(|e| format!("Failed to fetch emails for scope folders: {}", e))?;
            seed_emails.append(&mut emails);
        }
    }

    if !label_ids.is_empty() {
        if match_all_labels {
            let mut emails = email_repo
                .find_by_labels(&label_ids, true, limit * 20, 0)
                .await
                .map_err(|e| format!("Failed to fetch emails for scope labels: {}", e))?;
            seed_emails.append(&mut emails);
        } else {
            for label_id in &label_ids {
                let mut emails = email_repo
                    .find_by_label_with_filters(
                        *label_id,
                        limit * 20,
                        0,
                        &sort_by,
                        &sort_order,
                        filter_read,
                        filter_has_attachments,
                    )
                    .await
                    .map_err(|e| format!("Failed to fetch emails for scope labels: {}", e))?;
                seed_emails.append(&mut emails);
            }
        }
    }

    if seed_emails.is_empty() && !normalized_groups.is_empty() {
        let mut all_seed = email_repo
            .find_synced_batch(limit * 50, 0)
            .await
            .map_err(|e| format!("Failed to fetch emails for scoped filters: {}", e))?;
        seed_emails.append(&mut all_seed);
    }

    if seed_emails.is_empty() {
        return Ok(Vec::new());
    }

    let mut deduped_seed_emails = Vec::new();
    let mut seen_email_ids = HashSet::new();
    for email in seed_emails {
        if seen_email_ids.insert(email.id) {
            deduped_seed_emails.push(email);
        }
    }

    let folder_filter: HashSet<Uuid> = folder_ids.iter().copied().collect();
    let label_filter: HashSet<Uuid> = label_ids.iter().copied().collect();

    let mut conversation_order = Vec::new();
    let mut latest_sort_values: HashMap<Uuid, i64> = HashMap::new();
    let mut seen_conversation_ids = HashSet::new();

    for email in deduped_seed_emails {
        let Some(conversation_id_str) = email.conversation_id.as_ref() else {
            continue;
        };

        let Ok(conversation_id) = Uuid::parse_str(conversation_id_str) else {
            continue;
        };

        let conversation_emails = email_repo
            .find_by_conversation_id(conversation_id)
            .await
            .map_err(|e| format!("Failed to fetch scoped conversation emails: {}", e))?;

        let mut scoped_emails_with_labels = Vec::new();

        for scoped_email in conversation_emails {
            if let Some(is_read) = filter_read {
                if scoped_email.is_read != is_read {
                    continue;
                }
            }

            if let Some(has_attachments) = filter_has_attachments {
                if scoped_email.has_attachments != has_attachments {
                    continue;
                }
            }

            let email_labels = label_repo
                .find_by_email(scoped_email.id)
                .await
                .map_err(|e| format!("Failed to fetch labels for scoped email: {}", e))?;

            let email_label_ids: HashSet<Uuid> =
                email_labels.iter().map(|label| label.id).collect();

            scoped_emails_with_labels.push((scoped_email, email_label_ids));
        }

        let mut conversation_matches = false;
        let mut best_sort_value: Option<i64> = None;

        let advanced_matches = matches_root_filters(&scoped_emails_with_labels, &normalized_groups);
        let has_flat_filters = !folder_filter.is_empty() || !label_filter.is_empty();

        for (scoped_email, email_label_ids) in &scoped_emails_with_labels {
            let flat_folder_matches =
                folder_filter.is_empty() || folder_filter.contains(&scoped_email.folder_id);

            let flat_label_matches = if label_filter.is_empty() {
                true
            } else if match_all_labels {
                label_filter
                    .iter()
                    .all(|label_id| email_label_ids.contains(label_id))
            } else {
                label_filter
                    .iter()
                    .any(|label_id| email_label_ids.contains(label_id))
            };

            let flat_matches = flat_folder_matches && flat_label_matches;

            let matches_scope = if has_flat_filters && !normalized_groups.is_empty() {
                flat_matches && advanced_matches
            } else if has_flat_filters {
                flat_matches
            } else {
                advanced_matches
            };

            if !matches_scope {
                continue;
            }

            conversation_matches = true;

            let sort_value = match sort_by.as_str() {
                "sent_at" => scoped_email
                    .sent_at
                    .map(|dt| dt.timestamp_millis())
                    .unwrap_or_default(),
                "size" => scoped_email.size,
                _ => scoped_email.received_at.timestamp_millis(),
            };

            match best_sort_value {
                Some(current) => {
                    let should_replace = if sort_order.eq_ignore_ascii_case("asc") {
                        sort_value < current
                    } else {
                        sort_value > current
                    };
                    if should_replace {
                        best_sort_value = Some(sort_value);
                    }
                }
                None => best_sort_value = Some(sort_value),
            }
        }

        if !conversation_matches {
            continue;
        }

        let sort_value = best_sort_value.unwrap_or_default();
        latest_sort_values.insert(conversation_id, sort_value);

        if seen_conversation_ids.insert(conversation_id) {
            conversation_order.push(conversation_id);
        }
    }

    if latest_sort_values.is_empty() {
        return Ok(Vec::new());
    }

    conversation_order.sort_by_key(|conversation_id| {
        latest_sort_values
            .get(conversation_id)
            .copied()
            .unwrap_or_default()
    });

    if sort_order != "asc" {
        conversation_order.reverse();
    }

    let paged_conversation_ids: Vec<Uuid> = conversation_order
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();

    if paged_conversation_ids.is_empty() {
        return Ok(Vec::new());
    }

    let conversations = conversation_repo
        .find_by_ids(paged_conversation_ids.clone())
        .await
        .map_err(|e| format!("Failed to fetch scoped conversations: {}", e))?;

    let mut conversation_map: HashMap<Uuid, _> = HashMap::new();
    for conversation in conversations {
        let conversation_emails = email_repo
            .find_by_conversation_id(conversation.id)
            .await
            .map_err(|e| format!("Failed to fetch scoped conversation emails: {}", e))?;
        let conversation_email_ids: Vec<Uuid> =
            conversation_emails.iter().map(|email| email.id).collect();
        let notified_at_by_email =
            reminder_notification_map(&state, &conversation_email_ids).await?;

        let mut email_list_items = Vec::new();
        for email in conversation_emails {
            let labels = label_repo
                .find_by_email(email.id)
                .await
                .map_err(|e| format!("Failed to fetch labels: {}", e))?
                .iter()
                .map(LabelInfo::from)
                .collect();

            let mut email_list_item = EmailListItem::from_email(&email, labels);
            email_list_item.notified_at = notified_at_by_email.get(&email.id).copied();
            email_list_items.push(email_list_item);
        }

        conversation_map.insert(conversation.id, conversation.to_list_item(email_list_items));
    }

    let result = paged_conversation_ids
        .iter()
        .filter_map(|id| conversation_map.remove(id))
        .collect();

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
    let conversation_email_ids: Vec<Uuid> =
        conversation_emails.iter().map(|item| item.id).collect();
    let notified_at_by_email = reminder_notification_map(&state, &conversation_email_ids).await?;

    let mut email_list_items = Vec::new();
    for email in conversation_emails {
        let labels = label_repo
            .find_by_email(email.id)
            .await
            .map_err(|e| format!("Failed to fetch labels: {}", e))?
            .iter()
            .map(LabelInfo::from)
            .collect();

        let mut email_list_item = EmailListItem::from_email(&email, labels);
        email_list_item.notified_at = notified_at_by_email.get(&email.id).copied();
        email_list_items.push(email_list_item);
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
    let conversation_email_ids: Vec<Uuid> =
        conversation_emails.iter().map(|item| item.id).collect();
    let notified_at_by_email = reminder_notification_map(&state, &conversation_email_ids).await?;

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

        let mut email_detail = EmailDetail::from_email(&email, labels, attachments);
        email_detail.notified_at = notified_at_by_email.get(&email.id).copied();
        email_details.push(email_detail);
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
