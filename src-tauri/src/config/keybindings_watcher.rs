use std::sync::Arc;
use std::time::Duration;

use notify::{Error, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter};

use crate::config::error::ConfigError;
use crate::config::keybindings::KeyBindings;

/// Keybindings file watcher
pub struct KeyBindingsWatcher {
    _watcher: RecommendedWatcher,
}

impl KeyBindingsWatcher {
    pub fn new(keybindings: Arc<KeyBindings>, app_handle: AppHandle) -> Result<Self, ConfigError> {
        let path_to_watch = keybindings.user_keymap_path().to_path_buf();

        let mut watcher = RecommendedWatcher::new(
            move |result: Result<Event, Error>| {
                let event = result.unwrap();

                if event.kind.is_modify() {
                    if let Err(err) = keybindings.reload() {
                        log::error!("Failed to reload keybindings: {}", err);
                    } else {
                        log::info!("Keybindings reloaded due to file changes");

                        // Emit event to frontend
                        if let Err(err) = app_handle.emit("keybindings-changed", ()) {
                            log::error!("Failed to emit keybindings-changed event: {}", err);
                        }
                    }
                }
            },
            notify::Config::default()
                .with_compare_contents(true)
                .with_poll_interval(Duration::from_secs(2)),
        )?;

        watcher.watch(&path_to_watch, RecursiveMode::Recursive)?;

        Ok(Self { _watcher: watcher })
    }
}
