use crate::rate_limiter::{RateLimitState, VerifyContext, check_rate_limit, update_lockout_state, apply_debounce, RateLimitDecision};
use crate::verify_logger::{record_attempt, VerifyLogEntry, VerifyFailReason};
use crate::secure_storage::{read_encrypted_internal, write_encrypted_internal};
use crate::models::state::AppState;
use argon2::{Argon2, password_hash::PasswordHash, password_hash::PasswordVerifier};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::time::{timeout, Duration};

const CREDENTIALS_FILE: &str = "credentials.enc";
const LOCKOUT_FILE: &str = "lockout.enc";

/// Result of a verification attempt, returned to the frontend.
#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyResult {
    pub success: bool,
}

/// Status of the lockout state, for UI purposes.
#[derive(Debug, Serialize, Deserialize)]
pub struct LockoutStatus {
    pub is_locked_out: bool,
    pub seconds_remaining: Option<u64>,
}

/// Internal error types for verification.
#[derive(Debug)]
pub enum VerifyError {
    HashLoadFailed,
    VerifyFailed,
    RateLimited,
    Debounced,
    Timeout,
    StorageError,
    InputInvalid,
    HardLocked,
}

/// Main entry point for credential verification called from the frontend.
#[tauri::command]
pub async fn verify_credential(
    input: String,
    context: String,
    app_id: Option<String>,
    state: State<'_, Arc<AppState>>,
    app_handle: AppHandle,
) -> Result<VerifyResult, String> {
    // 1. Parse and validate context
    let verify_context = VerifyContext::from_str(&context)
        .ok_or_else(|| "Verification failed".to_string())?;

    // 2. Debounce rapid calls (checked BEFORE argon2)
    {
        let mut debounce = state.debounce_state.lock().unwrap();
        if apply_debounce(verify_context, &mut debounce) {
            // Drop call silently, do not log, do not count attempt
            return Err("Verification failed".to_string());
        }
    }

    // 3. Rate limit check (checked BEFORE argon2)
    let rate_limit_reason = {
        let mut rl = state.rate_limit_state.lock().unwrap();
        match check_rate_limit(&mut rl) {
            RateLimitDecision::Allowed => None,
            RateLimitDecision::RateLimited => Some(VerifyFailReason::RateLimited),
            RateLimitDecision::LockedOut(_) => Some(VerifyFailReason::RateLimited),
        }
    };

    if let Some(reason) = rate_limit_reason {
        let rl_state = state.rate_limit_state.lock().unwrap();
        let _ = record_attempt(&app_handle, VerifyLogEntry {
            timestamp: Utc::now(),
            success: false,
            context: verify_context,
            app_id: app_id.clone(),
            failure_reason: Some(reason),
            attempt_number: rl_state.consecutive_failures,
            was_rate_limited: true,
            was_debounced: false,
            verification_duration_ms: 0,
        });
        return Err("Verification failed".to_string());
    }

    // 4. Sanitize input
    let sanitized = input.trim();
    if sanitized.is_empty() || sanitized.len() > 128 || sanitized.contains('\0') {
        let rl_state = state.rate_limit_state.lock().unwrap();
        let _ = record_attempt(&app_handle, VerifyLogEntry {
            timestamp: Utc::now(),
            success: false,
            context: verify_context,
            app_id: app_id.clone(),
            failure_reason: Some(VerifyFailReason::InputInvalid),
            attempt_number: rl_state.consecutive_failures,
            was_rate_limited: false,
            was_debounced: false,
            verification_duration_ms: 0,
        });
        return Err("Verification failed".to_string());
    }

    // 5. Run verification (spawn_blocking with timeout)
    let start_time = Utc::now();
    let verify_future = verify_internal(&app_handle, sanitized);
    let result = timeout(Duration::from_secs(2), verify_future).await;

    let (success, fail_reason) = match result {
        Ok(Ok(true)) => (true, None),
        Ok(Ok(false)) => (false, Some(VerifyFailReason::WrongCredential)),
        Ok(Err(VerifyError::HashLoadFailed)) => (false, Some(VerifyFailReason::StorageError)),
        Ok(Err(_)) => (false, Some(VerifyFailReason::StorageError)),
        Err(_) => (false, Some(VerifyFailReason::Timeout)),
    };

    let duration_ms = (Utc::now() - start_time).num_milliseconds().max(0) as u64;

    // 6. Update and persist rate limit state
    let attempt_number = {
        let mut rl = state.rate_limit_state.lock().unwrap();
        update_lockout_state(success, &mut rl);
        let _ = persist_lockout_state(&app_handle, &rl);
        rl.consecutive_failures
    };

    // 7. Log the attempt
    let _ = record_attempt(&app_handle, VerifyLogEntry {
        timestamp: Utc::now(),
        success,
        context: verify_context,
        app_id,
        failure_reason: fail_reason,
        attempt_number,
        was_rate_limited: false,
        was_debounced: false,
        verification_duration_ms: duration_ms,
    });

    if success {
        Ok(VerifyResult { success: true })
    } else {
        Err("Verification failed".to_string())
    }
}

