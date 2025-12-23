use crate::config::ConfigValue;
use crate::state::AppState;
use serde_json::Value as JsonValue;
use tauri::State;

/// Get a setting by key
#[tauri::command]
pub async fn get_setting(state: State<'_, AppState>, key: String) -> Result<JsonValue, String> {
    let value: ConfigValue = state.settings.get(&key).map_err(|e| e.to_string())?;

    value.try_deserialize().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reload_settings(state: State<'_, AppState>) -> Result<(), String> {
    state.settings.reload().map_err(|e| e.to_string())
}

/// Set a setting by key - flattens nested objects to only persist leaf values
#[tauri::command]
pub async fn set_setting(
    state: State<'_, AppState>,
    key: String,
    value: JsonValue,
) -> Result<(), String> {
    fn flatten_value(prefix: &str, value: &JsonValue, results: &mut Vec<(String, JsonValue)>) {
        match value {
            JsonValue::Object(map) => {
                for (k, v) in map {
                    let new_key = format!("{}.{}", prefix, k);
                    match v {
                        JsonValue::Object(_) => flatten_value(&new_key, v, results),
                        JsonValue::Array(_) => results.push((new_key, v.clone())),
                        _ => results.push((new_key, v.clone())),
                    }
                }
            }
            JsonValue::Array(_) => results.push((prefix.to_string(), value.clone())),
            _ => results.push((prefix.to_string(), value.clone())),
        }
    }

    let mut results = Vec::new();
    flatten_value(&key, &value, &mut results);

    for (k, v) in results {
        state.settings.set(&k, v).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn remove_setting(state: State<'_, AppState>, key: String) -> Result<(), String> {
    state.settings.remove(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_user_keys(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state.settings.get_user_keys().map_err(|e| e.to_string())
}

/// Get all settings
#[tauri::command]
pub async fn get_all_settings(state: State<'_, AppState>) -> Result<JsonValue, String> {
    state.settings.get_all().map_err(|e| e.to_string())
}

/// Set multiple settings at once - only saves leaf values (non-object values)
#[tauri::command]
pub async fn set_settings(state: State<'_, AppState>, settings: JsonValue) -> Result<(), String> {
    fn flatten_json(prefix: &str, value: &JsonValue, results: &mut Vec<(String, JsonValue)>) {
        match value {
            JsonValue::Object(map) => {
                for (key, val) in map {
                    let new_prefix = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    match val {
                        JsonValue::Object(_) => flatten_json(&new_prefix, val, results),
                        JsonValue::Array(_) => {
                            results.push((new_prefix, val.clone()));
                        }
                        _ => results.push((new_prefix, val.clone())),
                    }
                }
            }
            _ => {
                if !prefix.is_empty() {
                    results.push((prefix.to_string(), value.clone()));
                }
            }
        }
    }

    let mut flattened = Vec::new();
    flatten_json("", &settings, &mut flattened);

    for (key, value) in flattened {
        state.settings.set(&key, value).map_err(|e| e.to_string())?;
    }

    Ok(())
}
