use crate::models::config::{AppConfig, LockedApp};
use crate::window_manager::{SendHhook, WindowSnapshot};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;

use crate::rate_limiter::{DebounceState, RateLimitState};

use chrono::{DateTime, Utc};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HardLockState {
    pub locked: bool,
    pub locked_at: Option<DateTime<Utc>>,
    pub app_id: String,
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
    pub rate_limit_state: Mutex<RateLimitState>,
    pub debounce_state: Mutex<DebounceState>,
    pub window_snapshots: Arc<RwLock<HashMap<isize, WindowSnapshot>>>,
    pub keyboard_hook: Arc<Mutex<Option<SendHhook>>>,
    pub settings_log: Mutex<Vec<serde_json::Value>>,
    pub session_token: Mutex<Option<String>>,
    pub hard_locks: Mutex<HashMap<String, HardLockState>>,
    pub recovery_fail_counter: Mutex<HashMap<String, (u32, Option<DateTime<Utc>>)>>,
    pub reset_tokens: Mutex<HashMap<String, DateTime<Utc>>>,
}
