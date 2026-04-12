use std::sync::Arc;
use tauri::{AppHandle, Manager, State, Emitter};
use crate::models::{AppState, AppConfig, AuthMode, LockedApp};
use crate::services::{scanner, security};
use crate::utils::config::save_config;

#[cfg(target_os = "windows")]
use winreg::{RegKey, enums::*};

#[tauri::command]
pub async fn get_is_unlocked(state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    let unlocked = state.is_unlocked.lock().unwrap();
    Ok(*unlocked)
}

#[tauri::command]
pub async fn get_blocked_app(state: State<'_, Arc<AppState>>) -> Result<Option<LockedApp>, String> {
    let app = state.active_blocked_app.lock().unwrap();
    Ok(app.clone())
}

#[tauri::command]
pub async fn get_system_apps() -> Result<Vec<scanner::InstalledApp>, String> {
    Ok(scanner::get_apps())
}

#[tauri::command]
pub async fn check_setup(state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    let config = state.config.lock().unwrap();
    Ok(!config.hashed_password.is_empty())
}

#[tauri::command]
pub async fn setup_password(password: String, mode: AuthMode, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    config.hashed_password = security::hash_password(&password);
    config.auth_mode = Some(mode);
    save_config(&config, &state.config_path)?;
    Ok(())
}

#[tauri::command]
pub async fn get_config(state: State<'_, Arc<AppState>>) -> Result<AppConfig, String> {
    let config = state.config.lock().unwrap();
    Ok(config.clone())
}

#[tauri::command]
pub async fn update_settings(new_config: AppConfig, state: State<'_, Arc<AppState>>, app_handle: AppHandle) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    
    // If autostart changed, update registry
    if config.autostart != new_config.autostart {
        #[cfg(target_os = "windows")]
        {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            if let Ok(run_key) = hkcu.open_subkey_with_flags("Software\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_WRITE) {
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

    // Handle stealth_mode (Skip taskbar)
    if config.stealth_mode != new_config.stealth_mode {
        if let Some(window) = app_handle.get_webview_window("main") {
            let _ = window.set_skip_taskbar(new_config.stealth_mode.unwrap_or(false));
        }
    }

    *config = new_config;
    save_config(&config, &state.config_path)?;
    Ok(())
}

#[tauri::command]
pub async fn reset_app(state: State<'_, Arc<AppState>>, app_handle: AppHandle) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    *config = AppConfig::default();
    save_config(&config, &state.config_path)?;
    
    let mut unlocked = state.is_unlocked.lock().unwrap();
    *unlocked = false;
    
    // Re-trigger onboarding
    app_handle.emit("reload-app", {}).unwrap();
    Ok(())
}

#[tauri::command]
pub async fn verify_password(password: String, state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    let mut config = state.config.lock().unwrap();
    verify_impl(&password, &mut config, &state)
}

fn verify_impl(password: &str, config: &mut AppConfig, state: &Arc<AppState>) -> Result<bool, String> {
    // All verification now goes through the Argon2 hashing system
    println!("[Auth] Attempting verification for password length: {}", password.len());

    // Standard Lockout check
    if let Some(until) = config.lockout_until {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        if now < until {
            println!("[Auth] Denied: Security lockout active for {} more seconds", until - now);
            return Err(format!("Security lockout active. Try again in {} seconds.", until - now));
        } else {
            println!("[Auth] Lockout expired, resetting attempts");
            config.lockout_until = None;
            config.wrong_attempts = Some(0);
        }
    }

    let is_valid = security::verify_password(password, &config.hashed_password);

    if is_valid {
        println!("[Auth] Code matched. Granting access.");
        let mut unlocked = state.is_unlocked.lock().unwrap();
        *unlocked = true;
        config.wrong_attempts = Some(0);
    } else {
        let attempts = config.wrong_attempts.unwrap_or(0) + 1;
        println!("[Auth] Code MISMATCH. Attempts: {}/{}", attempts, config.attempt_limit.unwrap_or(5));
        config.wrong_attempts = Some(attempts);
        
        let limit = config.attempt_limit.unwrap_or(5);
        if attempts >= limit {
            let duration = config.lockout_duration.unwrap_or(30); 
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
            println!("[Auth] LOCKOUT ACTIVATED for {} seconds", duration);
            config.lockout_until = Some(now + duration as u64);
        }
    }
    
    save_config(config, &state.config_path)?;
    Ok(is_valid)
}

#[tauri::command]
pub async fn verify_gatekeeper(password: String, state: State<'_, Arc<AppState>>, app_handle: AppHandle) -> Result<bool, String> {
    // Get the app to launch BEFORE taking big locks, to avoid deadlocks
    let app_to_launch = {
        let active = state.active_blocked_app.lock().unwrap();
        active.clone()
    };

    // Verify the password — this takes config lock internally
    let is_valid = {
        let mut config = state.config.lock().unwrap();
        verify_impl(&password, &mut config, &state)?
    };
    // config lock is now RELEASED

    if is_valid {
        if let Some(app) = app_to_launch {
            // Set the safety fuse IMMEDIATELY on success
            {
                let mut last_success = state.last_success_time.lock().unwrap();
                *last_success = Some(std::time::Instant::now());
            }

            // Authorize the app path so the monitor ignores it on relaunch
            {
                let mut auth_paths = state.authorized_paths.lock().unwrap();
                let expiry = std::time::Instant::now() + std::time::Duration::from_secs(15);
                auth_paths.insert(app.exec_name.to_lowercase(), expiry);
            }

            // Clear the active blocked app state
            {
                let mut active = state.active_blocked_app.lock().unwrap();
                *active = None;
            }

            // Close the gatekeeper window from Rust — guaranteed, no permission issues
            if let Some(win) = app_handle.get_webview_window("gatekeeper") {
                let _ = win.close();
            }

            // Relaunch the app
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new(&app.exec_name).spawn();
            }
        }
    }
    // If is_valid is false, we do NOTHING — app stays blocked, prompt stays visible

    Ok(is_valid)
}

#[tauri::command]
pub async fn lock_session(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut unlocked = state.is_unlocked.lock().unwrap();
    *unlocked = false;
    Ok(())
}

#[tauri::command]
pub async fn save_selection(apps: Vec<LockedApp>, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    {
        let mut config = state.config.lock().unwrap();
        config.locked_apps = apps.clone();
        save_config(&config, &state.config_path)?;
    }

    // For apps that are currently running when locked, authorize their current session.
    // The monitor will challenge them on next launch (when their PID is gone and a new one appears).
    let processes = crate::services::process_win::get_processes();
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

#[tauri::command]
pub async fn get_apps(state: State<'_, Arc<AppState>>) -> Result<Vec<LockedApp>, String> {
    let config = state.config.lock().unwrap();
    Ok(config.locked_apps.clone())
}

#[tauri::command]
pub async fn release_app(state: State<'_, Arc<AppState>>, app_handle: AppHandle) -> Result<(), String> {
    let mut active = state.active_blocked_app.lock().unwrap();
    *active = None;

    if let Some(win) = app_handle.get_webview_window("gatekeeper") {
        let _ = win.close();
    }

    Ok(())
}
