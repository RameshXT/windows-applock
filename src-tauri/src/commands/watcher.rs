use tauri::{AppHandle, State, Emitter};
use crate::lock_session::{LockSessionManager, WatcherState, ActiveLockSession, LockedAppEntry};
use std::sync::Arc;

#[tauri::command]
pub async fn start_watcher(_app_handle: AppHandle, session_manager: State<'_, Arc<LockSessionManager>>) -> Result<(), String> {
    let mut state = session_manager.watcher_state.write().unwrap();
    *state = WatcherState::Running;
    Ok(())
}

#[tauri::command]
pub fn stop_watcher(session_manager: State<'_, Arc<LockSessionManager>>) -> Result<(), String> {
    let mut state = session_manager.watcher_state.write().unwrap();
    *state = WatcherState::Paused;
    Ok(())
}

#[tauri::command]
pub fn pause_watcher(session_manager: State<'_, Arc<LockSessionManager>>) -> Result<(), String> {
    let mut state = session_manager.watcher_state.write().unwrap();
    *state = WatcherState::Paused;
    Ok(())
}

#[tauri::command]
pub fn resume_watcher(session_manager: State<'_, Arc<LockSessionManager>>) -> Result<(), String> {
    let mut state = session_manager.watcher_state.write().unwrap();
    *state = WatcherState::Running;
    Ok(())
}

#[tauri::command]
pub fn get_watcher_state(session_manager: State<'_, Arc<LockSessionManager>>) -> Result<WatcherState, String> {
    Ok(*session_manager.watcher_state.read().unwrap())
}

#[tauri::command]
pub fn get_active_lock_sessions(session_manager: State<'_, Arc<LockSessionManager>>) -> Result<Vec<ActiveLockSession>, String> {
    let sessions = session_manager.active_sessions.read().unwrap();
    Ok(sessions.values().cloned().collect())
}

#[tauri::command]
pub async fn unlock_app(
    app_handle: AppHandle,
    process_id: u32,
    session_manager: State<'_, Arc<LockSessionManager>>
) -> Result<(), String> {
    // Forward to the new restoration logic
    crate::commands::window_management::restore_app_window(
        app_handle.clone(),
        session_manager,
        process_id
    ).await.map(|_| ())?;

    app_handle.emit("app_unlocked", serde_json::json!({
        "process_id": process_id
    })).unwrap();

    Ok(())
}

#[tauri::command]
pub fn add_portable_app(
    exe_path: String,
    session_manager: State<'_, Arc<LockSessionManager>>
) -> Result<LockedAppEntry, String> {
    let path = std::path::Path::new(&exe_path);
    if !path.exists() {
        return Err("Executable path does not exist".to_string());
    }

    let entry = LockedAppEntry {
        id: uuid::Uuid::new_v4().to_string(),
        name: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
        executable_path: exe_path.to_lowercase(),
        executable_name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
        is_uwp: false,
        package_family_name: String::new(),
    };

    let mut locked = session_manager.locked_apps.write().unwrap();
    locked.push(entry.clone());

    Ok(entry)
}
