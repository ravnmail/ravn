use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::config::error::ConfigError;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBinding {
    pub context: String,
    pub bindings: std::collections::HashMap<String, KeyAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum KeyAction {
    None,
    Simple(String),
    WithProps(String, JsonValue),
}

impl KeyAction {
    pub fn action(&self) -> Option<&str> {
        match self {
            KeyAction::None => None,
            KeyAction::Simple(action) => Some(action),
            KeyAction::WithProps(action, _) => Some(action),
        }
    }

    pub fn props(&self) -> Option<&JsonValue> {
        match self {
            KeyAction::None => None,
            KeyAction::Simple(_) => None,
            KeyAction::WithProps(_, props) => Some(props),
        }
    }
}

pub type KeyMapFile = Vec<KeyBinding>;

#[derive(Debug, Clone)]
pub struct KeyBindings {
    inner: Arc<RwLock<KeyMapFile>>,
    user_keymap_path: PathBuf,
    default_keymap_path: PathBuf,
}

impl KeyBindings {
    /// Create a new keybindings instance with bundled defaults and user overrides
    pub fn new(
        resource_dir: &Path,
        app_data_dir: &Path,
        default_mapping_name: Option<String>,
    ) -> Result<Self, ConfigError> {
        std::fs::create_dir_all(app_data_dir)?;

        let default_name = default_mapping_name.unwrap_or_else(|| "default.json".to_string());
        let default_keymap_path = resource_dir.join(format!("resources/keymaps/{}", default_name));
        let user_keymap_path = app_data_dir.join("keymap.json");

        // Create user keymap file if it doesn't exist
        if !user_keymap_path.exists() {
            std::fs::write(&user_keymap_path, "[]")?;
        }

        // Ensure default keymap exists, create with empty bindings if not
        if !default_keymap_path.exists() {
            log::warn!(
                "Default keymap not found at {:?}, creating empty default",
                default_keymap_path
            );
            if let Some(parent) = default_keymap_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&default_keymap_path, "[]")?;
        }

        let keymap = Self::load_keymaps(&default_keymap_path, &user_keymap_path)?;

        Ok(Self {
            inner: Arc::new(RwLock::new(keymap)),
            user_keymap_path,
            default_keymap_path,
        })
    }

    /// Load keymaps from default and user files, merging them appropriately
    fn load_keymaps(default_path: &Path, user_path: &Path) -> Result<KeyMapFile, ConfigError> {
        // Load default keymap with fallback
        let default_keymap: KeyMapFile = match std::fs::read_to_string(default_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
                log::error!("Failed to parse default keymap: {}, using empty default", e);
                vec![]
            }),
            Err(e) => {
                log::warn!("Failed to read default keymap: {}, using empty default", e);
                vec![]
            }
        };

        // Load user keymap with fallback
        let user_keymap: KeyMapFile = match std::fs::read_to_string(user_path) {
            Ok(content) if !content.trim().is_empty() && content.trim() != "[]" => {
                serde_json::from_str(&content).unwrap_or_else(|e| {
                    log::error!(
                        "Failed to parse user keymap: {}, using empty user config",
                        e
                    );
                    vec![]
                })
            }
            _ => vec![],
        };

        // Start with a mutable copy of defaults
        let mut merged_keymap = default_keymap;

        // Merge user keymaps into defaults
        // User bindings override default bindings for the same key in the same context
        for user_context in user_keymap {
            if let Some(default_context) = merged_keymap
                .iter_mut()
                .find(|c| c.context == user_context.context)
            {
                // Override/extend bindings in existing context
                for (key, action) in user_context.bindings {
                    default_context.bindings.insert(key, action);
                }
            } else {
                // Add new context from user
                merged_keymap.push(user_context);
            }
        }

        Ok(merged_keymap)
    }

    /// Get all keybindings as a merged result
    pub fn get_all(&self) -> Result<KeyMapFile, ConfigError> {
        let keymap_guard = self.inner.read().map_err(|_| {
            ConfigError::AccessError("Failed to acquire read lock for keybindings".to_string())
        })?;

        Ok(keymap_guard.clone())
    }

    /// Reload keybindings from disk files
    pub fn reload(&self) -> Result<(), ConfigError> {
        let new_keymap = Self::load_keymaps(&self.default_keymap_path, &self.user_keymap_path)?;

        let mut keymap_guard = self.inner.write().map_err(|_| {
            ConfigError::AccessError("Failed to acquire write lock for keybindings".to_string())
        })?;
        *keymap_guard = new_keymap;

        log::info!("Keybindings reloaded");

        Ok(())
    }

    pub fn user_keymap_path(&self) -> &Path {
        &self.user_keymap_path
    }

    /// Set a user keybinding
    pub fn set(
        &self,
        context: &str,
        key: &str,
        action: Option<String>,
        props: Option<JsonValue>,
    ) -> Result<(), ConfigError> {
        let user_content = std::fs::read_to_string(&self.user_keymap_path)?;
        let mut user_keymap: KeyMapFile =
            if user_content.trim().is_empty() || user_content.trim() == "[]" {
                vec![]
            } else {
                serde_json::from_str(&user_content).map_err(|e| {
                    ConfigError::AccessError(format!("Failed to parse user keymap: {}", e))
                })?
            };

        // Find or create the context
        let context_entry = user_keymap.iter_mut().find(|c| c.context == context);

        let key_action = match (action, props) {
            (None, _) => KeyAction::None,
            (Some(a), None) => KeyAction::Simple(a),
            (Some(a), Some(p)) => KeyAction::WithProps(a, p),
        };

        if let Some(entry) = context_entry {
            entry.bindings.insert(key.to_string(), key_action);
        } else {
            let mut bindings = std::collections::HashMap::new();
            bindings.insert(key.to_string(), key_action);
            user_keymap.push(KeyBinding {
                context: context.to_string(),
                bindings,
            });
        }

        // Write back to file
        let serialized = serde_json::to_string_pretty(&user_keymap)
            .map_err(|e| ConfigError::AccessError(format!("Failed to serialize keymap: {}", e)))?;
        std::fs::write(&self.user_keymap_path, serialized)?;

        // Reload to update merged state
        self.reload()?;

        Ok(())
    }

    /// Remove a user keybinding
    pub fn remove(&self, context: &str, key: &str) -> Result<(), ConfigError> {
        let user_content = std::fs::read_to_string(&self.user_keymap_path)?;
        let mut user_keymap: KeyMapFile =
            if user_content.trim().is_empty() || user_content.trim() == "[]" {
                vec![]
            } else {
                serde_json::from_str(&user_content).map_err(|e| {
                    ConfigError::AccessError(format!("Failed to parse user keymap: {}", e))
                })?
            };

        // Find the context and remove the key
        if let Some(context_entry) = user_keymap.iter_mut().find(|c| c.context == context) {
            context_entry.bindings.remove(key);

            // Remove context if it has no bindings left
            if context_entry.bindings.is_empty() {
                user_keymap.retain(|c| c.context != context);
            }
        }

        // Write back to file
        let serialized = serde_json::to_string_pretty(&user_keymap)
            .map_err(|e| ConfigError::AccessError(format!("Failed to serialize keymap: {}", e)))?;
        std::fs::write(&self.user_keymap_path, serialized)?;

        // Reload to update merged state
        self.reload()?;

        Ok(())
    }

    /// Get user-defined keybindings only
    pub fn get_user_keymap(&self) -> Result<KeyMapFile, ConfigError> {
        let user_content = std::fs::read_to_string(&self.user_keymap_path)?;
        let user_keymap: KeyMapFile =
            if user_content.trim().is_empty() || user_content.trim() == "[]" {
                vec![]
            } else {
                serde_json::from_str(&user_content).map_err(|e| {
                    ConfigError::AccessError(format!("Failed to parse user keymap: {}", e))
                })?
            };

        Ok(user_keymap)
    }
}
