use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::database::models::contact::{Contact, ContactSummary};
use crate::database::models::folder::FolderType;
use crate::database::repositories::{ContactRepository, EmailRepository, RepositoryFactory};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchContactsRequest {
    pub account_id: Uuid,
    pub query: String,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTopContactsRequest {
    pub account_id: Uuid,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetContactsRequest {
    pub account_id: Uuid,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[tauri::command]
pub async fn search_contacts(
    state: State<'_, AppState>,
    request: SearchContactsRequest,
) -> Result<Vec<ContactSummary>, String> {
    log::debug!(
        "Searching contacts for account {} with query: {}",
        request.account_id,
        request.query
    );

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let contact_repo = repo_factory.contact_repository();

    contact_repo
        .search_contacts(
            request.account_id,
            &request.query,
            request.limit.unwrap_or(20),
        )
        .await
        .map_err(|e| format!("Failed to search contacts: {}", e))
}

#[tauri::command]
pub async fn get_top_contacts(
    state: State<'_, AppState>,
    request: GetTopContactsRequest,
) -> Result<Vec<ContactSummary>, String> {
    log::debug!("Getting top contacts for account {}", request.account_id);

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let contact_repo = repo_factory.contact_repository();

    contact_repo
        .get_top_contacts(request.account_id, request.limit.unwrap_or(10))
        .await
        .map_err(|e| format!("Failed to get top contacts: {}", e))
}

#[tauri::command]
pub async fn get_contacts(
    state: State<'_, AppState>,
    request: GetContactsRequest,
) -> Result<Vec<Contact>, String> {
    log::debug!("Getting contacts for account {}", request.account_id);

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let contact_repo = repo_factory.contact_repository();

    contact_repo
        .find_all(
            request.account_id,
            request.limit.unwrap_or(50),
            request.offset.unwrap_or(0),
        )
        .await
        .map_err(|e| format!("Failed to get contacts: {}", e))
}

#[tauri::command]
pub async fn get_contact_by_id(
    state: State<'_, AppState>,
    contact_id: Uuid,
) -> Result<Option<Contact>, String> {
    log::debug!("Getting contact by id: {}", contact_id);

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let contact_repo = repo_factory.contact_repository();

    contact_repo
        .find_by_id(contact_id)
        .await
        .map_err(|e| format!("Failed to get contact: {}", e))
}

#[tauri::command]
pub async fn get_contact_by_email(
    state: State<'_, AppState>,
    email: String,
) -> Result<Option<Contact>, String> {
    log::debug!("Getting contact by email: {}", email);

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let contact_repo = repo_factory.contact_repository();

    contact_repo
        .find_by_email(&email)
        .await
        .map_err(|e| format!("Failed to get contact by email: {}", e))
}

#[tauri::command]
pub async fn create_contact(state: State<'_, AppState>, contact: Contact) -> Result<Uuid, String> {
    log::debug!("Creating contact: {:?}", contact);

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let contact_repo = repo_factory.contact_repository();

    contact_repo
        .create(&contact)
        .await
        .map_err(|e| format!("Failed to create contact: {}", e))
}

#[tauri::command]
pub async fn update_contact(state: State<'_, AppState>, contact: Contact) -> Result<(), String> {
    log::debug!("Updating contact: {:?}", contact);

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let contact_repo = repo_factory.contact_repository();

    contact_repo
        .update(&contact)
        .await
        .map_err(|e| format!("Failed to update contact: {}", e))
}

#[tauri::command]
pub async fn delete_contact(state: State<'_, AppState>, contact_id: Uuid) -> Result<(), String> {
    log::debug!("Deleting contact: {}", contact_id);

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let contact_repo = repo_factory.contact_repository();

    contact_repo
        .delete(contact_id)
        .await
        .map_err(|e| format!("Failed to delete contact: {}", e))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResyncContactCountersRequest {
    pub account_id: Uuid,
}

#[tauri::command]
pub async fn resync_contact_counters(
    state: State<'_, AppState>,
    request: ResyncContactCountersRequest,
) -> Result<String, String> {
    log::info!(
        "Resyncing contact counters for account {}",
        request.account_id
    );

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let contact_repo = repo_factory.contact_repository();
    let email_repo = repo_factory.email_repository();

    // Step 1: Reset all counters to 0 for this account
    contact_repo
        .reset_counters_by_account(request.account_id)
        .await
        .map_err(|e| format!("Failed to reset contact counters: {}", e))?;

    log::debug!(
        "Reset all contact counters for account {}",
        request.account_id
    );

    // Step 2: Fetch all emails for this account with folder information
    let emails = email_repo
        .find_with_folder_type(request.account_id)
        .await
        .map_err(|e| format!("Failed to fetch emails: {}", e))?;

    log::debug!(
        "Found {} emails to process for account {}",
        emails.len(),
        request.account_id
    );

    let mut sent_count = 0;
    let mut received_count = 0;

    // Step 3: Iterate through emails and update counters
    for (email, folder_type) in &emails {
        let sent_at = email.sent_at;

        if *folder_type == FolderType::Sent {
            // For sent emails, increment send_count for all recipients
            let to = &email.to.0;
            let cc = &email.cc.0;
            let bcc = &email.bcc.0;

            for addr in to.iter().chain(cc.iter()).chain(bcc.iter()) {
                contact_repo
                    .increment_send_count(&addr.address, addr.name.as_deref(), sent_at)
                    .await
                    .map_err(|e| format!("Failed to increment send count: {}", e))?;
                sent_count += 1;
            }
        } else {
            // For received emails, increment receive_count for sender
            let from = &email.from.0;

            contact_repo
                .increment_receive_count(&from.address, from.name.as_deref())
                .await
                .map_err(|e| format!("Failed to increment receive count: {}", e))?;
            received_count += 1;
        }
    }

    let message = format!(
        "Resync complete: processed {} emails ({} sent, {} received)",
        emails.len(),
        sent_count,
        received_count
    );
    log::info!("{}", message);

    Ok(message)
}
