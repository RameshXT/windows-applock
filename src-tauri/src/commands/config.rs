use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use crate::models::{AppConfig, AppState};
use crate::utils::config::save_config;

#[cfg(target_os = "windows")]
use winreg::{RegKey, enums::*};
#[tauri::command]
pub async fn get_config(state: State<'_, Arc<AppState>>) -> Result<AppConfig, String> {
    let config = state.config.lock().unwrap();
    Ok(config.clone())
}
#[tauri::command]
pub async fn update_settings(
    new_config: AppConfig,
    state: State<'_, Arc<AppState>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();

    // Sync autostart registry entry if changed
    if config.autostart != new_config.autostart {
        #[cfg(target_os = "windows")]
        {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            if let Ok(run_key) = hkcu.open_subkey_with_flags(
                "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                KEY_WRITE,
            ) {
                if new_config.autostart.unwrap_or(false) {
                    if let Ok(exe) = std::env::current_exe() {
                        let path = exe.to_string_lossy().to_string();
                        let _: std::io::Result<()> = run_key.set_value("AppLock", &path);
                    }
                } else {
                    let _: std::io::Result<()> = run_key.delete_value("AppLock");
                }
            }
        }
    }

    // Apply stealth mode (skip taskbar) if changed
    if config.stealth_mode != new_config.stealth_mode {
        if let Some(window) = app_handle.get_webview_window("main") {
            let _ = window.set_skip_taskbar(new_config.stealth_mode.unwrap_or(false));
        }
    }

    // Track credential change timestamp if the hash was modified
    if config.hashed_password != new_config.hashed_password {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_millis() as u64;
        let mut final_config = new_config;
        final_config.last_credential_change = Some(now);
        *config = final_config;
    } else {
        *config = new_config;
    }

    save_config(&config, &state.config_path)
}
#[tauri::command]
pub async fn reset_app(
    state: State<'_, Arc<AppState>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    *config = AppConfig::default();
    save_config(&config, &state.config_path)?;

    let mut unlocked = state.is_unlocked.lock().unwrap();
    *unlocked = false;

    app_handle.emit("reload-app", ()).map_err(|e| e.to_string())
}
