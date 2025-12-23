use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::config::error::ConfigError;
use config::{Config, File};
use serde::Deserialize;
use serde_json::Value as JsonValue;

#[derive(Debug, Clone)]
pub struct Settings {
    inner: Arc<RwLock<Config>>,
    user_config_path: PathBuf,
    default_path: PathBuf,
}

impl Settings {
    /// Create a new settings instance with bundled defaults and user overrides
    pub fn new(resource_dir: &Path, app_data_dir: &Path) -> Result<Self, ConfigError> {
        std::fs::create_dir_all(app_data_dir)?;
        let default_path = resource_dir.join("resources/settings.json5");
        let user_config_path = app_data_dir.join("settings.json5");

        if !user_config_path.exists() {
            std::fs::write(&user_config_path, "{}")?;
        }

        let config = Self::load_config(&default_path, &user_config_path)?;

        Ok(Self {
            inner: Arc::new(RwLock::new(config)),
            user_config_path,
            default_path,
        })
    }

    /// Load configuration from bundled defaults and user config with proper merging
    fn load_config(default_path: &Path, user_config_path: &Path) -> Result<Config, ConfigError> {
        let config = Config::builder()
            .add_source(File::from(default_path))
            .add_source(File::from(user_config_path.to_path_buf()))
            .build()?;

        Ok(config)
    }

    /// Retrieve a setting value using dot notation (e.g., "ai.api.baseUrl")
    pub fn get<'de, T: Deserialize<'de>>(&self, key: &str) -> Result<T, ConfigError> {
        let config_guard = self.inner.read().map_err(|_| {
            ConfigError::AccessError("Failed to acquire read lock for config".to_string())
        })?;

        Ok(config_guard.get::<T>(key)?)
    }

    /// Get all settings as a hashmap
    pub fn as_table(
        &self,
    ) -> Result<std::collections::HashMap<String, config::Value>, ConfigError> {
        let config_guard = self.inner.read().map_err(|_| {
            ConfigError::AccessError("Failed to acquire read lock for config".to_string())
        })?;

        Ok(config_guard.clone().try_deserialize()?)
    }

    /// Reload configuration from disk files
    pub fn reload(&self) -> Result<(), ConfigError> {
        let new_config = Self::load_config(&self.default_path, &self.user_config_path)?;

        let mut config_guard = self.inner.write().map_err(|_| {
            ConfigError::AccessError("Failed to acquire write lock for config".to_string())
        })?;
        *config_guard = new_config;

        log::info!("Configuration reloaded");

        Ok(())
    }

    pub fn user_config_path(&self) -> &Path {
        &self.user_config_path
    }

    /// Set a setting value and persist to disk in flat key format
    pub fn set(&self, key: &str, value: JsonValue) -> Result<(), ConfigError> {
        let user_config_content = std::fs::read_to_string(&self.user_config_path)?;
        let mut user_config: JsonValue =
            if user_config_content.trim().is_empty() || user_config_content.trim() == "{}" {
                serde_json::json!({})
            } else {
                json5::from_str(&user_config_content).map_err(|e| {
                    ConfigError::AccessError(format!("Failed to parse user config: {}", e))
                })?
            };

        Self::set_flat_value(&mut user_config, key, value)?;

        let serialized = serde_json::to_string_pretty(&user_config)
            .map_err(|e| ConfigError::AccessError(format!("Failed to serialize config: {}", e)))?;
        std::fs::write(&self.user_config_path, serialized)?;

        self.reload()?;

        Ok(())
    }

    pub fn get_user_keys(&self) -> Result<Vec<String>, ConfigError> {
        let user_config_content = std::fs::read_to_string(&self.user_config_path)?;
        let user_config: JsonValue =
            if user_config_content.trim().is_empty() || user_config_content.trim() == "{}" {
                serde_json::json!({})
            } else {
                json5::from_str(&user_config_content).map_err(|e| {
                    ConfigError::AccessError(format!("Failed to parse user config: {}", e))
                })?
            };

        if !user_config.is_object() {
            return Err(ConfigError::AccessError(
                "Root config must be an object".to_string(),
            ));
        }

        let obj = user_config.as_object().unwrap();
        Ok(obj.keys().cloned().collect())
    }

    pub fn remove(&self, key: &str) -> Result<(), ConfigError> {
        let user_config_content = std::fs::read_to_string(&self.user_config_path)?;
        let mut user_config: JsonValue =
            if user_config_content.trim().is_empty() || user_config_content.trim() == "{}" {
                serde_json::json!({})
            } else {
                json5::from_str(&user_config_content).map_err(|e| {
                    ConfigError::AccessError(format!("Failed to parse user config: {}", e))
                })?
            };

        if !user_config.is_object() {
            return Err(ConfigError::AccessError(
                "Root config must be an object".to_string(),
            ));
        }

        let obj = user_config.as_object_mut().unwrap();
        obj.remove(key);

        let serialized = serde_json::to_string_pretty(&user_config)
            .map_err(|e| ConfigError::AccessError(format!("Failed to serialize config: {}", e)))?;
        std::fs::write(&self.user_config_path, serialized)?;

        self.reload()?;

        Ok(())
    }

    /// Set a value using flat dot-notation keys (e.g., "appearance.theme")
    /// This maintains a flat structure in the user config file while supporting dot notation access
    fn set_flat_value(
        config: &mut JsonValue,
        key: &str,
        value: JsonValue,
    ) -> Result<(), ConfigError> {
        if !config.is_object() {
            return Err(ConfigError::AccessError(
                "Root config must be an object".to_string(),
            ));
        }

        let obj = config.as_object_mut().unwrap();
        obj.insert(key.to_string(), value);

        Ok(())
    }

    /// Get all settings as a JSON value
    pub fn get_all(&self) -> Result<JsonValue, ConfigError> {
        let config_guard = self.inner.read().map_err(|_| {
            ConfigError::AccessError("Failed to acquire read lock for config".to_string())
        })?;

        let table = config_guard.clone().try_deserialize::<JsonValue>()?;

        let json_str = serde_json::to_string(&table)
            .map_err(|e| ConfigError::AccessError(format!("Failed to serialize config: {}", e)))?;
        let json_value: JsonValue = serde_json::from_str(&json_str)
            .map_err(|e| ConfigError::AccessError(format!("Failed to parse config: {}", e)))?;

        Ok(json_value)
    }
}
