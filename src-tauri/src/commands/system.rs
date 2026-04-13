use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use crate::models::{AppState, LockedApp};
#[tauri::command]
pub async fn get_blocked_app(state: State<'_, Arc<AppState>>) -> Result<Option<LockedApp>, String> {
    let app = state.active_blocked_app.lock().unwrap();
    Ok(app.clone())
}
#[tauri::command]
pub async fn release_app(
    state: State<'_, Arc<AppState>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mut active = state.active_blocked_app.lock().unwrap();
    *active = None;

    if let Some(win) = app_handle.get_webview_window("gatekeeper") {
        let _ = win.close();
    }

    Ok(())
}
