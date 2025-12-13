use serde::{Deserialize, Serialize};
use tauri::{Manager, State, WebviewWindowBuilder};
use uuid::Uuid;

use crate::database::models::account::{Account, AccountType};
use crate::database::repositories::{AccountRepository, FolderRepository, RepositoryFactory};
use crate::state::AppState;
use crate::sync::{
    auth::OAuth2Helper,
    types::{AccountSettings, ImapCredentials, SyncFolder},
};

#[derive(Debug, Serialize)]
pub struct SyncReport {
    pub folders_synced: usize,
    pub emails_synced: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct StartOAuth2Request {
    pub provider: String,
    pub redirect_uri: String,
}

#[derive(Debug, Serialize)]
pub struct StartOAuth2Response {
    pub auth_url: String,
    pub csrf_token: String,
}

#[tauri::command]
pub async fn start_oauth2_flow(
    state: State<'_, AppState>,
    request: StartOAuth2Request,
    account_id: Uuid,
) -> Result<StartOAuth2Response, String> {
    let (auth_url, csrf_token, pkce_verifier) =
        OAuth2Helper::start_oauth2_flow(&request.provider, &request.redirect_uri)
            .map_err(|e| e.to_string())?;

    let oauth_state = crate::sync::oauth_state::OAuthState {
        csrf_token: csrf_token.clone(),
        pkce_verifier,
        provider: request.provider.clone(),
        account_id,
        redirect_uri: request.redirect_uri.clone(),
        created_at: chrono::Utc::now(),
    };

    state
        .oauth_state_manager
        .store(csrf_token.clone(), oauth_state)
        .await
        .map_err(|e| e.to_string())?;

    Ok(StartOAuth2Response {
        auth_url,
        csrf_token,
    })
}

/// Open OAuth window for authentication
#[tauri::command]
pub async fn open_oauth_window(
    app_handle: tauri::AppHandle,
    auth_url: String,
    provider: String,
) -> Result<(), String> {
    let window_label = format!("oauth-{}-{}", provider, chrono::Utc::now().timestamp());

    if let Some(existing_window) = app_handle.get_webview_window(&format!("oauth-{}", provider)) {
        let _ = existing_window.close();
    }

    WebviewWindowBuilder::new(
        &app_handle,
        window_label,
        tauri::WebviewUrl::External(
            auth_url
                .parse()
                .map_err(|e: url::ParseError| e.to_string())?,
        ),
    )
    .title(format!("{} Authentication", provider))
    .inner_size(600.0, 700.0)
    .resizable(true)
    .center()
    .focused(true)
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Close OAuth window
#[tauri::command]
pub async fn close_oauth_window(
    app_handle: tauri::AppHandle,
    window_label: String,
) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window(&window_label) {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct ExchangeOAuth2CodeRequest {
    pub provider: String,
    pub code: String,
    pub redirect_uri: String,
    pub csrf_token: String,
    pub account_id: Uuid,
}

#[tauri::command]
pub async fn exchange_oauth2_code(
    state: State<'_, AppState>,
    code: String,
    csrf_token: String,
) -> Result<String, String> {
    let oauth_state = state
        .oauth_state_manager
        .get_and_remove(&csrf_token)
        .await
        .map_err(|e| e.to_string())?;

    let credentials = OAuth2Helper::exchange_code(
        &oauth_state.provider,
        &code,
        &oauth_state.redirect_uri,
        &oauth_state.pkce_verifier,
    )
    .await
    .map_err(|e| e.to_string())?;

    state
        .credential_store
        .store_oauth2(oauth_state.account_id, &credentials)
        .await
        .map_err(|e| e.to_string())?;

    log::info!(
        "OAuth2 authentication successful for account {}",
        oauth_state.account_id
    );

    if let Err(e) = state
        .background_sync_manager
        .start_account_sync(&oauth_state.account_id)
        .await
    {
        log::warn!(
            "Failed to start background sync for account {}: {}",
            oauth_state.account_id,
            e
        );
        // Don't fail the whole operation if sync start fails
    }

    Ok("OAuth2 authentication successful".to_string())
}

#[derive(Debug, Deserialize)]
pub struct StoreImapCredentialsRequest {
    pub account_id: Uuid,
    pub username: String,
    pub password: String,
}

#[tauri::command]
pub async fn store_imap_credentials(
    state: State<'_, AppState>,
    request: StoreImapCredentialsRequest,
) -> Result<String, String> {
    let credentials = ImapCredentials {
        username: request.username,
        password: request.password,
    };

    state
        .credential_store
        .store_imap(request.account_id, &credentials)
        .await
        .map_err(|e| e.to_string())?;

    log::info!(
        "IMAP credentials stored successfully for account {}",
        request.account_id
    );

    if let Err(e) = state
        .background_sync_manager
        .start_account_sync(&request.account_id)
        .await
    {
        log::warn!(
            "Failed to start background sync for account {}: {}",
            request.account_id,
            e
        );
    }

    Ok("IMAP credentials stored successfully".to_string())
}

#[tauri::command]
pub async fn sync_account(
    state: State<'_, AppState>,
    account_id: Uuid,
) -> Result<SyncReport, String> {
    let report = state
        .sync_coordinator
        .sync_account(account_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(SyncReport {
        folders_synced: report.folders_synced,
        emails_synced: report.emails_synced,
        errors: report.errors,
    })
}

#[tauri::command]
pub async fn sync_folder(
    state: State<'_, AppState>,
    account_id: Uuid,
    folder_id: Uuid,
    full: Option<bool>,
) -> Result<usize, String> {
    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let folder_repo = repo_factory.folder_repository();

    let folder_model = folder_repo
        .find_by_id(folder_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Folder {} not found", folder_id))?;

    let folder = SyncFolder {
        id: Some(folder_model.id),
        account_id: folder_model.account_id,
        name: folder_model.name,
        folder_type: folder_model.folder_type,
        remote_id: folder_model.remote_id.unwrap_or_default(),
        parent_id: folder_model.parent_id,
        icon: folder_model.icon,
        color: folder_model.color,
        synced_at: Some(folder_model.synced_at),
        sync_interval: folder_model.sync_interval,
        attributes: Vec::new(),
        unread_count: folder_model.unread_count as i32,
        total_count: folder_model.total_count as i32,
        expanded: folder_model.expanded,
        hidden: folder_model.hidden,
    };

    let count = state
        .sync_coordinator
        .sync_folder(account_id, &folder, full.unwrap_or(false))
        .await
        .map_err(|e| e.to_string())?;

    Ok(count)
}

#[tauri::command]
pub async fn set_flag(
    state: State<'_, AppState>,
    account_id: Uuid,
    email_id: Uuid,
    flagged: bool,
) -> Result<String, String> {
    state
        .sync_coordinator
        .set_flag(account_id, email_id, flagged)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!(
        "Email {}",
        if flagged { "flagged" } else { "unflagged" }
    ))
}

#[tauri::command]
pub async fn open_add_account_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    if app_handle.get_webview_window("add-account").is_some() {
        return Ok(());
    }

    WebviewWindowBuilder::new(
        &app_handle,
        "add-account",
        tauri::WebviewUrl::App("/auth/add-account".into()),
    )
    .title("Add Email Account")
    .inner_size(800.0, 700.0)
    .resizable(false)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub name: String,
    pub email: String,
    pub account_type: String,
    pub settings: Option<serde_json::Value>,
}

#[tauri::command]
pub async fn create_account(
    state: State<'_, AppState>,
    request: CreateAccountRequest,
) -> Result<Account, String> {
    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let account_repo = repo_factory.account_repository();

    let settings = if let Some(settings) = request.settings {
        settings
    } else {
        serde_json::to_value(AccountSettings::default())
            .map_err(|e| format!("Failed to serialize default settings: {}", e))?
    };

    let account = Account {
        id: Uuid::now_v7(),
        name: request.name,
        email: request.email,
        account_type: AccountType::from(request.account_type),
        settings,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let _ = account_repo
        .create(&account)
        .await
        .map_err(|e| e.to_string())?;

    Ok(account)
}

#[tauri::command]
pub async fn get_accounts(state: State<'_, AppState>) -> Result<Vec<Account>, String> {
    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let account_repo = repo_factory.account_repository();

    let accounts = account_repo.find_all().await.map_err(|e| e.to_string())?;

    Ok(accounts)
}

#[tauri::command]
pub async fn delete_account(
    state: State<'_, AppState>,
    account_id: Uuid,
) -> Result<String, String> {
    let _ = state.credential_store.delete(account_id).await;

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let account_repo = repo_factory.account_repository();

    account_repo
        .delete(account_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok("Account deleted successfully".to_string())
}

#[tauri::command]
pub async fn start_background_sync(
    state: State<'_, AppState>,
    account_id: Uuid,
) -> Result<String, String> {
    state
        .background_sync_manager
        .start_account_sync(&account_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!(
        "Background sync started for account {}",
        account_id
    ))
}

#[tauri::command]
pub async fn stop_background_sync(
    state: State<'_, AppState>,
    account_id: Uuid,
) -> Result<String, String> {
    state
        .background_sync_manager
        .stop_account_sync(&account_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!(
        "Background sync stopped for account {}",
        account_id
    ))
}

#[tauri::command]
pub async fn get_sync_status(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let active_syncs = state.background_sync_manager.get_active_syncs().await;
    Ok(active_syncs.into_iter().map(|id| id.to_string()).collect())
}

#[tauri::command]
pub async fn is_account_syncing(
    state: State<'_, AppState>,
    account_id: Uuid,
) -> Result<bool, String> {
    let is_syncing = state.background_sync_manager.is_syncing(&account_id).await;
    Ok(is_syncing)
}

#[derive(Debug, Deserialize)]
pub struct MoveFolderRequest {
    pub account_id: Uuid,
    pub folder_id: Uuid,
    pub new_parent_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFolderSettingsRequest {
    pub folder_id: Uuid,
    pub settings: crate::database::models::folder::FolderSettings,
}

#[tauri::command]
pub async fn update_folder_settings(
    state: State<'_, AppState>,
    request: UpdateFolderSettingsRequest,
) -> Result<(), String> {
    use crate::database::repositories::{FolderRepository, SqliteFolderRepository};

    let folder_repo = SqliteFolderRepository::new(state.db_pool.clone());

    let mut folder = folder_repo
        .find_by_id(request.folder_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Folder {} not found", request.folder_id))?;

    folder.settings = request.settings;

    folder_repo
        .update(&folder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct UndeleteEmailsRequest {
    pub account_id: Uuid,
}
