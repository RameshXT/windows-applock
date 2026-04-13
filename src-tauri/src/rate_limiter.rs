use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// Context of the verification request.
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

/// State for the sliding window rate limiter and hard lockouts.
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

/// State for debouncing rapid verification calls.
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

/// Decision from the rate limiter.
pub enum RateLimitDecision {
    Allowed,
    RateLimited,
    LockedOut(u64), // seconds remaining
}

/// Check if a call should be debounced based on the context.
/// Minimum 500ms for AppLock, 1000ms for DashboardLock.
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

/// Check rate limit and lockout status.
/// Implements a sliding window of max 5 attempts per 30 seconds.
pub fn check_rate_limit(state: &mut RateLimitState) -> RateLimitDecision {
    let now = Utc::now();

    // 1. Check if currently locked out
    if state.is_locked_out {
        if let Some(until) = state.lockout_until {
            if now < until {
                let remaining = (until - now).num_seconds().max(0) as u64;
                return RateLimitDecision::LockedOut(remaining);
            } else {
                // Lockout expired
                state.is_locked_out = false;
                state.lockout_until = None;
                // Note: consecutive_failures is NOT reset here, only on success.
            }
        } else {
            // Hard lock (lockout_until is None)
            return RateLimitDecision::LockedOut(0);
        }
    }

    // 2. Sliding window check: 5 attempts per 30 seconds
    let window_start = now - Duration::seconds(30);
    state.attempt_timestamps.retain(|&t| t > window_start);

    if state.attempt_timestamps.len() >= 5 {
        return RateLimitDecision::RateLimited;
    }

    // Note: We don't push the timestamp here yet, 
    // we do it in record_attempt_timestamp to ensure ONLY allowed calls count.
    RateLimitDecision::Allowed
}

/// Records the timestamp of a permitted verification attempt.
pub fn record_attempt_timestamp(state: &mut RateLimitState) {
    state.attempt_timestamps.push(Utc::now());
}

/// Update lockout state based on verification result.
/// Implements tiered cooldown durations.
pub fn update_lockout_state(success: bool, state: &mut RateLimitState) {
    let now = Utc::now();

    if success {
        state.consecutive_failures = 0;
        state.is_locked_out = false;
        state.lockout_until = None;
    } else {
        state.consecutive_failures += 1;

        // Tiered lockout logic:
        // 3 failures  -> 30s
        // 5 failures  -> 5m
        // 10 failures -> 30m
        // 15 failures -> hard lock
        let cooldown = match state.consecutive_failures {
            0..=2 => None,
            3..=4 => Some(Duration::seconds(30)),
            5..=9 => Some(Duration::minutes(5)),
            10..=14 => Some(Duration::minutes(30)),
            _ => None, // Hard lock handled below
        };

        if let Some(duration) = cooldown {
            // Only update if we aren't already in a stricter lockout
            // (though consecutive_failures increment should handle this naturally)
            state.is_locked_out = true;
            state.lockout_until = Some(now + duration);
        } else if state.consecutive_failures >= 15 {
            state.is_locked_out = true;
            state.lockout_until = None; // Hard locked
        }
    }
}
