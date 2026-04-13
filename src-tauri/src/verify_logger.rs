use crate::rate_limiter::VerifyContext;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::secure_storage::{read_encrypted_internal, write_encrypted_internal};
use tauri::AppHandle;

/// Reasons for a verification failure.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum VerifyFailReason {
    WrongCredential,
    RateLimited,
    Debounced,
    Timeout,
    StorageError,
    InputInvalid,
}

/// A single entry in the verification audit log.
#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyLogEntry {
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub context: VerifyContext,
    pub app_id: Option<String>,
    pub failure_reason: Option<VerifyFailReason>,
    pub attempt_number: u32,
    pub was_rate_limited: bool,
    pub was_debounced: bool,
    pub verification_duration_ms: u64,
}

const LOGS_FILE: &str = "logs.enc";

/// Record a verification attempt to the secure audit log.
/// Implements log rotation to keep only the last 1000 entries.
pub fn record_attempt(app_handle: &AppHandle, entry: VerifyLogEntry) -> Result<(), String> {
    // Load existing logs (ignore error if file doesn't exist, we'll start a new list)
    let mut logs: Vec<VerifyLogEntry> = match read_encrypted_internal(app_handle, LOGS_FILE) {
        Ok(l) => l,
        Err(_) => Vec::new(),
    };
    
    logs.push(entry);
    
    // Log rotation: keep last 1000 entries
    if logs.len() > 1000 {
        let overflow = logs.len() - 1000;
        logs.drain(0..overflow);
    }
    
    // Persist logs back to encrypted storage
    write_encrypted_internal(app_handle, LOGS_FILE, &logs).map_err(|e| e.to_string())
}
