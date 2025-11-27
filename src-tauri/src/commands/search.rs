use crate::database::models::conversation::ConversationListItem;
use crate::database::models::email_dto::{EmailListItem, LabelInfo};
use crate::database::repositories::RepositoryFactory;
use crate::database::repositories::{EmailRepository, LabelRepository};
use crate::search::SearchQuery;
use crate::state::AppState;
use tauri::State;
use uuid::Uuid;

/// Search emails using full-text search with Tantivy
#[tauri::command]
pub async fn search_emails(
    state: State<'_, AppState>,
    query: String,
    account_id: Option<Uuid>,
    folder_id: Option<Uuid>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<SearchResults, String> {
    let search_query = SearchQuery {
        query,
        account_id,
        folder_id,
        conversation_id: None,
        limit: limit.unwrap_or(50),
        offset: offset.unwrap_or(0),
    };

    let search_results = state
        .search_manager
        .search(search_query)
        .await
        .map_err(|e| format!("Search failed: {}", e))?;

    let email_ids: Vec<Uuid> = search_results.iter().map(|r| r.id).collect();

    if email_ids.is_empty() {
        return Ok(SearchResults {
            emails: vec![],
            conversations: vec![],
            total: 0,
        });
    }

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let email_repo = repo_factory.email_repository();
    let label_repo = repo_factory.label_repository();

    let mut emails = Vec::new();
    for email_id in &email_ids {
        if let Ok(Some(email)) = email_repo.find_by_id(*email_id).await {
            let labels = label_repo
                .find_by_email(*email_id)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|label| LabelInfo {
                    id: label.id,
                    name: label.name,
                    color: label.color,
                    icon: label.icon,
                })
                .collect();

            let email_list_item = EmailListItem {
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
                received_at: email.received_at.to_rfc3339().parse().unwrap(),
                sent_at: email.sent_at.map(|dt| dt.to_rfc3339().parse().unwrap()),
                is_read: email.is_read,
                is_draft: email.is_draft,
                is_flagged: email.is_flagged,
                size: email.size,
                sync_status: email.sync_status.clone(),
                has_attachments: email.has_attachments,
                labels,
            };

            emails.push(email_list_item);
        }
    }

    let mut conversations: Vec<ConversationListItem> = Vec::new();
    let mut conversation_map: std::collections::HashMap<String, Vec<EmailListItem>> =
        std::collections::HashMap::new();

    for email in emails.iter() {
        if let Some(conv_id) = &email.conversation_id {
            conversation_map
                .entry(conv_id.clone())
                .or_insert_with(Vec::new)
                .push(email.clone());
        }
    }

    for (conv_id, messages) in conversation_map {
        conversations.push(ConversationListItem {
            id: conv_id,
            message_count: messages.len() as i64,
            ai_cache: None,
            messages,
        });
    }

    Ok(SearchResults {
        emails,
        conversations,
        total: search_results.len(),
    })
}

/// Reindex all emails in the search index
#[tauri::command]
pub async fn reindex_all_emails(state: State<'_, AppState>) -> Result<ReindexResult, String> {
    log::info!("[Search] Starting full reindex of all emails");

    state
        .search_manager
        .clear_index()
        .await
        .map_err(|e| format!("Failed to clear index: {}", e))?;

    let batch_size = 1000;
    let mut offset = 0;
    let mut total_indexed = 0;

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let email_repo = repo_factory.email_repository();

    loop {
        let emails = email_repo
            .find_synced_batch(batch_size, offset)
            .await
            .map_err(|e| format!("Failed to fetch emails: {}", e))?;

        if emails.is_empty() {
            break;
        }

        let count = emails.len();

        state
            .search_manager
            .index_emails_batch(&emails)
            .await
            .map_err(|e| format!("Failed to index batch: {}", e))?;

        total_indexed += count;
        offset += batch_size;

        log::info!(
            "[Search] Indexed {} emails (total: {})",
            count,
            total_indexed
        );
    }

    state
        .search_manager
        .commit()
        .await
        .map_err(|e| format!("Failed to commit index: {}", e))?;

    log::info!(
        "[Search] Reindex complete. Total emails indexed: {}",
        total_indexed
    );

    Ok(ReindexResult {
        total_indexed,
        success: true,
    })
}

/// Reindex emails for a specific account
#[tauri::command]
pub async fn reindex_account_emails(
    state: State<'_, AppState>,
    account_id: Uuid,
) -> Result<ReindexResult, String> {
    log::info!("[Search] Reindexing emails for account {}", account_id);

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let email_repo = repo_factory.email_repository();

    let emails = email_repo
        .find_synced_by_account(account_id)
        .await
        .map_err(|e| format!("Failed to fetch emails: {}", e))?;

    let total = emails.len();

    state
        .search_manager
        .index_emails_batch(&emails)
        .await
        .map_err(|e| format!("Failed to index emails: {}", e))?;

    state
        .search_manager
        .commit()
        .await
        .map_err(|e| format!("Failed to commit index: {}", e))?;

    log::info!(
        "[Search] Reindexed {} emails for account {}",
        total,
        account_id
    );

    Ok(ReindexResult {
        total_indexed: total,
        success: true,
    })
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SearchResults {
    pub emails: Vec<EmailListItem>,
    pub conversations: Vec<ConversationListItem>,
    pub total: usize,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ReindexResult {
    pub total_indexed: usize,
    pub success: bool,
}
