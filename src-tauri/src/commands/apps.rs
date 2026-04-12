use std::sync::Arc;
use tauri::State;
use crate::models::{AppState, LockedApp};
use crate::services::{scanner, process_win, detailed_scanner};
use crate::utils::config::save_config;

/// Returns the full enriched list of installed applications (registry + Store).
#[tauri::command]
pub async fn get_detailed_apps() -> Result<Vec<detailed_scanner::DetailedApp>, String> {
    detailed_scanner::get_detailed_apps_inner().await
}

/// Returns the full list of installed applications with icons and metadata.
#[tauri::command]
pub async fn get_system_apps() -> Result<Vec<scanner::InstalledApp>, String> {
    Ok(scanner::get_apps())
}

/// Returns the current list of protected (locked) apps from config.
#[tauri::command]
pub async fn get_apps(state: State<'_, Arc<AppState>>) -> Result<Vec<LockedApp>, String> {
    let config = state.config.lock().unwrap();
    Ok(config.locked_apps.clone())
}

/// Persists the updated locked apps list and authorizes already-running instances.
#[tauri::command]
pub async fn save_selection(
    apps: Vec<LockedApp>,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    {
        let mut config = state.config.lock().unwrap();
        config.locked_apps = apps.clone();
        save_config(&config, &state.config_path)?;
    }

    // Authorize currently running instances so the monitor ignores them until next launch
    let processes = process_win::get_processes();
    let mut authorized = state.authorized_pids.lock().unwrap();

    for app in &apps {
        let target_path = app.exec_name.to_lowercase();
        let target_filename = std::path::Path::new(&target_path)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or(&target_path)
            .to_lowercase();
        let target_filename_no_exe = target_filename.strip_suffix(".exe").unwrap_or(&target_filename);

        for process in &processes {
            let proc_path_lower = process.path.to_lowercase();
            let proc_name_lower = process.name.to_lowercase();

            let is_match = proc_path_lower == target_path
                || proc_name_lower == target_filename
                || proc_name_lower == target_filename_no_exe;

            if is_match {
                authorized.insert(process.pid);
            }
        }
    }

    Ok(())
}