/// Returns the current lockout status to the frontend.
#[tauri::command]
pub fn get_lockout_status(state: State<'_, Arc<AppState>>) -> Result<LockoutStatus, String> {
    let mut rl = state.rate_limit_state.lock().unwrap();
    match check_rate_limit(&mut rl) {
        RateLimitDecision::LockedOut(remaining) => Ok(LockoutStatus {
            is_locked_out: true,
            seconds_remaining: Some(remaining),
        }),
        _ => {
            if rl.is_locked_out && rl.lockout_until.is_none() {
                // Hard lockout
                Ok(LockoutStatus {
                    is_locked_out: true,
                    seconds_remaining: None,
                })
            } else {
                Ok(LockoutStatus {
                    is_locked_out: false,
                    seconds_remaining: None,
                })
            }
        }
    }
}

/// Reset lockout state after administrative recovery.
#[tauri::command]
pub fn clear_lockout_admin(app_handle: AppHandle, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut rl = state.rate_limit_state.lock().unwrap();
    *rl = RateLimitState::default();
    persist_lockout_state(&app_handle, &rl).map_err(|e| e.to_string())
}

/// Internal verification function using Argon2.
/// Runs in spawn_blocking to avoid blocking the async runtime.
pub async fn verify_internal(app_handle: &AppHandle, input: &str) -> Result<bool, VerifyError> {
    // a) Load stored hash from credentials.enc
    let cred_config: crate::credential_manager::CredentialConfig = read_encrypted_internal(app_handle, CREDENTIALS_FILE)
        .map_err(|_| VerifyError::HashLoadFailed)?;

    let input_bytes = input.as_bytes().to_vec();
    let hash_str = cred_config.hash.clone();

    // c) spawn_blocking -> argon2 verify
    let is_valid = tokio::task::spawn_blocking(move || {
        let parsed_hash = PasswordHash::new(&hash_str).map_err(|_| ())?;
        Argon2::default()
            .verify_password(&input_bytes, &parsed_hash)
            .map(|_| true)
            .map_err(|_| ())
    }).await.map_err(|_| VerifyError::VerifyFailed)?
    .map_err(|_| VerifyError::VerifyFailed)?;

    Ok(is_valid)
}

/// Persist lockout state to encrypted storage.
pub fn persist_lockout_state(app_handle: &AppHandle, state: &RateLimitState) -> Result<(), String> {
    write_encrypted_internal(app_handle, LOCKOUT_FILE, state).map_err(|e| e.to_string())
}

/// Load lockout state from encrypted storage.
pub fn load_lockout_state(app_handle: &AppHandle) -> Result<RateLimitState, String> {
    read_encrypted_internal(app_handle, LOCKOUT_FILE).map_err(|e| e.to_string())
}
