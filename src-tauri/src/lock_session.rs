use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LockedAppEntry {
    pub id: String,
    pub name: String,
    pub executable_path: String,
    pub executable_name: String,
    pub is_uwp: bool,
    pub package_family_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

use crate::window_manager::WindowSnapshot;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitorInfo {
    pub handle: isize,
    pub work_area: Rect,
    pub full_rect: Rect,
    pub is_primary: bool,
    pub dpi: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FreezeResult {
    Success,
    PartialSuccess { reason: String },
    Failed { reason: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActiveLockSession {
    pub app_id: String,
    pub process_id: u32,
    pub snapshots: Vec<WindowSnapshot>,
    pub detected_at: DateTime<Utc>,
    pub freeze_applied: bool,
    pub lock_shown: bool,
    pub child_pids: Vec<u32>,
    pub relaunch_count: u32,
    pub monitor_info: Option<MonitorInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum WatcherState {
    Running,
    Paused,
    Crashed,
    Restarting,
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum LockEngineError {
    #[error("Process not found: {0}")]
    ProcessNotFound(u32),
    #[error("Freeze failure: {0}")]
    FreezeFailure(String),
    #[error("Window not found for process {0}")]
    WindowNotFound(u32),
    #[error("Elevation required to lock this app")]
    ElevationRequired,
    #[error("UWP Error: {0}")]
    UwpError(String),
    #[error("WMI Error: {0}")]
    WmiError(String),
    #[error("Watcher crashed: {0}")]
    WatcherCrashed(String),
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Internal Error: {0}")]
    InternalError(String),
}

pub struct LockSessionManager {
    pub active_sessions: Arc<RwLock<HashMap<u32, ActiveLockSession>>>,
    pub locked_apps: Arc<RwLock<Vec<LockedAppEntry>>>,
    pub watcher_state: Arc<RwLock<WatcherState>>,
    pub relaunch_watch: Arc<RwLock<HashMap<String, (u32, DateTime<Utc>)>>>,
    pub rehider_tasks: Arc<RwLock<HashMap<u32, tokio::task::JoinHandle<()>>>>,
    pub overlay_tasks: Arc<RwLock<HashMap<u32, tokio::task::JoinHandle<()>>>>,
}

impl LockSessionManager {
    pub fn new() -> Self {
        Self {
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            locked_apps: Arc::new(RwLock::new(Vec::new())),
            watcher_state: Arc::new(RwLock::new(WatcherState::Paused)),
            relaunch_watch: Arc::new(RwLock::new(HashMap::new())),
            rehider_tasks: Arc::new(RwLock::new(HashMap::new())),
            overlay_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_session(&self, session: ActiveLockSession) {
        let mut sessions = self.active_sessions.write().unwrap();
        sessions.insert(session.process_id, session);
    }

    pub fn remove_session(&self, pid: u32) -> Option<ActiveLockSession> {
        if let Ok(mut tasks) = self.rehider_tasks.write() {
            if let Some(task) = tasks.remove(&pid) {
                task.abort();
            }
        }
        if let Ok(mut tasks) = self.overlay_tasks.write() {
            if let Some(task) = tasks.remove(&pid) {
                task.abort();
            }
        }

        let mut sessions = self.active_sessions.write().unwrap();
        sessions.remove(&pid)
    }

    pub fn is_app_locked(&self, path: &str) -> Option<LockedAppEntry> {
        let locked = self.locked_apps.read().unwrap();
        let path_lower = path.to_lowercase();
        locked
            .iter()
            .find(|a| a.executable_path == path_lower)
            .cloned()
    }
}
