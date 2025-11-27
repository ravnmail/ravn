use crate::commands::emails::AttachmentData;
use crate::database::repositories::{AttachmentRepository, SqliteAttachmentRepository};
use crate::state::AppState;
use crate::sync::storage::PathGenerator;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentInfo {
    pub id: String,
    pub email_id: String,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub is_inline: bool,
    pub is_cached: bool,
    pub full_path: Option<String>,
}

#[tauri::command]
pub async fn get_email_attachments(
    state: State<'_, AppState>,
    email_id: String,
) -> Result<Vec<AttachmentInfo>, String> {
    log::info!("Getting attachments for email: {}", email_id);

    let email_uuid = Uuid::parse_str(&email_id).map_err(|e| format!("Invalid email ID: {}", e))?;

    let attachment_repo = SqliteAttachmentRepository::new(state.db_pool.clone());
    let attachments = attachment_repo
        .find_by_email(email_uuid)
        .await
        .map_err(|e| format!("Failed to get attachments: {}", e))?;

    log::debug!("Found {} attachments", attachments.len());

    let app_data_dir = PathBuf::from(&state.app_data_dir);

    let attachment_infos: Vec<AttachmentInfo> = attachments
        .into_iter()
        .map(|a| {
            let full_path = if a.is_cached && a.cache_path.is_some() {
                let cache_path = a.cache_path.unwrap();
                let path_buf = PathGenerator::cache_path_to_pathbuf(&cache_path);
                let full_path_buf = app_data_dir.join("attachments").join(path_buf);
                Some(full_path_buf.to_string_lossy().to_string())
            } else {
                None
            };

            AttachmentInfo {
                id: a.id.to_string(),
                email_id: a.email_id.to_string(),
                filename: a.filename,
                content_type: a.content_type,
                size: a.size,
                is_inline: a.is_inline,
                is_cached: a.is_cached,
                full_path,
            }
        })
        .collect();

    Ok(attachment_infos)
}

