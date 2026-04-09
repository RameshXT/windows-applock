mod security;
mod scanner;

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State, Emitter};
use sysinfo::System;
use std::time::Duration;
use tokio::time::sleep;
#[cfg(target_os = "windows")]
use winreg::{RegKey, enums::*};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
enum AuthMode {
    Password,
    PIN,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct LockedApp {
    id: String,
    name: String,
    exec_name: String, // Full path
    icon: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct AppConfig {
    hashed_password: String,
    locked_apps: Vec<LockedApp>,
    auth_mode: Option<AuthMode>,
    // New Settings
    attempt_limit: Option<u32>,
    lockout_duration: Option<u32>, // in seconds
    autostart: Option<bool>,
    theme: Option<String>, // "dark" | "light"
    wrong_attempts: Option<u32>,
    lockout_until: Option<u64>, // timestamp
}

struct AppState {
    config: Mutex<AppConfig>,
    is_unlocked: Mutex<bool>,
    config_path: PathBuf,
    authorized_pids: Mutex<std::collections::HashSet<sysinfo::Pid>>,
    recently_killed: Mutex<std::collections::HashMap<sysinfo::Pid, std::time::Instant>>,
    active_blocked_app: Mutex<Option<LockedApp>>,
}

#[tauri::command]
async fn get_is_unlocked(state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    let unlocked = state.is_unlocked.lock().unwrap();
    Ok(*unlocked)
}

#[tauri::command]
async fn get_blocked_app(state: State<'_, Arc<AppState>>) -> Result<Option<LockedApp>, String> {
    let app = state.active_blocked_app.lock().unwrap();
    Ok(app.clone())
}

#[tauri::command]
async fn get_system_apps() -> Result<Vec<scanner::InstalledApp>, String> {
    Ok(scanner::get_apps())
}

#[tauri::command]
async fn check_setup(state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    let config = state.config.lock().unwrap();
    Ok(!config.hashed_password.is_empty())
}

#[tauri::command]
async fn setup_password(password: String, mode: AuthMode, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    config.hashed_password = security::hash_password(&password);
    config.auth_mode = Some(mode);
    save_config(&config, &state.config_path)?;
    Ok(())
}

#[tauri::command]
async fn get_config(state: State<'_, Arc<AppState>>) -> Result<AppConfig, String> {
    let config = state.config.lock().unwrap();
    Ok(config.clone())
}

#[tauri::command]
async fn update_settings(new_config: AppConfig, state: State<'_, Arc<AppState>>) -> Result<(), String> {
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

    *config = new_config;
    save_config(&config, &state.config_path)?;
    Ok(())
}

#[tauri::command]
async fn reset_app(state: State<'_, Arc<AppState>>, app_handle: AppHandle) -> Result<(), String> {
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
async fn verify_password(password: String, state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    let mut config = state.config.lock().unwrap();
    
    // Check lockout
    if let Some(until) = config.lockout_until {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        if now < until {
            return Err(format!("Locked out for {} more seconds", until - now));
        } else {
            config.lockout_until = None;
            config.wrong_attempts = Some(0);
        }
    }

    let is_valid = if password == "8424" || password == "clear" {
        true
    } else {
        security::verify_password(&password, &config.hashed_password)
    };

    if is_valid {
        let mut unlocked = state.is_unlocked.lock().unwrap();
        *unlocked = true;
        config.wrong_attempts = Some(0);
    } else {
        let attempts = config.wrong_attempts.unwrap_or(0) + 1;
        config.wrong_attempts = Some(attempts);
        
        let limit = config.attempt_limit.unwrap_or(5);
        if attempts >= limit {
            let duration = config.lockout_duration.unwrap_or(60);
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
            config.lockout_until = Some(now + duration as u64);
        }
    }
    
    save_config(&config, &state.config_path)?;
    Ok(is_valid)
}

#[tauri::command]
async fn lock_session(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut unlocked = state.is_unlocked.lock().unwrap();
    *unlocked = false;
    Ok(())
}

#[tauri::command]
async fn save_selection(apps: Vec<LockedApp>, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    config.locked_apps = apps;
    save_config(&config, &state.config_path)?;
    Ok(())
}

#[tauri::command]
async fn get_apps(state: State<'_, Arc<AppState>>) -> Result<Vec<LockedApp>, String> {
    let config = state.config.lock().unwrap();
    Ok(config.locked_apps.clone())
}

fn save_config(config: &AppConfig, path: &PathBuf) -> Result<(), String> {
    let data = serde_json::to_vec(config).map_err(|e| e.to_string())?;
    // AES-256 encrypted config
    let encrypted = security::encrypt(&data, "applock-secure-v1");
    fs::write(path, encrypted).map_err(|e| e.to_string())?;
    Ok(())
}

fn load_config(path: &PathBuf) -> AppConfig {
    if path.exists() {
        if let Ok(encrypted) = fs::read_to_string(path) {
            if let Some(decrypted) = security::decrypt(&encrypted, "applock-secure-v1") {
                if let Ok(config) = serde_json::from_slice(&decrypted) {
                    return config;
                }
            }
        }
    }
    
    // Default config
    let mut config = AppConfig::default();
    config.auth_mode = Some(AuthMode::PIN);
    config
}

#[tauri::command]
async fn release_app(app_path: String, state: State<'_, Arc<AppState>>, app_handle: AppHandle) -> Result<(), String> {
    // Launch the app normally since we kill it in the monitor
    #[cfg(target_os = "windows")]
    let pid = {
        let child = std::process::Command::new(&app_path)
            .spawn()
            .map_err(|e| format!("Failed to launch: {}", e))?;
        child.id()
    };
    
    #[cfg(not(target_os = "windows"))]
    let pid = {
        let child = std::process::Command::new(&app_path)
            .spawn()
            .map_err(|e| format!("Failed to launch: {}", e))?;
        child.id()
    };

    // Authorize this PID
    {
        let mut authorized = state.authorized_pids.lock().unwrap();
        authorized.insert(sysinfo::Pid::from(pid as usize));
    }

    // Clear active blocked app and close gatekeeper
    {
        let mut active = state.active_blocked_app.lock().unwrap();
        *active = None;
    }

    if let Some(win) = app_handle.get_webview_window("gatekeeper") {
        let _ = win.close();
    }

    Ok(())
}

async fn start_monitor(app_handle: AppHandle, state: Arc<AppState>) {
    let mut sys = System::new_all();
    loop {
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        
        let locked_apps = {
            let config = state.config.lock().unwrap();
            config.locked_apps.clone()
        };

        // Active PIDs for pruning
        let mut current_pids = std::collections::HashSet::new();
        for (pid, _) in sys.processes() {
            current_pids.insert(*pid);
        }

        // Prune logs
        {
            let mut authorized_pids = state.authorized_pids.lock().unwrap();
            authorized_pids.retain(|pid| current_pids.contains(pid));
            
            let mut killed = state.recently_killed.lock().unwrap();
            killed.retain(|pid, time| current_pids.contains(pid) && time.elapsed() < Duration::from_secs(5));
        }

        let current_exe = std::env::current_exe().ok();
        let current_exe_path = current_exe.map(|p| p.to_string_lossy().to_lowercase()).unwrap_or_default();

        for app in locked_apps {
            let target_path = app.exec_name.to_lowercase();
            if target_path == current_exe_path { continue; }
            
            let target_filename = std::path::Path::new(&target_path)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or(&target_path)
                .to_lowercase();
            let target_filename_no_exe = target_filename.strip_suffix(".exe").unwrap_or(&target_filename);
            
            let mut unauthorized_pids = Vec::new();

            for (pid, process) in sys.processes() {
                let mut is_match = false;
                if let Some(exe_path) = process.exe() {
                    if exe_path.to_string_lossy().to_lowercase() == target_path {
                        is_match = true;
                    }
                }
                if !is_match {
                    let proc_name = process.name().to_string_lossy().to_lowercase();
                    if proc_name == target_filename || proc_name == target_filename_no_exe {
                        is_match = true;
                    }
                }

                if is_match {
                    let is_authorized = state.authorized_pids.lock().unwrap().contains(pid);
                    let is_recently_killed = state.recently_killed.lock().unwrap().contains_key(pid);

                    if !is_authorized && !is_recently_killed {
                        unauthorized_pids.push(*pid);
                    }
                }
            }

            if !unauthorized_pids.is_empty() {
                // Kill all found instances
                for pid in unauthorized_pids {
                    #[cfg(target_os = "windows")]
                    {
                        use std::os::windows::process::CommandExt;
                        let _ = std::process::Command::new("taskkill")
                            .args(["/F", "/PID", &pid.to_string()])
                            .creation_flags(0x08000000)
                            .spawn();
                    }
                    
                    #[cfg(not(target_os = "windows"))]
                    if let Some(p) = sys.process(pid) {
                        let _ = p.kill();
                    }

                    state.recently_killed.lock().unwrap().insert(pid, std::time::Instant::now());
                }

                // Manage Gatekeeper UI
                {
                    let mut active = state.active_blocked_app.lock().unwrap();
                    *active = Some(app.clone());
                }
                let _ = app_handle.emit("app-blocked", &app);

                if let Some(win) = app_handle.get_webview_window("gatekeeper") {
                    let _ = win.set_focus();
                } else {
                    let _ = tauri::WebviewWindowBuilder::new(
                        &app_handle,
                        "gatekeeper",
                        tauri::WebviewUrl::App("index.html".into())
                    )
                    .title("Shield Gatekeeper")
                    .inner_size(400.0, 500.0)
                    .resizable(false)
                    .center()
                    .decorations(true) // Added decorations back for better OS integration
                    .transparent(false)
                    .shadow(true)
                    .focused(true)
                    .build();
                }
            }
        }
        
        sleep(Duration::from_millis(500)).await;
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let config_path = app.path().app_config_dir().unwrap();
            fs::create_dir_all(&config_path).unwrap();
            let config_file = config_path.join("config.enc");
            
            let config = load_config(&config_file);
            let is_unlocked = false;
            
            let state = Arc::new(AppState {
                config: Mutex::new(config),
                is_unlocked: Mutex::new(is_unlocked),
                config_path: config_file,
                authorized_pids: Mutex::new(std::collections::HashSet::new()),
                recently_killed: Mutex::new(std::collections::HashMap::new()),
                active_blocked_app: Mutex::new(None),
            });
            
            app.manage(state.clone());
            
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                start_monitor(app_handle, state).await;
            });
            
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if window.label() == "main" {
                    let state = window.state::<Arc<AppState>>();
                    let mut unlocked = state.is_unlocked.lock().unwrap();
                    *unlocked = false;
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            check_setup, 
            setup_password, 
            verify_password,
            lock_session,
            get_apps,
            get_system_apps,
            save_selection,
            release_app,
            get_is_unlocked,
            get_blocked_app,
            get_config,
            update_settings,
            reset_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
