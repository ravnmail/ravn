use std::sync::Arc;
use std::time::Duration;

use notify::{Error, Event, RecommendedWatcher, RecursiveMode, Watcher};

use crate::config::error::ConfigError;
use crate::config::settings::Settings;

/// Configuration file watcher
pub struct ConfigWatcher {
    _watcher: RecommendedWatcher,
}

impl ConfigWatcher {
    pub fn new(settings: Arc<Settings>) -> Result<Self, ConfigError> {
        let path_to_watch = settings.user_config_path().to_path_buf();

        let mut watcher = RecommendedWatcher::new(
            move |result: Result<Event, Error>| {
                let event = result.unwrap();

                if event.kind.is_modify() {
                    if let Err(err) = settings.reload() {
                        log::error!("Failed to reload configuration: {}", err);
                    } else {
                        log::info!("Configuration reloaded due to file changes");
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
