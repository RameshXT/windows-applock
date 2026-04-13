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
pub mod keyboard_hook;
pub mod uwp_handler;
pub mod watcher_supervisor;
pub mod rate_limiter;
pub mod verify_logger;
pub mod credential_verifier;
pub mod grace_manager;
pub mod system_event_watcher;
pub mod overlay_manager;
pub mod input_blocker;
pub mod process_guard;
pub mod fullscreen_handler;
pub mod settings_manager;
pub mod onboarding_finalizer;
pub mod recovery_manager;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::fs;
use tauri::Manager;
use crate::models::AppState;
use crate::utils::config::load_config;
use crate::lock_session::LockSessionManager;
use crate::process_watcher::ProcessWatcher;
use crate::watcher_supervisor::WatcherSupervisor;
use crate::grace_manager::GraceSessionStore;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--start-minimized"])))
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
                window_snapshots: Arc::new(RwLock::new(HashMap::new())),
                keyboard_hook: Arc::new(Mutex::new(None)),
                settings_log: Mutex::new(Vec::new()),
                session_token: Mutex::new(None),
                hard_locks: Mutex::new(HashMap::new()),
                recovery_fail_counter: Mutex::new(HashMap::new()),
                reset_tokens: Mutex::new(HashMap::new()),
            });

            let session_manager = Arc::new(LockSessionManager::new());
            app.manage(session_manager.clone());

            let grace_store = Arc::new(tokio::sync::RwLock::new(GraceSessionStore::new()));
            app.manage(grace_store.clone());

            app.manage(state.clone());

            // Delegate setup to dedicated modules
            let _ = setup::shortcut::register_shortcuts(app, state.clone());
            let _ = setup::tray::setup_tray(app, state.clone());
            let _ = setup::window::setup_window(app, state.clone());

            // Initialize rehash status check on boot
            credential_manager::initialize_rehash_status(&app.handle());

            // Start system event watcher for grace period resets
            system_event_watcher::start_system_event_watcher(app.handle().clone());

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

            // Grace Period domain
            grace_manager::check_grace_session,
            grace_manager::get_all_grace_sessions,
            grace_manager::re_lock_app,
            grace_manager::re_lock_all,
            grace_manager::get_grace_settings,
            grace_manager::update_grace_settings,
            grace_manager::set_max_security_mode,
            grace_manager::get_max_security_mode,

            // Window Management domain
            window_manager::freeze_target_window,
            window_manager::assert_overlay_topmost,
            window_manager::get_target_monitor_bounds,
            window_manager::restore_locked_window,
            window_manager::install_hook,
            window_manager::uninstall_hook,

            // Settings Management domain
            settings_manager::set_autostart,
            settings_manager::set_minimize_to_tray,
            settings_manager::set_dashboard_lock,
            settings_manager::set_grace_duration,
            settings_manager::set_cooldown_tiers,
            settings_manager::set_max_failed_attempts,
            settings_manager::set_notification_prefs,
            settings_manager::set_theme,
            settings_manager::get_settings_change_log,
            settings_manager::export_settings,
            settings_manager::import_settings,
            // Onboarding domain
            onboarding_finalizer::finalize_onboarding,
            // Recovery domain
            recovery_manager::get_hard_lock_status,
            recovery_manager::get_new_recovery_key,
            recovery_manager::verify_recovery_key,
            recovery_manager::initiate_full_reset,
            recovery_manager::perform_full_reset,
            recovery_manager::store_recovery_key_hash,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
