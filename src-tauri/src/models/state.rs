use std::sync::Mutex;
use std::path::PathBuf;
use std::collections::{HashSet, HashMap};
use std::time::Instant;
use crate::models::config::{AppConfig, LockedApp};

use crate::rate_limiter::{RateLimitState, DebounceState};

/// Shared application runtime state managed by Tauri's state manager.
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
}
