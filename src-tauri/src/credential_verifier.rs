use crate::models::state::{AppState, HardLockState};
use crate::rate_limiter::{
    apply_debounce, check_rate_limit, update_lockout_state, RateLimitDecision, RateLimitState,
    VerifyContext,
};
use crate::secure_storage::{read_encrypted_internal, write_encrypted_internal};
use crate::verify_logger::{record_attempt, VerifyAttempt, VerifyFailReason};
use argon2::{password_hash::PasswordHash, password_hash::PasswordVerifier, Argon2};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::time::{timeout, Duration};

const CREDENTIALS_FILE: &str = "credentials.enc";
const LOCKOUT_FILE: &str = "lockout.enc";
#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyResult {
    pub success: bool,
    pub token: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LockoutStatus {
    pub is_locked_out: bool,
    pub seconds_remaining: Option<u64>,
}
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
#[tauri::command]
pub async fn verify_credential(
    input: String,
    context: String,
    app_id: Option<String>,
    state: State<'_, Arc<AppState>>,
    app_handle: AppHandle,
) -> Result<VerifyResult, String> {
    let verify_context =
        VerifyContext::from_str(&context).ok_or_else(|| "Verification failed".to_string())?;
    let target_app_id = app_id
        .clone()
        .unwrap_or_else(|| "dashboard_lock".to_string());
    {
        let hard_locks = state.hard_locks.lock().unwrap();
        if let Some(lock) = hard_locks.get(&target_app_id) {
            if lock.locked {
                return Err("hard_locked".to_string());
            }
        }
    }
    {
        let mut debounce = state.debounce_state.lock().unwrap();
        if apply_debounce(verify_context, &mut debounce) {
            return Err("Verification failed".to_string());
        }
    }
    let rate_limit_reason = {
        let mut rl = state.rate_limit_state.lock().unwrap();
        match check_rate_limit(&mut rl) {
            RateLimitDecision::Allowed => {
                crate::rate_limiter::record_attempt_timestamp(&mut rl);
                None
            }
            RateLimitDecision::RateLimited => Some(VerifyFailReason::RateLimited),
            RateLimitDecision::LockedOut(_) => Some(VerifyFailReason::RateLimited),
        }
    };

    if let Some(reason) = rate_limit_reason {
        let rl_state = state.rate_limit_state.lock().unwrap();
        let _ = record_attempt(
            &app_handle,
            VerifyAttempt {
                timestamp: Utc::now(),
                success: false,
                context: verify_context,
                app_id: app_id.clone(),
                failure_reason: Some(reason),
                attempt_number: rl_state.consecutive_failures,
                was_rate_limited: true,
                was_debounced: false,
                verification_duration_ms: 0,
            },
        );
        return Err("Verification failed".to_string());
    }
    let sanitized = input.trim();
    if sanitized.is_empty() || sanitized.len() > 128 || sanitized.contains('\0') {
        let rl_state = state.rate_limit_state.lock().unwrap();
        let _ = record_attempt(
            &app_handle,
            VerifyAttempt {
                timestamp: Utc::now(),
                success: false,
                context: verify_context,
                app_id: app_id.clone(),
                failure_reason: Some(VerifyFailReason::InputInvalid),
                attempt_number: rl_state.consecutive_failures,
                was_rate_limited: false,
                was_debounced: false,
                verification_duration_ms: 0,
            },
        );
        return Err("Verification failed".to_string());
    }
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
    let attempt_number = {
        let mut rl = state.rate_limit_state.lock().unwrap();
        update_lockout_state(success, &mut rl);
        let _ = persist_lockout_state(&app_handle, &rl);

        if rl.consecutive_failures >= 10 {
            let mut hard_locks = state.hard_locks.lock().unwrap();
            let lock_state = HardLockState {
                locked: true,
                locked_at: Some(Utc::now()),
                app_id: target_app_id.clone(),
            };
            hard_locks.insert(target_app_id.clone(), lock_state);

            let _ = app_handle.emit(
                "hard_lock_active",
                serde_json::json!({
                    "app_id": target_app_id,
                    "locked_at": Utc::now().to_rfc3339()
                }),
            );
        }

        rl.consecutive_failures
    };
    let _ = record_attempt(
        &app_handle,
        VerifyAttempt {
            timestamp: Utc::now(),
            success,
            context: verify_context,
            app_id: app_id.clone(),
            failure_reason: fail_reason,
            attempt_number,
            was_rate_limited: false,
            was_debounced: false,
            verification_duration_ms: duration_ms,
        },
    );

    if success {
        let token = uuid::Uuid::new_v4().to_string();
        {
            let mut s_token = state.session_token.lock().unwrap();
            *s_token = Some(token.clone());
        }

        if let Some(id) = &app_id {
            let app_name = {
                let config = state.config.lock().unwrap();
                config
                    .locked_apps
                    .iter()
                    .find(|a| &a.id == id)
                    .map(|a| a.name.clone())
                    .unwrap_or_else(|| "Unknown App".to_string())
            };

            let handle = app_handle.clone();
            let id_clone = id.clone();
            tauri::async_runtime::spawn(async move {
                let _ =
                    crate::grace_manager::start_grace_session(&id_clone, &app_name, handle).await;
            });
        }
        Ok(VerifyResult {
            success: true,
            token: Some(token),
        })
    } else {
        Err("Verification failed".to_string())
    }
}
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
#[tauri::command]
pub fn clear_lockout_admin(
    app_handle: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut rl = state.rate_limit_state.lock().unwrap();
    *rl = RateLimitState::default();
    persist_lockout_state(&app_handle, &rl).map_err(|e| e.to_string())
}
pub async fn verify_internal(app_handle: &AppHandle, input: &str) -> Result<bool, VerifyError> {
    let cred_config: crate::credential_manager::CredentialConfig =
        read_encrypted_internal(app_handle, CREDENTIALS_FILE)
            .map_err(|_| VerifyError::HashLoadFailed)?;

    let input_bytes = input.as_bytes().to_vec();
    let hash_str = cred_config.hash.clone();
    let is_valid = tokio::task::spawn_blocking(move || {
        let parsed_hash = PasswordHash::new(&hash_str).map_err(|_| ())?;
        Argon2::default()
            .verify_password(&input_bytes, &parsed_hash)
            .map(|_| true)
            .map_err(|_| ())
    })
    .await
    .map_err(|_| VerifyError::VerifyFailed)?
    .map_err(|_| VerifyError::VerifyFailed)?;

    Ok(is_valid)
}
pub fn persist_lockout_state(app_handle: &AppHandle, state: &RateLimitState) -> Result<(), String> {
    write_encrypted_internal(app_handle, LOCKOUT_FILE, state).map_err(|e| e.to_string())
}
pub fn load_lockout_state(app_handle: &AppHandle) -> Result<RateLimitState, String> {
    read_encrypted_internal(app_handle, LOCKOUT_FILE).map_err(|e| e.to_string())
}
