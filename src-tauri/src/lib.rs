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
        .plugin(tauri_plugin_notification::init())
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
                authorized_paths: Mutex::new(std::collections::HashMap::new()),
                last_success_time: Mutex::new(None),
                recently_killed: Mutex::new(std::collections::HashMap::new()),
                active_blocked_app: Mutex::new(None),
                min_window_size: Mutex::new((800, 600)),
                was_maximized: Mutex::new(true),
            });
            
            app.manage(state.clone());
            
            // --- TRAY SETUP ---
            let quit_i = tauri::menu::MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).unwrap();
            let show_i = tauri::menu::MenuItem::with_id(app, "show", "Show App", true, None::<&str>).unwrap();
            let menu = tauri::menu::Menu::with_items(app, &[&show_i, &quit_i]).unwrap();

            let _tray = tauri::tray::TrayIconBuilder::with_id("tray")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "quit" => { app.exit(0); }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { 
                        button: tauri::tray::MouseButton::Left, 
                        button_state: tauri::tray::MouseButtonState::Up, .. 
                    } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;
            // ------------------

            let app_handle = app.handle().clone();
            let monitor_state = state.clone();
            tauri::async_runtime::spawn(async move {
                start_monitor(app_handle, monitor_state).await;
            });

            // Start window in maximized mode with 75% width and 80% height minimum size
            if let Some(window) = app.get_webview_window("main") {
                let config = state.config.lock().unwrap();
                let is_stealth = config.stealth_mode.unwrap_or(false);
                let _ = window.set_skip_taskbar(is_stealth);

                if let Ok(Some(monitor)) = window.primary_monitor() {
                    let size = monitor.size();
                    let min_width = (size.width as f64 * 0.75) as u32;
                    let min_height = (size.height as f64 * 0.80) as u32;
                    let _ = window.set_min_size(Some(tauri::Size::Physical(tauri::PhysicalSize::new(min_width, min_height))));
                    let mut mws = state.min_window_size.lock().unwrap();
                    *mws = (min_width, min_height);
                }
                window.maximize()?;
                window.show()?;
            }
            
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
                        let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize::new(min_size.0, min_size.1)));
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
            commands::reset_app,
            commands::verify_gatekeeper,
            crate::services::detailed_scanner::get_detailed_apps
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
