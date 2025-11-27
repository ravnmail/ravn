use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::ConfigError),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Watcher error: {0}")]
    WatcherError(#[from] notify::Error),

    #[error("Failed to access configuration: {0}")]
    AccessError(String),
}
