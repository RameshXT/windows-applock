use crate::models::AppState;
use std::sync::Arc;
use tauri::{App, Manager};
pub fn setup_window(app: &mut App, state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window("main") {
        let config = state.config.lock().unwrap();
        let _ = window.set_skip_taskbar(config.stealth_mode.unwrap_or(false));
        drop(config);

        if let Ok(Some(monitor)) = window.primary_monitor() {
            let size = monitor.size();
            let min_width = (size.width as f64 * 0.75) as u32;
            let min_height = (size.height as f64 * 0.80) as u32;
            let _ = window.set_min_size(Some(tauri::Size::Physical(tauri::PhysicalSize::new(
                min_width, min_height,
            ))));
            let mut mws = state.min_window_size.lock().unwrap();
            *mws = (min_width, min_height);
        }
        window.maximize()?;
        window.show()?;
    }
    Ok(())
}
