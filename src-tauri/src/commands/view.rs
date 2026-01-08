use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::{
    database::{
        models::view::{SwimlaneState, View, ViewConfig, ViewType},
        repositories::{RepositoryFactory, ViewRepository},
    },
    state::AppState,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateViewRequest {
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub view_type: ViewType,
    #[serde(default)]
    pub config: ViewConfig,
    #[serde(default)]
    pub folders: Vec<String>,
    #[serde(default)]
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateViewRequest {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub view_type: ViewType,
    pub config: ViewConfig,
    pub folders: Vec<String>,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSwimlaneRequest {
    pub view_id: String,
    pub title: String,
    pub color: Option<String>,
    pub label_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder_ids: Option<Vec<String>>,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSwimlaneRequest {
    pub view_id: String,
    pub swimlane_id: String,
    pub title: String,
    pub color: Option<String>,
    pub label_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder_ids: Option<Vec<String>>,
    pub state: SwimlaneState,
    pub sort_order: i32,
}

#[tauri::command]
pub async fn get_views(state: State<'_, AppState>) -> Result<Vec<View>, String> {
    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let view_repo = repo_factory.view_repository();

    view_repo
        .get_all()
        .await
        .map_err(|e| format!("Failed to get views: {}", e))
}

#[tauri::command]
pub async fn get_view(state: State<'_, AppState>, view_id: String) -> Result<Option<View>, String> {
    let id = Uuid::parse_str(&view_id).map_err(|e| format!("Invalid view ID: {}", e))?;

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let view_repo = repo_factory.view_repository();

    view_repo
        .find_by_id(id)
        .await
        .map_err(|e| format!("Failed to get view: {}", e))
}

#[tauri::command]
pub async fn create_view(
    state: State<'_, AppState>,
    request: CreateViewRequest,
) -> Result<View, String> {
    let folder_ids: Result<Vec<Uuid>, _> =
        request.folders.iter().map(|f| Uuid::parse_str(f)).collect();

    let folder_ids = folder_ids.map_err(|e| format!("Invalid folder ID: {}", e))?;

    let view = View {
        id: Uuid::now_v7(),
        name: request.name,
        view_type: request.view_type,
        config: request.config,
        icon: request.icon,
        color: request.color,
        folders: folder_ids,
        sort_order: request.sort_order,
        is_default: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let view_repo = repo_factory.view_repository();

    view_repo
        .create(&view)
        .await
        .map_err(|e| format!("Failed to create view: {}", e))?;

    Ok(view)
}

#[tauri::command]
pub async fn update_view(
    state: State<'_, AppState>,
    request: UpdateViewRequest,
) -> Result<(), String> {
    let id = Uuid::parse_str(&request.id).map_err(|e| format!("Invalid view ID: {}", e))?;

    let folder_ids: Result<Vec<Uuid>, _> =
        request.folders.iter().map(|f| Uuid::parse_str(f)).collect();

    log::debug!("folder_ids: {:?}", folder_ids);

    let folder_ids = folder_ids.map_err(|e| format!("Invalid folder ID: {}", e))?;

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let view_repo = repo_factory.view_repository();

    let existing = view_repo
        .find_by_id(id)
        .await
        .map_err(|e| format!("Failed to find view: {}", e))?
        .ok_or_else(|| format!("View {} not found", request.id))?;

    let updated_view = View {
        id,
        name: request.name,
        view_type: request.view_type,
        config: request.config,
        icon: request.icon,
        color: request.color,
        folders: folder_ids,
        sort_order: request.sort_order,
        is_default: existing.is_default,
        created_at: existing.created_at,
        updated_at: Utc::now(),
    };

    view_repo
        .update(&updated_view)
        .await
        .map_err(|e| format!("Failed to update view: {}", e))
}

#[tauri::command]
pub async fn delete_view(state: State<'_, AppState>, view_id: String) -> Result<(), String> {
    let id = Uuid::parse_str(&view_id).map_err(|e| format!("Invalid view ID: {}", e))?;

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let view_repo = repo_factory.view_repository();

    view_repo
        .delete(id)
        .await
        .map_err(|e| format!("Failed to delete view: {}", e))
}
