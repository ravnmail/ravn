use crate::config::{ConfigWatcher, KeyBindings, KeyBindingsWatcher, Settings};
use crate::licensing::{LicenseManager, LicenseRefreshRunner};
use crate::search::SearchManager;
use crate::services::avatar_service::AvatarService;
use crate::services::corvus::CorvusService;
use crate::sync::auth::CredentialStore;
use crate::sync::{
    BackgroundAiAnalyzer, BackgroundAvatarFetcher, BackgroundBodyFetcher, BackgroundCleanup,
    BackgroundSyncManager, OAuthStateManager, SyncCoordinator,
};
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;

pub struct AppState {
    pub db_pool: SqlitePool,
    pub settings: Arc<Settings>,
    pub keybindings: Arc<KeyBindings>,
    pub ai_service: Arc<CorvusService>,
    pub avatar_service: Arc<AvatarService>,
    pub oauth_state_manager: Arc<OAuthStateManager>,
    pub background_sync_manager: Arc<BackgroundSyncManager>,
    pub background_body_fetcher: Arc<BackgroundBodyFetcher>,
    pub background_ai_analyzer: Arc<BackgroundAiAnalyzer>,
    pub background_avatar_fetcher: Arc<BackgroundAvatarFetcher>,
    pub background_cleanup: Arc<BackgroundCleanup>,
    pub sync_coordinator: Arc<SyncCoordinator>,
    pub credential_store: Arc<CredentialStore>,
    pub search_manager: Arc<SearchManager>,
    pub license_manager: Arc<LicenseManager>,
    pub license_refresh_runner: Arc<LicenseRefreshRunner>,
    pub app_handle: tauri::AppHandle,
    pub app_data_dir: PathBuf,
    pub download_dir: PathBuf,
    pub _config_watcher: ConfigWatcher,
    pub _keybindings_watcher: KeyBindingsWatcher,
}
