use crate::models::{AppState, AuthMode};
use crate::services::{auth, security};
use crate::utils::config::save_config;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
#[tauri::command]
pub async fn check_setup(state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    let config = state.config.lock().unwrap();
    let is_onboarded = config.onboarding_completed.unwrap_or(false);
    let has_password = !config.hashed_password.is_empty();

    Ok(is_onboarded && has_password)
}
#[tauri::command]
pub async fn setup_password(
    password: String,
    mode: AuthMode,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    config.hashed_password = security::hash_password(&password);
    config.auth_mode = Some(mode);
    config.last_credential_change = Some(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_millis() as u64,
    );
    save_config(&config, &state.config_path)
}
#[tauri::command]
pub async fn verify_password(
    password: String,
    state: State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    let mut config = state.config.lock().unwrap();
    auth::verify_impl(&password, &mut config, &state)
}
#[tauri::command]
pub async fn verify_gatekeeper(
    password: String,
    state: State<'_, Arc<AppState>>,
    app_handle: AppHandle,
) -> Result<bool, String> {
    let app_to_launch = {
        let active = state.active_blocked_app.lock().unwrap();
        active.clone()
    };

    let is_valid = {
        let mut config = state.config.lock().unwrap();
        auth::verify_impl(&password, &mut config, &state)?
    };

    if is_valid {
        if let Some(app) = app_to_launch {
            {
                let mut last_success = state.last_success_time.lock().unwrap();
                *last_success = Some(std::time::Instant::now());
            }
            {
                let mut auth_paths = state.authorized_paths.lock().unwrap();
                let expiry = std::time::Instant::now() + std::time::Duration::from_secs(15);
                auth_paths.insert(app.exec_name.to_lowercase(), expiry);
            }
            {
                let mut active = state.active_blocked_app.lock().unwrap();
                *active = None;
            }
            if let Some(win) = app_handle.get_webview_window("gatekeeper") {
                let _ = win.close();
            }
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new(&app.exec_name).spawn();
            }
        }
    }

    Ok(is_valid)
}
#[tauri::command]
pub async fn lock_session(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut unlocked = state.is_unlocked.lock().unwrap();
    *unlocked = false;
    Ok(())
}
#[tauri::command]
pub async fn get_is_unlocked(state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    let unlocked = state.is_unlocked.lock().unwrap();
    Ok(*unlocked)
}
