pub mod models;
pub mod services;
pub mod commands;
pub mod utils;
pub mod setup;
pub mod credential_manager;
pub mod crypto;
pub mod secure_storage;
pub mod app_scanner;
pub mod icon_extractor;
pub mod file_watcher;
pub mod lock_session;
pub mod process_watcher;
pub mod window_manager;
pub mod uwp_handler;
pub mod watcher_supervisor;
pub mod rate_limiter;
pub mod verify_logger;
pub mod credential_verifier;

use std::sync::{Arc, Mutex};
use std::fs;
use tauri::Manager;
use crate::models::AppState;
use crate::utils::config::load_config;
use crate::lock_session::LockSessionManager;
use crate::process_watcher::ProcessWatcher;
use crate::watcher_supervisor::WatcherSupervisor;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let config_path = app.path().app_config_dir()?;
            fs::create_dir_all(&config_path)?;
            let config_file = config_path.join("config.enc");
            let config = load_config(&config_file);

            let state = Arc::new(AppState {
                config: Mutex::new(config),
                is_unlocked: Mutex::new(false),
                config_path: config_file,
                authorized_pids: Mutex::new(std::collections::HashSet::new()),
                authorized_paths: Mutex::new(std::collections::HashMap::new()),
                last_success_time: Mutex::new(None),
                recently_killed: Mutex::new(std::collections::HashMap::new()),
                active_blocked_app: Mutex::new(None),
                min_window_size: Mutex::new((800, 600)),
                was_maximized: Mutex::new(true),
                rate_limit_state: Mutex::new(credential_verifier::load_lockout_state(&app.handle()).unwrap_or_default()),
                debounce_state: Mutex::new(rate_limiter::DebounceState::default()),
            });

            let session_manager = Arc::new(LockSessionManager::new());
            app.manage(session_manager.clone());

            app.manage(state.clone());

            // Delegate setup to dedicated modules
            setup::shortcut::register_shortcuts(app, state.clone())?;
            setup::tray::setup_tray(app, state.clone())?;
            setup::window::setup_window(app, state.clone())?;

            // Initialize rehash status check on boot
            credential_manager::initialize_rehash_status(&app.handle());

            // Start the App Lock Engine background tasks
            let watcher_app_handle = app.handle().clone();
            let watcher_session_manager = session_manager.clone();
            tauri::async_runtime::spawn(async move {
               let watcher = ProcessWatcher::new(watcher_app_handle, watcher_session_manager.clone());
               watcher.start_polling().await;
            });

            let supervisor_app_handle = app.handle().clone();
            let supervisor_session_manager = session_manager.clone();
            tauri::async_runtime::spawn(async move {
                let supervisor = WatcherSupervisor::new(supervisor_app_handle, supervisor_session_manager);
                supervisor.run().await;
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Resized(_) = event {
                if window.label() == "main" {
                    let is_max = window.is_maximized().unwrap_or(false);
                    let state = window.state::<Arc<AppState>>();
                    let mut was_max = state.was_maximized.lock().unwrap();
                    if *was_max && !is_max {
                        let min_size = state.min_window_size.lock().unwrap();
                        let _ = window.set_size(tauri::Size::Physical(
                            tauri::PhysicalSize::new(min_size.0, min_size.1),
                        ));
                        let _ = window.center();
                    }
                    *was_max = is_max;
                }
            }
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<Arc<AppState>>();
                if window.label() == "main" {
                    let config = state.config.lock().unwrap();
                    if config.minimize_to_tray.unwrap_or(false) {
                        api.prevent_close();
                        window.hide().unwrap();
                        return;
                    }
                    let mut unlocked = state.is_unlocked.lock().unwrap();
                    *unlocked = false;
                } else if window.label() == "gatekeeper" {
                    let mut active = state.active_blocked_app.lock().unwrap();
                    *active = None;
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            // Auth domain
            commands::auth::check_setup,
            commands::auth::setup_password,
            commands::auth::verify_password,
            commands::auth::verify_gatekeeper,
            commands::auth::lock_session,
            commands::auth::get_is_unlocked,
            // Apps domain
            commands::apps::get_apps,
            commands::apps::get_system_apps,
            commands::apps::get_detailed_apps,
            commands::apps::save_selection,
            // Config domain
            commands::config::get_config,
            commands::config::update_settings,
            commands::config::reset_app,
            // System domain
            commands::system::get_blocked_app,
            commands::system::release_app,
            // Credential domain
            commands::credentials::set_credential,
            credential_verifier::verify_credential,
            commands::credentials::update_credential,
            commands::credentials::get_credential_type,
            commands::credentials::check_rehash_needed,
            credential_verifier::get_lockout_status,
            credential_verifier::clear_lockout_admin,
            // Storage domain
            commands::storage::verify_storage_integrity,
            commands::storage::get_storage_status,
            // Scanner domain
            commands::scanner::start_app_scan,
            commands::scanner::get_scan_results,
            commands::scanner::get_scan_status,
            commands::scanner::refresh_scan,
            commands::scanner::start_file_watcher,
            commands::scanner::stop_file_watcher,
            // Watcher domain
            commands::watcher::start_watcher,
            commands::watcher::stop_watcher,
            commands::watcher::pause_watcher,
            commands::watcher::resume_watcher,
            commands::watcher::get_watcher_state,
            commands::watcher::get_active_lock_sessions,
            commands::watcher::unlock_app,
            commands::watcher::add_portable_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
