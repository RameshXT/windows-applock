use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use sysinfo::{System, ProcessesToUpdate};
use tauri::{AppHandle, Manager, Emitter};
use crate::models::AppState;

pub async fn start_monitor(app_handle: AppHandle, state: Arc<AppState>) {
    let mut sys = System::new_all();
    loop {
        sys.refresh_processes(ProcessesToUpdate::All, true);
        
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
                    .decorations(true)
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
