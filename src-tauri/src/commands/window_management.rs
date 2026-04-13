use tauri::{command, AppHandle, Emitter, Manager};
use crate::lock_session::{LockSessionManager, ActiveLockSession, FreezeResult, MonitorInfo};
use crate::window_manager::{get_process_windows, take_window_snapshot, hide_window, suspend_process, resume_process, restore_window_from_snapshot};
use crate::overlay_manager::{enumerate_monitors, position_overlay, assert_topmost, get_window_monitor};
use crate::input_blocker::InputBlocker;
use crate::process_guard::{apply_kill_protection, KillProtectionStatus};
use std::sync::Arc;
use chrono::Utc;
use windows::Win32::UI::WindowsAndMessaging::IsWindowVisible;

#[derive(serde::Serialize)]
pub struct WindowSnapshotView {
    pub app_id: String,
    pub process_id: u32,
    pub original_x: i32,
    pub original_y: i32,
    pub original_width: i32,
    pub original_height: i32,
    pub was_fullscreen: bool,
}

#[command]
pub async fn freeze_app_window(
    app: AppHandle,
    session_manager: tauri::State<'_, Arc<LockSessionManager>>,
    process_id: u32,
    app_id: String,
) -> Result<FreezeResult, String> {
    let hwnds = get_process_windows(process_id);
    if hwnds.is_empty() {
        return Ok(FreezeResult::Failed { reason: "No windows found".to_string() });
    }

    let mut snapshots = Vec::new();
    for hwnd in &hwnds {
        match take_window_snapshot(*hwnd, process_id, &app_id) {
            Ok(snapshot) => {
                let _ = hide_window(*hwnd);
                snapshots.push(snapshot);
            }
            Err(e) => {
                let _ = hide_window(*hwnd);
                println!("Warning: Failed to take snapshot for HWND {:?}: {}", hwnd, e);
            }
        }
    }

    let mut partial = false;
    let mut reason = String::new();

    if let Err(e) = suspend_process(process_id) {
        partial = true;
        reason = e.to_string();
    }

    let monitor_info = if let Some(first_hwnd) = hwnds.first() {
        get_window_monitor(*first_hwnd).ok()
    } else {
        None
    };

    let session = ActiveLockSession {
        app_id: app_id.clone(),
        process_id,
        snapshots,
        detected_at: Utc::now(),
        freeze_applied: true,
        lock_shown: true,
        child_pids: Vec::new(),
        relaunch_count: 0,
        monitor_info: monitor_info.clone(),
    };

    session_manager.add_session(session);

    // Start rehider task
    let pid_clone = process_id;
    let rehider_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
        loop {
            interval.tick().await;
            let hwnds = get_process_windows(pid_clone);
            for hwnd in hwnds {
                unsafe {
                    if IsWindowVisible(hwnd).as_bool() {
                        let _ = hide_window(hwnd);
                    }
                }
            }
        }
    });
    if let Ok(mut tasks) = session_manager.rehider_tasks.write() {
        tasks.insert(process_id, rehider_task);
    }

    // Overlay persistence task
    let app_handle_clone = app.clone();
    let overlay_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(500));
        loop {
            interval.tick().await;
            if let Some(window) = app_handle_clone.get_webview_window("main") {
                if let Ok(hwnd) = window.hwnd() {
                    let _ = assert_topmost(hwnd);
                }
            }
        }
    });
    if let Ok(mut tasks) = session_manager.overlay_tasks.write() {
        tasks.insert(process_id, overlay_task);
    }

    let _ = app.emit("window_frozen", serde_json::json!({
        "app_id": app_id,
        "process_id": process_id,
        "freeze_result": if partial { "PartialSuccess" } else { "Success" }
    }));

    if partial {
        Ok(FreezeResult::PartialSuccess { reason })
    } else {
        Ok(FreezeResult::Success)
    }
}

#[command]
pub async fn restore_app_window(
    _app: AppHandle,
    session_manager: tauri::State<'_, Arc<LockSessionManager>>,
    process_id: u32,
) -> Result<bool, String> {
    let session = match session_manager.remove_session(process_id) {
        Some(s) => s,
        None => return Err("Session not found".to_string()),
    };

    let _ = resume_process(process_id);

    let mut all_success = true;
    for snapshot in &session.snapshots {
        if let Err(e) = restore_window_from_snapshot(snapshot) {
            println!("Error restoring window: {}", e);
            all_success = false;
        }
    }

    Ok(all_success)
}

#[command]
pub async fn get_monitor_layout() -> Result<Vec<MonitorInfo>, String> {
    enumerate_monitors()
}

#[command]
pub async fn get_window_snapshot(
    session_manager: tauri::State<'_, Arc<LockSessionManager>>,
    app_id: String,
) -> Result<WindowSnapshotView, String> {
    let sessions = session_manager.active_sessions.read().unwrap();
    let session = sessions.values().find(|s| s.app_id == app_id)
        .ok_or_else(|| "Session not found".to_string())?;

    let snap = session.snapshots.first()
        .ok_or_else(|| "No snapshots available".to_string())?;

    Ok(WindowSnapshotView {
        app_id: snap.app_id.clone(),
        process_id: snap.process_id,
        original_x: snap.original_x,
        original_y: snap.original_y,
        original_width: snap.original_width,
        original_height: snap.original_height,
        was_fullscreen: snap.was_fullscreen,
    })
}

#[command]
pub async fn get_kill_protection_status() -> Result<KillProtectionStatus, String> {
    apply_kill_protection()
}

#[command]
pub async fn start_input_blocker() -> Result<(), String> {
    InputBlocker::start(Arc::new(std::sync::RwLock::new(true)))
}

#[command]
pub async fn stop_input_blocker() -> Result<(), String> {
    InputBlocker::stop();
    Ok(())
}

#[command]
pub async fn reposition_overlay(
    app: AppHandle,
    monitor_handle: isize,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let monitors = enumerate_monitors()?;
        if let Some(m) = monitors.iter().find(|m_info| m_info.handle == monitor_handle) {
            if let Ok(hwnd) = window.hwnd() {
                let _ = position_overlay(hwnd, m);
                
                let _ = app.emit("lock_overlay_positioned", serde_json::json!({
                    "monitor_index": 0,
                    "x": m.full_rect.left,
                    "y": m.full_rect.top,
                    "width": m.full_rect.right - m.full_rect.left,
                    "height": m.full_rect.bottom - m.full_rect.top,
                    "dpi": m.dpi
                }));
            }
        }
    }
    Ok(())
}
