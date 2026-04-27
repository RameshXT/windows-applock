use crate::models::AppState;
use crate::services::process_win::{get_foreground_process_id, get_processes, kill_process};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::sleep;

pub async fn start_monitor(app_handle: AppHandle, state: Arc<AppState>) {
    loop {
        let now = std::time::Instant::now();
        let (grace_duration, auto_lock_mins, notifications_on) = {
            let cfg = state.config.lock().unwrap();
            (
                cfg.grace_period.unwrap_or(0) as u64,
                cfg.auto_lock_duration.unwrap_or(0) as u64,
                cfg.notifications_enabled.unwrap_or(true),
            )
        };
        let mut is_grace_active = false;
        {
            let last_success = state.last_success_time.lock().unwrap();
            if let Some(time) = *last_success {
                let diff = now.duration_since(time);
                if auto_lock_mins > 0 && diff > Duration::from_secs(auto_lock_mins * 60) {
                    let mut unlocked = state.is_unlocked.lock().unwrap();
                    if *unlocked {
                        println!("[Security] Auto-Locking session due to inactivity.");
                        *unlocked = false;
                        if let Some(win) = app_handle.get_webview_window("main") {
                            let _ = win.hide();
                        }
                    }
                }

                if diff < Duration::from_secs(grace_duration).max(Duration::from_secs(5)) {
                    is_grace_active = true;
                }
            }
        }

        if is_grace_active {
            sleep(Duration::from_millis(500)).await;
            continue;
        }
        {
            let mut auth_paths = state.authorized_paths.lock().unwrap();
            auth_paths.retain(|_, expiry| *expiry > now);
        }

        let processes = get_processes();
        let foreground_pid = get_foreground_process_id();
        let running_pids: std::collections::HashSet<u32> =
            processes.iter().map(|p| p.pid).collect();

        {
            let mut authorized = state.authorized_pids.lock().unwrap();
            authorized.retain(|pid| running_pids.contains(pid));

            let mut killed = state.recently_killed.lock().unwrap();
            killed.retain(|pid, time| {
                running_pids.contains(pid) && time.elapsed() < Duration::from_secs(5)
            });
        }

        let current_exe = std::env::current_exe().ok();
        let current_exe_path = current_exe
            .map(|p| p.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let locked_apps = {
            let config = state.config.lock().unwrap();
            config.locked_apps.clone()
        };

        for app in locked_apps {
            let target_path = app.exec_name.to_lowercase();
            if target_path == current_exe_path {
                continue;
            }

            let target_filename = std::path::Path::new(&target_path)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or(&target_path)
                .to_lowercase();
            let target_filename_no_exe = target_filename
                .strip_suffix(".exe")
                .unwrap_or(&target_filename);

            let mut unauthorized_pids = Vec::new();

            for process in &processes {
                let pid = process.pid;
                let proc_path_lower = process.path.to_lowercase();
                let mut is_match = proc_path_lower == target_path;
                if !is_match {
                    let proc_name = process.name.to_lowercase();
                    is_match = proc_name == target_filename || proc_name == target_filename_no_exe;
                }

                if is_match {
                    let is_pid_authorized = state.authorized_pids.lock().unwrap().contains(&pid);
                    let is_path_authorized = {
                        let auth_paths = state.authorized_paths.lock().unwrap();
                        auth_paths.contains_key(&target_path)
                            || auth_paths.contains_key(&proc_path_lower)
                    };

                    if is_pid_authorized || is_path_authorized {
                        if is_path_authorized && !is_pid_authorized {
                            state.authorized_pids.lock().unwrap().insert(pid);
                        }
                        continue;
                    }
                    unauthorized_pids.push(pid);
                }
            }

            let mut is_app_in_foreground = false;
            if foreground_pid != 0 {
                if let Some(fp) = processes.iter().find(|p| p.pid == foreground_pid) {
                    let fp_path = fp.path.to_lowercase();
                    let fp_name = fp.name.to_lowercase();
                    if fp_path == target_path {
                        is_app_in_foreground = true;
                    } else if fp_name == target_filename || fp_name == target_filename_no_exe {
                        is_app_in_foreground = true;
                    } else {
                        let target_dir = std::path::Path::new(&target_path).parent();
                        let fp_dir = std::path::Path::new(&fp_path).parent();

                        if let (Some(td), Some(fd)) = (target_dir, fp_dir) {
                            let td_str = td.to_string_lossy().to_lowercase();
                            let fd_str = fd.to_string_lossy().to_lowercase();
                            if td_str == fd_str && td_str.len() > 15 {
                                is_app_in_foreground = true;
                            }
                        }
                    }
                    if !is_app_in_foreground && fp_name == "applicationframehost.exe" {
                        is_app_in_foreground = unauthorized_pids.len() > 0;
                    }
                }
            }

            if !unauthorized_pids.is_empty() && is_app_in_foreground {
                for pid in &unauthorized_pids {
                    let _ = kill_process(*pid);
                }
                let already_active = {
                    let mut active = state.active_blocked_app.lock().unwrap();
                    let is_same = active.as_ref().map(|a| a.id == app.id).unwrap_or(false);
                    if !is_same && notifications_on {
                        use tauri_plugin_notification::NotificationExt;
                        let _ = app_handle
                            .notification()
                            .builder()
                            .title("Application Blocked")
                            .body(format!(
                                "{} has been restricted by Windows AppLock.",
                                app.name
                            ))
                            .show();
                    }

                    *active = Some(app.clone());
                    is_same
                };

                if let Some(win) = app_handle.get_webview_window("gatekeeper") {
                    if !already_active {
                        let _ = win.unminimize();
                        let _ = win.set_focus();
                        let _ = win.emit_to("gatekeeper", "app-blocked", &app);
                    }
                } else {
                    let _ = tauri::WebviewWindowBuilder::new(
                        &app_handle,
                        "gatekeeper",
                        tauri::WebviewUrl::App("index.html".into()),
                    )
                    .title("Shield Gatekeeper")
                    .inner_size(420.0, 540.0)
                    .resizable(false)
                    .minimizable(true)
                    .closable(true)
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
