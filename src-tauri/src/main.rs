#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use app_lib::{
    commands::attachment,
    commands::config,
    commands::contacts,
    commands::conversation,
    commands::corvus,
    commands::emails,
    commands::folders,
    commands::keybindings as keybindings_commands,
    commands::label,
    commands::licensing,
    commands::navigation as nav_commands,
    commands::notification,
    commands::search,
    commands::sync,
    commands::themes,
    commands::view,
    config::ConfigWatcher,
    config::KeyBindings,
    config::KeyBindingsWatcher,
    config::Settings,
    database::Database,
    licensing::{LicenseManager, LicenseRefreshRunner},
    search::SearchManager,
    services::avatar_service::AvatarService,
    services::corvus::CorvusService,
    sync::{
        BackgroundAiAnalyzer, BackgroundAvatarFetcher, BackgroundBodyFetcher, BackgroundCleanup,
        BackgroundSyncManager, OAuthStateManager,
    },
    AppState,
};

use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    Emitter, Manager,
};

fn create_menu(app: &tauri::App) -> Result<Menu<tauri::Wry>, tauri::Error> {
    let menu = Menu::new(app)?;

    #[cfg(target_os = "macos")]
    {
        let app_menu = Submenu::new(app, "Ravn", true)?;
        app_menu.append(&PredefinedMenuItem::about(app, None, None)?)?;
        app_menu.append(&PredefinedMenuItem::separator(app)?)?;

        let settings_item = MenuItem::with_id(
            app,
            "ravn://settings",
            "Settings...",
            true,
            Some("CmdOrCtrl+,"),
        )?;
        app_menu.append(&settings_item)?;

        let keyboard_shortcuts_item = MenuItem::with_id(
            app,
            "ravn://keymap-editor",
            "Keymap editor...",
            true,
            Some("CmdOrCtrl+?"),
        )?;
        app_menu.append(&keyboard_shortcuts_item)?;

        let debug_item =
            MenuItem::with_id(app, "ravn://debugging", "Beta tools...", true, None::<&str>)?;
        app_menu.append(&debug_item)?;

        let update_item = MenuItem::with_id(
            app,
            "check_for_updates",
            "Check for Updates...",
            true,
            Some(""),
        )?;
        app_menu.append(&update_item)?;

        app_menu.append(&PredefinedMenuItem::separator(app)?)?;
        app_menu.append(&PredefinedMenuItem::hide(app, None)?)?;
        app_menu.append(&PredefinedMenuItem::hide_others(app, None)?)?;
        app_menu.append(&PredefinedMenuItem::show_all(app, None)?)?;
        app_menu.append(&PredefinedMenuItem::separator(app)?)?;
        app_menu.append(&PredefinedMenuItem::quit(app, None)?)?;

        menu.append(&app_menu)?;
    }

    let file_menu = Submenu::new(app, "File", true)?;

    #[cfg(not(target_os = "macos"))]
    {
        let settings_item = MenuItem::with_id(
            app,
            "ravn://settings",
            "Settings...",
            true,
            Some("CmdOrCtrl+,"),
        )?;
        file_menu.append(&settings_item)?;
        file_menu.append(&PredefinedMenuItem::separator(app)?)?;
    }

    file_menu.append(&PredefinedMenuItem::close_window(app, None)?)?;

    #[cfg(not(target_os = "macos"))]
    {
        file_menu.append(&PredefinedMenuItem::quit(app, None)?)?;
    }

    menu.append(&file_menu)?;

    let edit_menu = Submenu::new(app, "Edit", true)?;
    edit_menu.append(&PredefinedMenuItem::undo(app, None)?)?;
    edit_menu.append(&PredefinedMenuItem::redo(app, None)?)?;
    edit_menu.append(&PredefinedMenuItem::separator(app)?)?;
    edit_menu.append(&PredefinedMenuItem::cut(app, None)?)?;
    edit_menu.append(&PredefinedMenuItem::copy(app, None)?)?;
    edit_menu.append(&PredefinedMenuItem::paste(app, None)?)?;
    edit_menu.append(&PredefinedMenuItem::select_all(app, None)?)?;
    menu.append(&edit_menu)?;

    let view_menu = Submenu::new(app, "View", true)?;

    #[cfg(debug_assertions)]
    {
        view_menu.append(&MenuItem::with_id(
            app,
            "toggle_devtools",
            "Toggle Developer Tools",
            true,
            Some("CmdOrCtrl+Shift+I"),
        )?)?;
        view_menu.append(&PredefinedMenuItem::separator(app)?)?;
    }

    view_menu.append(&PredefinedMenuItem::fullscreen(app, None)?)?;
    menu.append(&view_menu)?;

    let window_menu = Submenu::new(app, "Window", true)?;
    window_menu.append(&PredefinedMenuItem::minimize(app, None)?)?;

    #[cfg(target_os = "macos")]
    {
        window_menu.append(&PredefinedMenuItem::maximize(app, None)?)?;
        window_menu.append(&PredefinedMenuItem::separator(app)?)?;
        window_menu.append(&MenuItem::with_id(
            app,
            "bring_all_to_front",
            "Bring All to Front",
            true,
            None::<&str>,
        )?)?;
    }

    menu.append(&window_menu)?;

    Ok(menu)
}

