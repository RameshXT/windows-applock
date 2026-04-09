pub mod models;
pub mod services;
pub mod commands;
pub mod utils;

use std::sync::{Arc, Mutex};
use std::fs;
use tauri::Manager;
use crate::models::{AppState};
use crate::utils::config::load_config;
use crate::services::monitor::start_monitor;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let config_path = app.path().app_config_dir().unwrap();
            fs::create_dir_all(&config_path).unwrap();
            let config_file = config_path.join("config.enc");
            
            let config = load_config(&config_file);
            let is_unlocked = false;
            
            let state = Arc::new(AppState {
                config: Mutex::new(config),
                is_unlocked: Mutex::new(is_unlocked),
                config_path: config_file,
                authorized_pids: Mutex::new(std::collections::HashSet::new()),
                recently_killed: Mutex::new(std::collections::HashMap::new()),
                active_blocked_app: Mutex::new(None),
            });
            
            app.manage(state.clone());
            
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                start_monitor(app_handle, state).await;
            });
            
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if window.label() == "main" {
                    let state = window.state::<Arc<AppState>>();
                    let mut unlocked = state.is_unlocked.lock().unwrap();
                    *unlocked = false;
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::check_setup, 
            commands::setup_password, 
            commands::verify_password,
            commands::lock_session,
            commands::get_apps,
            commands::get_system_apps,
            commands::save_selection,
            commands::release_app,
            commands::get_is_unlocked,
            commands::get_blocked_app,
            commands::get_config,
            commands::update_settings,
            commands::reset_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
