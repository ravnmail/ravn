use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};
use uuid::Uuid;

use crate::database::models::folder::{Folder, FolderSettings, FolderType};
use crate::database::repositories::{FolderRepository, SqliteFolderRepository};
use crate::state::AppState;
use crate::sync::SyncFolder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderResponse {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub folder_type: FolderType,
    pub remote_id: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub expanded: bool,
    pub hidden: bool,
    pub parent_id: Option<Uuid>,
    pub settings: FolderSettings,
    pub unread_count: i64,
    pub total_count: i64,
}

impl From<Folder> for FolderResponse {
    fn from(folder: Folder) -> Self {
        FolderResponse {
            id: folder.id,
            account_id: folder.account_id,
            name: folder.name,
            folder_type: folder.folder_type,
            remote_id: folder.remote_id,
            color: folder.color,
            icon: folder.icon,
            sort_order: folder.sort_order,
            expanded: folder.expanded,
            hidden: folder.hidden,
            parent_id: folder.parent_id,
            settings: folder.settings,
            unread_count: folder.unread_count,
            total_count: folder.total_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameRequest {
    pub name: String,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSettingsRequest {
    pub folder_id: Uuid,
    pub settings: FolderSettings,
}

fn emit_folder_event<S: serde::Serialize + Clone>(
    app_handle: &tauri::AppHandle,
    event_name: &str,
    payload: S,
) {
    if let Err(e) = app_handle.emit(event_name, payload) {
        log::error!("Failed to emit folder event '{}': {}", event_name, e);
    }
}

#[tauri::command]
pub async fn get_folders(
    state: State<'_, AppState>,
    account_id: Uuid,
) -> Result<Vec<FolderResponse>, String> {
    log::info!("Getting all folders for account {}", account_id);

    let folder_repo = SqliteFolderRepository::new(state.db_pool.clone());

    let folders = folder_repo
        .find_by_account(account_id)
        .await
        .map_err(|e| format!("Failed to fetch folders: {}", e))?;

    let responses = folders.into_iter().map(FolderResponse::from).collect();

    Ok(responses)
}

#[tauri::command]
pub async fn init_folder_sync(
    state: State<'_, AppState>,
    folder_id: Uuid,
    full: bool,
) -> Result<String, String> {
    log::info!("Initializing folder sync for folder {}", folder_id);

    let folder_repo = SqliteFolderRepository::new(state.db_pool.clone());

    let folder_model = folder_repo
        .find_by_id(folder_id)
        .await
        .map_err(|e| format!("Failed to fetch folder: {}", e))?
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

    let _ = state
        .sync_coordinator
        .sync_folder(folder_model.account_id, &folder, full)
        .await
        .map_err(|e| format!("Failed to initiate folder sync: {}", e))?;

    emit_folder_event(
        &state.app_handle,
        "folder:sync_started",
        serde_json::json!({
            "folder_id": folder_id.to_string()
        }),
    );

    Ok(format!("Folder {} sync initialized", folder_id))
}

#[tauri::command]
pub async fn update_expanded(
    state: State<'_, AppState>,
    folder_id: Uuid,
    is_expanded: bool,
) -> Result<(), String> {
    log::info!("Updating expanded state for folder {}", folder_id);

    let folder_repo = SqliteFolderRepository::new(state.db_pool.clone());

    let mut folder = folder_repo
        .find_by_id(folder_id)
        .await
        .map_err(|e| format!("Failed to fetch folder: {}", e))?
        .ok_or_else(|| format!("Folder {} not found", folder_id))?;

    folder.expanded = is_expanded;

    folder_repo
        .update(&folder)
        .await
        .map_err(|e| format!("Failed to update folder: {}", e))?;

    emit_folder_event(
        &state.app_handle,
        "folder:updated",
        serde_json::json!(folder),
    );

    Ok(())
}

#[tauri::command]
pub async fn update_hidden(
    state: State<'_, AppState>,
    folder_id: Uuid,
    is_hidden: bool,
) -> Result<(), String> {
    log::info!("Updating hidden state for folder {}", folder_id);

    let folder_repo = SqliteFolderRepository::new(state.db_pool.clone());

    let mut folder = folder_repo
        .find_by_id(folder_id)
        .await
        .map_err(|e| format!("Failed to fetch folder: {}", e))?
        .ok_or_else(|| format!("Folder {} not found", folder_id))?;

    folder.hidden = is_hidden;

    folder_repo
        .update(&folder)
        .await
        .map_err(|e| format!("Failed to update folder: {}", e))?;

    emit_folder_event(
        &state.app_handle,
        "folder:updated",
        serde_json::json!(folder),
    );

    Ok(())
}

#[tauri::command]
pub async fn rename(
    state: State<'_, AppState>,
    folder_id: Uuid,
    request: RenameRequest,
) -> Result<(), String> {
    log::info!("Renaming folder {}", folder_id);

    let folder_repo = SqliteFolderRepository::new(state.db_pool.clone());

    let mut folder = folder_repo
        .find_by_id(folder_id)
        .await
        .map_err(|e| format!("Failed to fetch folder: {}", e))?
        .ok_or_else(|| format!("Folder {} not found", folder_id))?;

    folder.name = request.name.clone();
    if let Some(color) = request.color {
        folder.color = Some(color);
    }
    if let Some(icon) = request.icon {
        folder.icon = Some(icon);
    }

    folder_repo
        .update(&folder)
        .await
        .map_err(|e| format!("Failed to update folder: {}", e))?;

    emit_folder_event(
        &state.app_handle,
        "folder:updated",
        serde_json::json!(folder),
    );

    Ok(())
}

#[tauri::command]
pub async fn update_settings(
    state: State<'_, AppState>,
    folder_id: Uuid,
    settings: FolderSettings,
) -> Result<(), String> {
    log::info!("Updating settings for folder {}", folder_id);

    let folder_repo = SqliteFolderRepository::new(state.db_pool.clone());

    let mut folder = folder_repo
        .find_by_id(folder_id)
        .await
        .map_err(|e| format!("Failed to fetch folder: {}", e))?
        .ok_or_else(|| format!("Folder {} not found", folder_id))?;

    folder.settings = settings.clone();

    folder_repo
        .update(&folder)
        .await
        .map_err(|e| format!("Failed to update folder: {}", e))?;

    emit_folder_event(
        &state.app_handle,
        "folder:updated",
        serde_json::json!(folder),
    );

    Ok(())
}
