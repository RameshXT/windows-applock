use crate::app_scanner;
use crate::file_watcher;
use tauri::AppHandle;

#[tauri::command]
pub async fn start_app_scan(app_handle: AppHandle) -> Result<String, String> {
    app_scanner::start_scan_internal(app_handle)
}

#[tauri::command]
pub async fn get_scan_results(
    app_handle: AppHandle,
) -> Result<Vec<app_scanner::ScannedApp>, String> {
    app_scanner::get_cached_results(&app_handle)
}

#[tauri::command]
pub async fn get_scan_status() -> Result<app_scanner::ScanStatus, String> {
    Ok(app_scanner::ScanStatus::Idle)
}

#[tauri::command]
pub async fn refresh_scan(app_handle: AppHandle) -> Result<String, String> {
    app_scanner::start_scan_internal(app_handle)
}

#[tauri::command]
pub async fn start_file_watcher(app_handle: AppHandle) -> Result<(), String> {
    file_watcher::start_file_watcher_internal(app_handle)
}

#[tauri::command]
pub async fn stop_file_watcher() -> Result<(), String> {
    file_watcher::stop_file_watcher_internal()
}