#[tauri::command]
pub async fn open_attachment(_state: State<'_, AppState>, file_path: String) -> Result<(), String> {
    log::info!("Opening attachment: {}", file_path);

    let path = PathBuf::from(&file_path);

    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    opener::open(&path).map_err(|e| format!("Failed to open file: {}", e))?;

    Ok(())
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn quicklook_attachment(
    _state: State<'_, AppState>,
    file_paths: Vec<String>,
) -> Result<(), String> {
    use std::process::Command;

    log::info!("QuickLook for {} files", file_paths.len());

    if file_paths.is_empty() {
        return Err("No files provided".to_string());
    }

    let path_buf = PathBuf::from(file_paths[0].clone());

    let mut cmd = Command::new("qlmanage");
    cmd.arg("-p");
    cmd.arg(path_buf);

    cmd.spawn()
        .map_err(|e| format!("Failed to launch QuickLook: {}", e))?;

    Ok(())
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub async fn quicklook_attachment(
    state: State<'_, AppState>,
    file_paths: Vec<String>,
) -> Result<(), String> {
    if file_paths.is_empty() {
        return Err("No files provided".to_string());
    }

    open_attachment(state, file_paths[0].clone()).await
}

#[tauri::command]
pub async fn save_attachment(
    _state: State<'_, AppState>,
    source_path: String,
    destination_path: String,
) -> Result<(), String> {
    log::info!(
        "Saving attachment from {} to {}",
        source_path,
        destination_path
    );

    let source = PathBuf::from(&source_path);
    let destination = PathBuf::from(&destination_path);

    if !source.exists() {
        return Err(format!("Source file not found: {}", source_path));
    }

    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create destination directory: {}", e))?;
    }

    std::fs::copy(&source, &destination).map_err(|e| format!("Failed to copy file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn get_downloads_path(state: State<'_, AppState>) -> Result<String, String> {
    let downloads_dir = &state.download_dir;

    Ok(downloads_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn read_attachment_for_forward(
    state: State<'_, AppState>,
    attachment_id: String,
) -> Result<AttachmentData, String> {
    log::info!("Reading attachment for forward: {}", attachment_id);

    let attachment_uuid =
        Uuid::parse_str(&attachment_id).map_err(|e| format!("Invalid attachment ID: {}", e))?;

    let attachment_repo = SqliteAttachmentRepository::new(state.db_pool.clone());
    let attachment = attachment_repo
        .find_by_id(attachment_uuid)
        .await
        .map_err(|e| format!("Failed to get attachment: {}", e))?
        .ok_or_else(|| format!("Attachment not found: {}", attachment_id))?;

    if !attachment.is_cached || attachment.cache_path.is_none() {
        return Err("Attachment not cached".to_string());
    }

    let app_data_dir = PathBuf::from(&state.app_data_dir);
    let cache_path = attachment.cache_path.unwrap();
    let path_buf = PathGenerator::cache_path_to_pathbuf(&cache_path);
    let full_path = app_data_dir.join("attachments").join(path_buf);

    let content =
        fs::read(&full_path).map_err(|e| format!("Failed to read attachment file: {}", e))?;

    Ok(AttachmentData {
        filename: attachment.filename,
        content,
        content_type: Some(attachment.content_type),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecalculateHashesResult {
    pub total_cached: usize,
    pub processed: usize,
    pub updated: usize,
    pub errors: usize,
    pub error_messages: Vec<String>,
}

/// Recalculate hashes for all cached attachments
#[tauri::command]
pub async fn recalculate_attachment_hashes(
    state: State<'_, AppState>,
) -> Result<RecalculateHashesResult, String> {
    log::info!("Starting attachment hash recalculation");

    let app_data_dir = PathBuf::from(&state.app_data_dir);

    let attachment_repo = SqliteAttachmentRepository::new(state.db_pool.clone());
    let cached_attachments = attachment_repo
        .find_all_cached()
        .await
        .map_err(|e| format!("Failed to fetch cached attachments: {}", e))?;

    let total_cached = cached_attachments.len();
    log::info!("Found {} cached attachments to process", total_cached);

    let mut processed = 0;
    let mut updated = 0;
    let mut errors = 0;
    let mut error_messages = Vec::new();

    for (attachment_id, cache_path, old_hash) in cached_attachments {
        processed += 1;

        let cache_path = match cache_path {
            Some(path) => path,
            None => {
                errors += 1;
                error_messages.push(format!("Attachment {} has no cache_path", attachment_id));
                continue;
            }
        };

        let path_buf = PathGenerator::cache_path_to_pathbuf(&cache_path);
        let full_path = app_data_dir.join("attachments").join(path_buf);

        let new_hash = match fs::read(&full_path) {
            Ok(data) => {
                format!("{:x}", md5::compute(&data))
            }
            Err(e) => {
                errors += 1;
                let msg = format!(
                    "Failed to read file for attachment {}: {}",
                    attachment_id, e
                );
                log::warn!("{}", msg);
                error_messages.push(msg);
                continue;
            }
        };

        if new_hash != old_hash {
            match attachment_repo.update_hash(&attachment_id, &new_hash).await {
                Ok(_) => {
                    updated += 1;
                    log::debug!(
                        "Updated hash for attachment {}: {} -> {}",
                        attachment_id,
                        old_hash,
                        new_hash
                    );
                }
                Err(e) => {
                    errors += 1;
                    let msg = format!(
                        "Failed to update hash for attachment {}: {}",
                        attachment_id, e
                    );
                    log::warn!("{}", msg);
                    error_messages.push(msg);
                }
            }
        }

        if processed % 100 == 0 {
            log::info!(
                "Progress: {}/{} processed, {} updated",
                processed,
                total_cached,
                updated
            );
        }
    }

    log::info!(
        "Hash recalculation complete: {}/{} processed, {} updated, {} errors",
        processed,
        total_cached,
        updated,
        errors
    );

    Ok(RecalculateHashesResult {
        total_cached,
        processed,
        updated,
        errors,
        error_messages,
    })
}
