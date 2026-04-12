pub mod models;
pub mod services;
pub mod commands;
pub mod utils;
pub mod setup;

use std::sync::{Arc, Mutex};
use std::fs;
use tauri::Manager;
use crate::models::AppState;
use crate::utils::config::load_config;
use crate::services::monitor::start_monitor;

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
            });

            app.manage(state.clone());

            // Delegate setup to dedicated modules
            setup::shortcut::register_shortcuts(app, state.clone())?;
            setup::tray::setup_tray(app, state.clone())?;
            setup::window::setup_window(app, state.clone())?;

            // Start the app monitor background task
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                start_monitor(app_handle, state).await;
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
