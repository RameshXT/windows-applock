use notify::{Watcher, RecursiveMode, EventKind};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::AppHandle;
use crate::app_scanner;

pub struct WatcherState {
    pub watcher: Option<notify::RecommendedWatcher>,
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_WATCHER: Arc<Mutex<WatcherState>> = Arc::new(Mutex::new(WatcherState { watcher: None }));
}

pub fn start_file_watcher_internal(app_handle: AppHandle) -> Result<(), String> {
    let mut state = GLOBAL_WATCHER.lock().unwrap();
    if state.watcher.is_some() { return Ok(()); }

    let app_handle_clone = app_handle.clone();
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = notify::RecommendedWatcher::new(tx, notify::Config::default())
        .map_err(|e| e.to_string())?;

    // Watch key directories
    let watch_paths = vec![
        std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string()),
        std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| "C:\\Program Files (x86)".to_string()),
        std::env::var("LOCALAPPDATA").unwrap_or_default(),
    ];

    for path in watch_paths {
        if !path.is_empty() {
            let _ = watcher.watch(Path::new(&path), RecursiveMode::Recursive);
        }
    }

    state.watcher = Some(watcher);

    // Spawn processing thread
    std::thread::spawn(move || {
        let mut last_event_time = std::time::Instant::now();
        let debounce_duration = Duration::from_secs(3);
        let mut pending_event = false;

        loop {
            // Check for new events with a timeout
            if let Ok(Ok(event)) = rx.recv_timeout(Duration::from_millis(500)) {
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) => {
                        // Check if it's an EXE
                        if event.paths.iter().any(|p| p.extension().and_then(|s| s.to_str()) == Some("exe")) {
                            last_event_time = std::time::Instant::now();
                            pending_event = true;
                        }
                    }
                    _ => {}
                }
            }

            // Debounce logic
            if pending_event && last_event_time.elapsed() >= debounce_duration {
                pending_event = false;
                // Trigger rescan
                let _ = app_scanner::start_scan_internal(app_handle_clone.clone());
            }

            // Simple exit check (if watcher is dropped)
            if GLOBAL_WATCHER.lock().unwrap().watcher.is_none() { break; }
        }
    });

    Ok(())
}

pub fn stop_file_watcher_internal() -> Result<(), String> {
    let mut state = GLOBAL_WATCHER.lock().unwrap();
    state.watcher = None; // Dropping the watcher stops it
    Ok(())
}
