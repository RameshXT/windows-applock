use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VerifyContext {
    #[serde(rename = "app_lock")]
    AppLock,
    #[serde(rename = "dashboard")]
    DashboardLock,
    #[serde(rename = "credential_change")]
    CredentialChange,
    #[serde(rename = "settings")]
    SettingsChange,
}

impl VerifyContext {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "app_lock" => Some(Self::AppLock),
            "dashboard" => Some(Self::DashboardLock),
            "credential_change" => Some(Self::CredentialChange),
            "settings" => Some(Self::SettingsChange),
            _ => None,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RateLimitState {
    pub attempt_timestamps: Vec<DateTime<Utc>>,
    pub is_locked_out: bool,
    pub lockout_until: Option<DateTime<Utc>>,
    pub consecutive_failures: u32,
}

impl Default for RateLimitState {
    fn default() -> Self {
        Self {
            attempt_timestamps: Vec::new(),
            is_locked_out: false,
            lockout_until: None,
            consecutive_failures: 0,
        }
    }
}
pub struct DebounceState {
    pub last_calls: HashMap<VerifyContext, Instant>,
}

impl Default for DebounceState {
    fn default() -> Self {
        Self {
            last_calls: HashMap::new(),
        }
    }
}
pub enum RateLimitDecision {
    Allowed,
    RateLimited,
    LockedOut(u64), // seconds remaining
}
pub fn apply_debounce(context: VerifyContext, state: &mut DebounceState) -> bool {
    let now = Instant::now();
    let min_interval = match context {
        VerifyContext::AppLock => std::time::Duration::from_millis(500),
        VerifyContext::DashboardLock => std::time::Duration::from_millis(1000),
        _ => std::time::Duration::from_millis(500),
    };

    if let Some(&last_call) = state.last_calls.get(&context) {
        if now.duration_since(last_call) < min_interval {
            return true; // Debounced
        }
    }

    state.last_calls.insert(context, now);
    false
}
pub fn check_rate_limit(state: &mut RateLimitState) -> RateLimitDecision {
    let now = Utc::now();
    if state.is_locked_out {
        if let Some(until) = state.lockout_until {
            if now < until {
                let remaining = (until - now).num_seconds().max(0) as u64;
                return RateLimitDecision::LockedOut(remaining);
            } else {
                state.is_locked_out = false;
                state.lockout_until = None;
            }
        } else {
            return RateLimitDecision::LockedOut(0);
        }
    }
    let window_start = now - Duration::seconds(30);
    state.attempt_timestamps.retain(|&t| t > window_start);

    if state.attempt_timestamps.len() >= 5 {
        return RateLimitDecision::RateLimited;
    }
    RateLimitDecision::Allowed
}
pub fn record_attempt_timestamp(state: &mut RateLimitState) {
    state.attempt_timestamps.push(Utc::now());
}
pub fn update_lockout_state(success: bool, state: &mut RateLimitState) {
    let now = Utc::now();

    if success {
        state.consecutive_failures = 0;
        state.is_locked_out = false;
        state.lockout_until = None;
    } else {
        state.consecutive_failures += 1;
        let cooldown = match state.consecutive_failures {
            0..=2 => None,
            3..=4 => Some(Duration::seconds(30)),
            5..=9 => Some(Duration::minutes(5)),
            10..=14 => Some(Duration::minutes(30)),
            _ => None, // Hard lock handled below
        };

        if let Some(duration) = cooldown {
            state.is_locked_out = true;
            state.lockout_until = Some(now + duration);
        } else if state.consecutive_failures >= 15 {
            state.is_locked_out = true;
            state.lockout_until = None; // Hard locked
        }
    }
}
