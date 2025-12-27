use crate::state::AppState;
use serde_json::Value as JsonValue;
use tauri::State;

/// Get all keybindings (merged defaults + user overrides)
#[tauri::command]
pub async fn get_keybindings(state: State<'_, AppState>) -> Result<JsonValue, String> {
    let keybindings = state.keybindings.get_all().map_err(|e| e.to_string())?;
    serde_json::to_value(keybindings).map_err(|e| e.to_string())
}

/// Get only user-defined keybindings
#[tauri::command]
pub async fn get_user_keybindings(state: State<'_, AppState>) -> Result<JsonValue, String> {
    let keybindings = state
        .keybindings
        .get_user_keymap()
        .map_err(|e| e.to_string())?;
    serde_json::to_value(keybindings).map_err(|e| e.to_string())
}

/// Set a user keybinding
#[tauri::command]
pub async fn set_keybinding(
    state: State<'_, AppState>,
    context: String,
    key: String,
    action: Option<String>,
    props: Option<JsonValue>,
) -> Result<(), String> {
    state
        .keybindings
        .set(&context, &key, action, props)
        .map_err(|e| e.to_string())
}

/// Remove a user keybinding
#[tauri::command]
pub async fn remove_keybinding(
    state: State<'_, AppState>,
    context: String,
    key: String,
) -> Result<(), String> {
    state
        .keybindings
        .remove(&context, &key)
        .map_err(|e| e.to_string())
}

/// Reload keybindings from disk
#[tauri::command]
pub async fn reload_keybindings(state: State<'_, AppState>) -> Result<(), String> {
    state.keybindings.reload().map_err(|e| e.to_string())
}
