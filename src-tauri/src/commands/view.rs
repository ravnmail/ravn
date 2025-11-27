use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::{
    database::{
        models::view::{KanbanSwimlane, SwimlaneState, View, ViewConfig, ViewType},
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

#[tauri::command]
pub async fn set_default_view(state: State<'_, AppState>, view_id: String) -> Result<(), String> {
    let id = Uuid::parse_str(&view_id).map_err(|e| format!("Invalid view ID: {}", e))?;

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let view_repo = repo_factory.view_repository();

    view_repo
        .set_default(id)
        .await
        .map_err(|e| format!("Failed to set default view: {}", e))
}

#[tauri::command]
pub async fn add_swimlane(
    state: State<'_, AppState>,
    request: CreateSwimlaneRequest,
) -> Result<View, String> {
    let view_id =
        Uuid::parse_str(&request.view_id).map_err(|e| format!("Invalid view ID: {}", e))?;

    let label_ids: Result<Vec<Uuid>, _> = request
        .label_ids
        .iter()
        .map(|l| Uuid::parse_str(l))
        .collect();

    let label_ids = label_ids.map_err(|e| format!("Invalid label ID: {}", e))?;

    let folder_ids = if let Some(folder_ids_str) = request.folder_ids {
        let parsed: Result<Vec<Uuid>, _> =
            folder_ids_str.iter().map(|f| Uuid::parse_str(f)).collect();
        Some(parsed.map_err(|e| format!("Invalid folder ID: {}", e))?)
    } else {
        None
    };

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let view_repo = repo_factory.view_repository();

    let mut view = view_repo
        .find_by_id(view_id)
        .await
        .map_err(|e| format!("Failed to find view: {}", e))?
        .ok_or_else(|| format!("View {} not found", request.view_id))?;

    let swimlane = KanbanSwimlane {
        id: Uuid::now_v7(),
        title: request.title,
        color: request.color,
        label_ids,
        folder_ids,
        state: SwimlaneState::Open,
        sort_order: request.sort_order,
    };

    match &mut view.config {
        ViewConfig::Kanban { swimlanes } => {
            swimlanes.push(swimlane);
            swimlanes.sort_by_key(|s| s.sort_order);
        }
        _ => return Err("View is not a Kanban view".to_string()),
    }

    view.updated_at = Utc::now();

    view_repo
        .update(&view)
        .await
        .map_err(|e| format!("Failed to update view: {}", e))?;

    Ok(view)
}

#[tauri::command]
pub async fn update_swimlane(
    state: State<'_, AppState>,
    request: UpdateSwimlaneRequest,
) -> Result<View, String> {
    let view_id =
        Uuid::parse_str(&request.view_id).map_err(|e| format!("Invalid view ID: {}", e))?;
    let swimlane_id =
        Uuid::parse_str(&request.swimlane_id).map_err(|e| format!("Invalid swimlane ID: {}", e))?;

    let label_ids: Result<Vec<Uuid>, _> = request
        .label_ids
        .iter()
        .map(|l| Uuid::parse_str(l))
        .collect();

    let label_ids = label_ids.map_err(|e| format!("Invalid label ID: {}", e))?;

    let folder_ids = if let Some(folder_ids_str) = request.folder_ids {
        let parsed: Result<Vec<Uuid>, _> =
            folder_ids_str.iter().map(|f| Uuid::parse_str(f)).collect();
        Some(parsed.map_err(|e| format!("Invalid folder ID: {}", e))?)
    } else {
        None
    };

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let view_repo = repo_factory.view_repository();

    let mut view = view_repo
        .find_by_id(view_id)
        .await
        .map_err(|e| format!("Failed to find view: {}", e))?
        .ok_or_else(|| format!("View {} not found", request.view_id))?;

    match &mut view.config {
        ViewConfig::Kanban { swimlanes } => {
            let swimlane = swimlanes
                .iter_mut()
                .find(|s| s.id == swimlane_id)
                .ok_or_else(|| format!("Swimlane {} not found", request.swimlane_id))?;

            swimlane.title = request.title;
            swimlane.color = request.color;
            swimlane.label_ids = label_ids;
            swimlane.folder_ids = folder_ids;
            swimlane.state = request.state;
            swimlane.sort_order = request.sort_order;

            swimlanes.sort_by_key(|s| s.sort_order);
        }
        _ => return Err("View is not a Kanban view".to_string()),
    }

    view.updated_at = Utc::now();

    view_repo
        .update(&view)
        .await
        .map_err(|e| format!("Failed to update view: {}", e))?;

    Ok(view)
}

#[tauri::command]
pub async fn delete_swimlane(
    state: State<'_, AppState>,
    view_id: String,
    swimlane_id: String,
) -> Result<View, String> {
    let view_id = Uuid::parse_str(&view_id).map_err(|e| format!("Invalid view ID: {}", e))?;
    let swimlane_id =
        Uuid::parse_str(&swimlane_id).map_err(|e| format!("Invalid swimlane ID: {}", e))?;

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let view_repo = repo_factory.view_repository();

    let mut view = view_repo
        .find_by_id(view_id)
        .await
        .map_err(|e| format!("Failed to find view: {}", e))?
        .ok_or_else(|| "View not found".to_string())?;

    match &mut view.config {
        ViewConfig::Kanban { swimlanes } => {
            swimlanes.retain(|s| s.id != swimlane_id);
        }
        _ => return Err("View is not a Kanban view".to_string()),
    }

    view.updated_at = Utc::now();

    view_repo
        .update(&view)
        .await
        .map_err(|e| format!("Failed to update view: {}", e))?;

    Ok(view)
}
