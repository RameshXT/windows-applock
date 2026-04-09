use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::path::PathBuf;
use std::collections::{HashSet, HashMap};
use std::time::Instant;
use sysinfo::Pid;

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
    // New Settings
    pub attempt_limit: Option<u32>,
    pub lockout_duration: Option<u32>, // in seconds
    pub autostart: Option<bool>,
    pub theme: Option<String>, // "dark" | "light"
    pub wrong_attempts: Option<u32>,
    pub lockout_until: Option<u64>, // timestamp
}

pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub is_unlocked: Mutex<bool>,
    pub config_path: PathBuf,
    pub authorized_pids: Mutex<HashSet<Pid>>,
    pub recently_killed: Mutex<HashMap<Pid, Instant>>,
    pub active_blocked_app: Mutex<Option<LockedApp>>,
}
