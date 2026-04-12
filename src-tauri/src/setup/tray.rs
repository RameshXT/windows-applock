use std::sync::Arc;
use tauri::{App, Manager};
use crate::models::AppState;

/// Builds and registers the system tray icon and menu.
pub fn setup_tray(app: &mut App, _state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    let quit_i = tauri::menu::MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let show_i = tauri::menu::MenuItem::with_id(app, "show", "Show App", true, None::<&str>)?;
    let menu = tauri::menu::Menu::with_items(app, &[&show_i, &quit_i])?;

    tauri::tray::TrayIconBuilder::with_id("tray")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "quit" => app.exit(0),
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

    Ok(())
}
