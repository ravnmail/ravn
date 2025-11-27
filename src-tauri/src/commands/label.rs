use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::{
    database::{
        models::label::Label,
        repositories::{LabelRepository, RepositoryFactory},
    },
    state::AppState,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLabelRequest {
    pub name: String,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateLabelRequest {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddLabelToEmailRequest {
    pub email_id: String,
    pub label_id: String,
}

#[tauri::command]
pub async fn get_labels(state: State<'_, AppState>) -> Result<Vec<Label>, String> {
    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let label_repo = repo_factory.label_repository();

    label_repo
        .get_all()
        .await
        .map_err(|e| format!("Failed to get labels: {}", e))
}

#[tauri::command]
pub async fn get_label(
    state: State<'_, AppState>,
    label_id: String,
) -> Result<Option<Label>, String> {
    let id = Uuid::parse_str(&label_id).map_err(|e| format!("Invalid label ID: {}", e))?;

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let label_repo = repo_factory.label_repository();

    label_repo
        .find_by_id(id)
        .await
        .map_err(|e| format!("Failed to get label: {}", e))
}

#[tauri::command]
pub async fn get_email_labels(
    state: State<'_, AppState>,
    email_id: String,
) -> Result<Vec<Label>, String> {
    let id = Uuid::parse_str(&email_id).map_err(|e| format!("Invalid email ID: {}", e))?;

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let label_repo = repo_factory.label_repository();

    label_repo
        .find_by_email(id)
        .await
        .map_err(|e| format!("Failed to get email labels: {}", e))
}

#[tauri::command]
pub async fn create_label(
    state: State<'_, AppState>,
    request: CreateLabelRequest,
) -> Result<Label, String> {
    let label = Label {
        id: Uuid::now_v7(),
        name: request.name,
        color: request.color,
        icon: request.icon,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let label_repo = repo_factory.label_repository();

    label_repo
        .create(&label)
        .await
        .map_err(|e| format!("Failed to create label: {}", e))?;

    Ok(label)
}

#[tauri::command]
pub async fn update_label(
    state: State<'_, AppState>,
    request: UpdateLabelRequest,
) -> Result<Label, String> {
    let id = Uuid::parse_str(&request.id).map_err(|e| format!("Invalid label ID: {}", e))?;

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let label_repo = repo_factory.label_repository();

    let existing = label_repo
        .find_by_id(id)
        .await
        .map_err(|e| format!("Failed to find label: {}", e))?
        .ok_or_else(|| format!("Label {} not found", request.id))?;

    let updated_label = Label {
        id,
        name: request.name,
        icon: request.icon,
        color: request.color,
        created_at: existing.created_at,
        updated_at: Utc::now(),
    };

    let _ = label_repo
        .update(&updated_label)
        .await
        .map_err(|e| format!("Failed to update label: {}", e));

    Ok(updated_label)
}

#[tauri::command]
pub async fn delete_label(state: State<'_, AppState>, label_id: String) -> Result<(), String> {
    let id = Uuid::parse_str(&label_id).map_err(|e| format!("Invalid label ID: {}", e))?;

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let label_repo = repo_factory.label_repository();

    label_repo
        .delete(id)
        .await
        .map_err(|e| format!("Failed to delete label: {}", e))
}

#[tauri::command]
pub async fn add_label_to_email(
    state: State<'_, AppState>,
    request: AddLabelToEmailRequest,
) -> Result<(), String> {
    let email_id =
        Uuid::parse_str(&request.email_id).map_err(|e| format!("Invalid email ID: {}", e))?;
    let label_id =
        Uuid::parse_str(&request.label_id).map_err(|e| format!("Invalid label ID: {}", e))?;

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let label_repo = repo_factory.label_repository();

    label_repo
        .add_to_email(email_id, label_id)
        .await
        .map_err(|e| format!("Failed to add label to email: {}", e))
}

#[tauri::command]
pub async fn remove_label_from_email(
    state: State<'_, AppState>,
    email_id: String,
    label_id: String,
) -> Result<(), String> {
    let email_id = Uuid::parse_str(&email_id).map_err(|e| format!("Invalid email ID: {}", e))?;
    let label_id = Uuid::parse_str(&label_id).map_err(|e| format!("Invalid label ID: {}", e))?;

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let label_repo = repo_factory.label_repository();

    label_repo
        .remove_from_email(email_id, label_id)
        .await
        .map_err(|e| format!("Failed to remove label from email: {}", e))
}