fn main() {
    if cfg!(debug_assertions) {
        env_logger::init();
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let app_handle = app.handle().clone();

            let app_data_dir = app_handle
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");
            let resources_dir = app_handle
                .path()
                .resource_dir()
                .expect("Failed to get resources directory");

            std::fs::create_dir_all(&app_data_dir).expect("Failed to create app directory");

            let avatar_cache_dir = app_data_dir.join("avatar_cache");
            let avatar_service = AvatarService::new(avatar_cache_dir, None);

            let settings = Arc::new(
                Settings::new(&resources_dir, &app_data_dir)
                    .expect("Failed to initialize configuration"),
            );

            let _watcher = ConfigWatcher::new(Arc::clone(&settings))
                .expect("Failed to initialize configuration watcher");

            // Initialize keybindings with optional default mapping from settings
            let default_mapping = settings.get::<String>("keyboard.defaultMapping").ok();
            let keybindings = match KeyBindings::new(&resources_dir, &app_data_dir, default_mapping)
            {
                Ok(kb) => {
                    log::info!("Keybindings initialized successfully");
                    Arc::new(kb)
                }
                Err(e) => {
                    log::error!(
                        "Failed to initialize keybindings: {}, using empty keybindings",
                        e
                    );
                    // Create a fallback with empty keybindings
                    Arc::new(
                        KeyBindings::new(&resources_dir, &app_data_dir, None).unwrap_or_else(
                            |_| {
                                panic!("Fatal: Could not initialize keybindings even with fallback")
                            },
                        ),
                    )
                }
            };

            let _keybindings_watcher =
                match KeyBindingsWatcher::new(Arc::clone(&keybindings), app_handle.clone()) {
                    Ok(watcher) => {
                        log::info!("Keybindings watcher initialized successfully");
                        watcher
                    }
                    Err(e) => {
                        log::error!("Failed to initialize keybindings watcher: {}", e);
                        // Create a dummy watcher that does nothing
                        KeyBindingsWatcher::new(Arc::clone(&keybindings), app_handle.clone())
                            .unwrap_or_else(|_| {
                                panic!("Fatal: Could not initialize keybindings watcher")
                            })
                    }
                };

            let db = tauri::async_runtime::block_on(async {
                Database::new(&app_data_dir)
                    .await
                    .expect("Failed to initialize database")
            });

            let oauth_state_manager = Arc::new(OAuthStateManager::new());

            let app_data_dir_str = app_data_dir.to_string_lossy().to_string();
            let credential_store = Arc::new(app_lib::sync::auth::CredentialStore::new(
                Some(db.get_pool().clone()),
                Some(app_data_dir_str.clone()),
            ));

            let background_sync_manager = Arc::new(BackgroundSyncManager::new(
                db.get_pool().clone(),
                app_data_dir_str.clone(),
                Arc::clone(&credential_store),
                app_handle.clone(),
            ));

            let background_body_fetcher = Arc::new(BackgroundBodyFetcher::new(
                db.get_pool().clone(),
                app_data_dir_str.clone(),
                Arc::clone(&credential_store),
            ));

            // Initialize licensing system
            let activation_service_url =
                option_env!("ACTIVATION_SERVICE_URL").map(|s| s.to_string());
            let mid_secret = option_env!("MID_SECRET").map(|s| s.to_string());

            log::info!(
                "Licensing configuration - Service URL: {}, Secret: {}",
                activation_service_url.is_some(),
                mid_secret.is_some()
            );

            let license_manager = Arc::new(
                LicenseManager::new(app_data_dir.clone(), activation_service_url, mid_secret)
                    .expect("Failed to initialize license manager"),
            );

            // Load cached license
            tauri::async_runtime::block_on(async {
                if let Err(e) = license_manager.load_cached_license().await {
                    log::error!("Failed to load cached license: {}", e);
                }

                // Validate license on startup if online
                if !license_manager.is_open_source_mode() {
                    match license_manager.validate_license().await {
                        Ok(_) => log::info!("License validated successfully"),
                        Err(e) => log::warn!("License validation failed on startup: {}", e),
                    }
                }
            });

            let license_refresh_runner =
                Arc::new(LicenseRefreshRunner::new(Arc::clone(&license_manager)));
            let license_refresh_runner_clone = Arc::clone(&license_refresh_runner);
            tauri::async_runtime::spawn(async move {
                license_refresh_runner_clone.start().await;
            });

            let ai_service = Arc::new(CorvusService::new(
                Arc::clone(&settings),
                Arc::clone(&license_manager),
            ));

            let background_ai_analyzer = Arc::new(BackgroundAiAnalyzer::new(
                db.get_pool().clone(),
                app_handle.clone(),
                Arc::clone(&ai_service),
            ));

            let avatar_providers = settings.get::<Vec<String>>("contacts.avatar.services").ok();
            let background_avatar_fetcher = Arc::new(BackgroundAvatarFetcher::new(
                db.get_pool().clone(),
                app_data_dir_str.clone(),
                avatar_providers,
            ));

            let background_cleanup = Arc::new(BackgroundCleanup::new(
                db.get_pool().clone(),
                app_data_dir_str.clone(),
            ));

            let search_index_dir = app_data_dir.join("search_index");
            let search_manager = Arc::new(
                SearchManager::new(search_index_dir).expect("Failed to initialize search manager"),
            );

            let sync_coordinator = Arc::new(
                app_lib::sync::SyncCoordinator::new(
                    db.get_pool().clone(),
                    app_data_dir_str,
                    Arc::clone(&credential_store),
                )
                .with_search_manager(Arc::clone(&search_manager))
                .with_app_handle(app_handle.clone()),
            );

            let state = AppState {
                db_pool: db.get_pool().clone(),
                settings: Arc::clone(&settings),
                keybindings: Arc::clone(&keybindings),
                ai_service,
                avatar_service: Arc::new(avatar_service),
                oauth_state_manager,
                background_sync_manager: Arc::clone(&background_sync_manager),
                background_body_fetcher: Arc::clone(&background_body_fetcher),
                background_ai_analyzer: Arc::clone(&background_ai_analyzer),
                background_avatar_fetcher: Arc::clone(&background_avatar_fetcher),
                background_cleanup: Arc::clone(&background_cleanup),
                sync_coordinator,
                credential_store,
                search_manager,
                license_manager: Arc::clone(&license_manager),
                license_refresh_runner: Arc::clone(&license_refresh_runner),
                app_handle: app_handle.clone(),
                download_dir: app_handle.path().download_dir().unwrap(),
                app_data_dir: app_handle.path().app_data_dir().unwrap(),
                _config_watcher: _watcher,
                _keybindings_watcher,
            };

            app_handle.manage(state);

            let sync_manager_clone = Arc::clone(&background_sync_manager);
            tauri::async_runtime::spawn(async move {
                match sync_manager_clone.start_all().await {
                    Ok(accounts) => {
                        log::info!("Background sync started for {} accounts", accounts.len());
                    }
                    Err(e) => {
                        log::error!("Failed to start background sync: {}", e);
                    }
                }
            });

            let body_fetcher_clone = Arc::clone(&background_body_fetcher);
            tauri::async_runtime::spawn(async move {
                match body_fetcher_clone.start().await {
                    Ok(_) => {
                        log::info!("Background body fetcher started successfully");
                    }
                    Err(e) => {
                        log::error!("Failed to start background body fetcher: {}", e);
                    }
                }
            });

            let avatar_fetcher_clone = Arc::clone(&background_avatar_fetcher);
            tauri::async_runtime::spawn(async move {
                match avatar_fetcher_clone.start().await {
                    Ok(_) => {
                        log::info!("Background avatar fetcher started successfully");
                    }
                    Err(e) => {
                        log::error!("Failed to start background avatar fetcher: {}", e);
                    }
                }
            });

            let cleanup_clone = Arc::clone(&background_cleanup);
            tauri::async_runtime::spawn(async move {
                match cleanup_clone.start().await {
                    Ok(_) => {
                        log::info!("Background cleanup worker started successfully");
                    }
                    Err(e) => {
                        log::error!("Failed to start background cleanup worker: {}", e);
                    }
                }
            });

            let menu = create_menu(app)?;
            app.set_menu(menu)?;

            app.on_menu_event(|app, event| {
                let menu_id = event.id().as_ref();
                log::debug!("[Menu] Menu event received: {}", menu_id);

                if menu_id.starts_with("ravn://") {
                    log::debug!("[Menu] Emitting navigation event for: {}", menu_id);
                    if let Some(window) = app.get_webview_window("main") {
                        if let Err(e) = window.emit("navigate-to-url", menu_id) {
                            log::error!("[Menu] Failed to emit navigation event: {}", e);
                        }
                    } else {
                        log::error!("[Menu] Main window not found");
                    }
                    return;
                }

                match menu_id {
                    #[cfg(debug_assertions)]
                    "toggle_devtools" => {
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_devtools_open() {
                                window.close_devtools();
                            } else {
                                window.open_devtools();
                            }
                        }
                    }
                    _ => {}
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            corvus::generate_email_completion,
            corvus::ask_ai,
            corvus::generate_search_query,
            corvus::generate_subject,
            corvus::analyze_email_with_ai,
            corvus::get_available_models,
            corvus::get_writing_style,
            corvus::set_writing_style,
            licensing::license_activate,
            licensing::license_trial,
            licensing::license_status,
            licensing::license_validate,
            licensing::license_clear,
            licensing::license_details,
            config::get_setting,
            config::set_setting,
            config::remove_setting,
            config::get_user_keys,
            config::get_all_settings,
            config::set_settings,
            config::reload_settings,
            keybindings_commands::get_keybindings,
            keybindings_commands::get_user_keybindings,
            keybindings_commands::set_keybinding,
            keybindings_commands::remove_keybinding,
            keybindings_commands::reload_keybindings,
            nav_commands::navigate_to_url,
            nav_commands::build_ravn_url,
            nav_commands::open_external_url,
            emails::send_email,
            emails::test_smtp_connection,
            emails::send_email_from_account,
            emails::save_draft,
            emails::get_accounts_for_sending,
            emails::get_drafts,
            emails::delete_draft,
            emails::get_emails,
            emails::get_emails_for_folders,
            emails::get_emails_for_labels,
            emails::update_read,
            emails::email_parse_body_plain,
            emails::move_email,
            emails::archive,
            emails::junk,
            emails::trash,
            emails::delete,
            emails::fetch_body,
            emails::update_blocking,
            folders::get_folder_navigation,
            folders::get_folder,
            folders::get_folders,
            folders::init_folder_sync,
            folders::update_expanded,
            folders::update_hidden,
            folders::move_folder,
            folders::rename,
            folders::update_settings,
            sync::start_oauth2_flow,
            sync::open_oauth_window,
            sync::close_oauth_window,
            sync::exchange_oauth2_code,
            sync::store_imap_credentials,
            sync::sync_account,
            sync::sync_folder,
            sync::open_add_account_window,
            sync::create_account,
            sync::get_accounts,
            sync::delete_account,
            sync::start_background_sync,
            sync::stop_background_sync,
            sync::get_sync_status,
            sync::is_account_syncing,
            contacts::search_contacts,
            contacts::get_top_contacts,
            contacts::get_contacts,
            contacts::get_contact_by_id,
            contacts::get_contact_by_email,
            contacts::create_contact,
            contacts::update_contact,
            contacts::delete_contact,
            contacts::resync_contact_counters,
            attachment::get_email_attachments,
            attachment::open_attachment,
            attachment::quicklook_attachment,
            attachment::save_attachment,
            attachment::get_downloads_path,
            attachment::read_attachment_for_forward,
            attachment::recalculate_attachment_hashes,
            label::get_labels,
            label::get_label,
            label::get_email_labels,
            label::create_label,
            label::update_label,
            label::delete_label,
            label::add_label_to_email,
            label::remove_label_from_email,
            view::get_views,
            view::get_view,
            view::create_view,
            view::update_view,
            view::delete_view,
            conversation::get_conversations_for_folder,
            conversation::get_conversation_for_message_id,
            conversation::get_conversation_by_id,
            search::search_emails,
            search::reindex_all_emails,
            search::reindex_account_emails,
            notification::update_badge_count,
            notification::get_badge_count,
            notification::test_notification_sound,
            themes::list_themes,
            themes::get_theme,
            themes::switch_theme,
            themes::get_current_theme,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
