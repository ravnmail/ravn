use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{Manager, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInfo {
    pub id: String,
    pub name: String,
    pub source: ThemeSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeSource {
    Builtin,
    User,
}

/// List all available themes from both builtin and user directories
#[tauri::command]
pub async fn list_themes(state: State<'_, AppState>) -> Result<Vec<ThemeInfo>, String> {
    let mut themes = Vec::new();

    // Get builtin themes from resources/themes
    let resource_dir = state
        .app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource directory: {}", e))?;
    let builtin_themes_dir = resource_dir.join("resources/themes");

    log::debug!(
        "Scanning for builtin themes in: {}",
        builtin_themes_dir.display()
    );
    log::debug!(
        "Builtin themes directory exists: {}",
        builtin_themes_dir.exists()
    );

    if builtin_themes_dir.exists() {
        if let Ok(entries) = fs::read_dir(&builtin_themes_dir) {
            for entry in entries.flatten() {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.ends_with(".css") {
                        let name = filename.trim_end_matches(".css");
                        themes.push(ThemeInfo {
                            id: format!("builtin/{}", filename),
                            name: name.to_string(),
                            source: ThemeSource::Builtin,
                        });
                    }
                }
            }
        }
    }

    // Get user themes from app data directory
    let user_themes_dir = state.app_data_dir.join("themes");

    log::debug!("Scanning for user themes in: {}", user_themes_dir.display());
    log::debug!("User themes directory exists: {}", user_themes_dir.exists());

    if user_themes_dir.exists() {
        if let Ok(entries) = fs::read_dir(&user_themes_dir) {
            for entry in entries.flatten() {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.ends_with(".css") {
                        let name = filename.trim_end_matches(".css");
                        themes.push(ThemeInfo {
                            id: format!("user/{}", filename),
                            name: name.to_string(),
                            source: ThemeSource::User,
                        });
                    }
                }
            }
        }
    }

    log::debug!("Available themes: {:?}", themes);

    Ok(themes)
}

/// Get the content of a theme file
#[tauri::command]
pub async fn get_theme(state: State<'_, AppState>, theme_id: String) -> Result<String, String> {
    let theme_path = resolve_theme_path(&state, &theme_id)?;

    // Security check: ensure the resolved path is within allowed directories
    validate_theme_path(&state, &theme_path)?;

    fs::read_to_string(&theme_path).map_err(|e| format!("Failed to read theme file: {}", e))
}

/// Switch to a different theme and save the preference
#[tauri::command]
pub async fn switch_theme(state: State<'_, AppState>, theme_id: String) -> Result<String, String> {
    // Validate that the theme exists by trying to get it
    let content = get_theme(state.clone(), theme_id.clone()).await?;

    // Save the theme preference to settings
    state
        .settings
        .set("appearance.theme", serde_json::json!(theme_id))
        .map_err(|e| format!("Failed to save theme preference: {}", e))?;

    Ok(content)
}

/// Get the currently selected theme
#[tauri::command]
pub async fn get_current_theme(state: State<'_, AppState>) -> Result<String, String> {
    state
        .settings
        .get::<String>("appearance.theme")
        .map_err(|_| "builtin/light.css".to_string())
        .or(Ok("builtin/light.css".to_string()))
}

// Helper functions

fn resolve_theme_path(state: &AppState, theme_id: &str) -> Result<PathBuf, String> {
    log::debug!("Resolving theme path for: {}", theme_id);

    // Parse the theme_id (format: "builtin/filename.css" or "user/filename.css")
    let parts: Vec<&str> = theme_id.split('/').collect();
    if parts.len() != 2 {
        return Err("Invalid theme ID format. Expected 'source/filename.css'".to_string());
    }

    let source = parts[0];
    let filename = parts[1];

    log::debug!("Theme source: {}, filename: {}", source, filename);

    // Validate filename to prevent path traversal attacks
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err("Invalid filename: path traversal not allowed".to_string());
    }

    if !filename.ends_with(".css") {
        return Err("Invalid filename: must be a .css file".to_string());
    }

    let path = match source {
        "builtin" => {
            let resource_dir = state
                .app_handle
                .path()
                .resource_dir()
                .map_err(|e| format!("Failed to get resource directory: {}", e))?;
            let theme_path = resource_dir.join("resources/themes").join(filename);
            log::debug!("Resolved builtin theme path: {}", theme_path.display());
            theme_path
        }
        "user" => {
            let theme_path = state.app_data_dir.join("themes").join(filename);
            log::debug!("Resolved user theme path: {}", theme_path.display());
            theme_path
        }
        _ => return Err("Invalid theme source. Must be 'builtin' or 'user'".to_string()),
    };

    Ok(path)
}

fn validate_theme_path(state: &AppState, path: &Path) -> Result<(), String> {
    // First check if the file exists
    if !path.exists() {
        return Err(format!("Theme file does not exist: {}", path.display()));
    }

    let resource_dir = state
        .app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource directory: {}", e))?;
    let builtin_themes_dir = resource_dir.join("resources/themes");
    let user_themes_dir = state.app_data_dir.join("themes");

    log::debug!("Validating theme path: {}", path.display());
    log::debug!("Builtin themes dir: {}", builtin_themes_dir.display());
    log::debug!("User themes dir: {}", user_themes_dir.display());

    // Canonicalize paths for comparison
    let canonical_path = path
        .canonicalize()
        .map_err(|e| format!("Failed to resolve theme path: {}", e))?;

    log::debug!("Canonical theme path: {}", canonical_path.display());

    // Check if path is in builtin directory
    let is_in_builtin = if builtin_themes_dir.exists() {
        builtin_themes_dir
            .canonicalize()
            .ok()
            .map(|dir| {
                log::debug!("Canonical builtin dir: {}", dir.display());
                canonical_path.starts_with(dir)
            })
            .unwrap_or(false)
    } else {
        log::warn!(
            "Builtin themes directory does not exist: {}",
            builtin_themes_dir.display()
        );
        false
    };

    // Check if path is in user directory
    let is_in_user = if user_themes_dir.exists() {
        user_themes_dir
            .canonicalize()
            .ok()
            .map(|dir| {
                log::debug!("Canonical user dir: {}", dir.display());
                canonical_path.starts_with(dir)
            })
            .unwrap_or(false)
    } else {
        log::debug!(
            "User themes directory does not exist: {}",
            user_themes_dir.display()
        );
        false
    };

    log::debug!(
        "Is in builtin: {}, Is in user: {}",
        is_in_builtin,
        is_in_user
    );

    if !is_in_builtin && !is_in_user {
        return Err(format!(
            "Theme path is not in an allowed directory. Path: {}, Builtin: {}, User: {}",
            canonical_path.display(),
            builtin_themes_dir.display(),
            user_themes_dir.display()
        ));
    }

    Ok(())
}
