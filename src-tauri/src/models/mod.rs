use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::path::PathBuf;
use std::collections::{HashSet, HashMap};
use std::time::Instant;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum AuthMode {
    Password,
    PIN,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LockedApp {
    pub id: String,
    pub name: String,
    pub exec_name: String, // Full path
    pub icon: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct AppConfig {
    pub hashed_password: String,
    pub locked_apps: Vec<LockedApp>,
    pub auth_mode: Option<AuthMode>,
    pub attempt_limit: Option<u32>,
    pub lockout_duration: Option<u32>, // in seconds
    pub autostart: Option<bool>,
    pub minimize_to_tray: Option<bool>,
    pub stealth_mode: Option<bool>,
    pub notifications_enabled: Option<bool>,
    pub animations_intensity: Option<String>,
    pub autolock_on_sleep: Option<bool>,
    pub wrong_attempts: Option<u32>,
    pub lockout_until: Option<u64>, // timestamp
}

pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub is_unlocked: Mutex<bool>,
    pub config_path: PathBuf,
    pub authorized_pids: Mutex<HashSet<u32>>,
    pub authorized_paths: Mutex<HashMap<String, Instant>>,
    pub last_success_time: Mutex<Option<Instant>>,
    pub recently_killed: Mutex<HashMap<u32, Instant>>,
    pub active_blocked_app: Mutex<Option<LockedApp>>,
    pub min_window_size: Mutex<(u32, u32)>,
    pub was_maximized: Mutex<bool>,
}
