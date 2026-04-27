use crate::credential_verifier::verify_internal;
use crate::models::state::AppState;
use crate::secure_storage::{log_event, read_encrypted_internal, write_encrypted_internal};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration as ChronoDuration, Utc};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use uuid::Uuid;

const RECOVERY_FILE: &str = "recovery.enc";
const CREDENTIALS_FILE: &str = "credentials.enc";
const LOCKED_APPS_FILE: &str = "locked-apps.json";
const SETTINGS_FILE: &str = "settings.json";
const LOGS_FILE: &str = "logs.enc";
#[derive(Debug, Serialize, Deserialize)]
pub struct HardLockStatus {
    pub is_locked: bool,
    pub locked_at: Option<String>,
    pub app_id: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct RecoveryResult {
    pub success: bool,
    pub access_restored: bool,
    pub failure_reason: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ResetVerifyResult {
    pub verified: bool,
    pub reset_token: Option<String>,
    pub failure_reason: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ResetResult {
    pub success: bool,
    pub files_deleted: Vec<String>,
    pub registry_cleared: bool,
    pub errors: Vec<String>,
}
#[derive(Debug)]
pub enum RecoveryError {
    #[allow(dead_code)]
    KeyNotFound,
    VerifyFailed,
    HashFailed(String),
    StorageError(String),
    #[allow(dead_code)]
    HardLockNotActive,
    InvalidToken,
    TokenExpired,
    ResetVerifyFailed(String),
    #[allow(dead_code)]
    WipeError(String),
    IoError(String),
}

impl From<RecoveryError> for String {
    fn from(err: RecoveryError) -> Self {
        match err {
            RecoveryError::KeyNotFound => "Recovery key not found".into(),
            RecoveryError::VerifyFailed => "Verification failed".into(),
            RecoveryError::HashFailed(e) => format!("Hash failed: {}", e),
            RecoveryError::StorageError(e) => format!("Storage error: {}", e),
            RecoveryError::HardLockNotActive => "Hard lock not active".into(),
            RecoveryError::InvalidToken => "Invalid reset token".into(),
            RecoveryError::TokenExpired => "Reset token expired".into(),
            RecoveryError::ResetVerifyFailed(e) => format!("Reset verification failed: {}", e),
            RecoveryError::WipeError(e) => format!("Wipe error: {}", e),
            RecoveryError::IoError(e) => format!("I/O error: {}", e),
        }
    }
}
#[tauri::command]
pub fn get_hard_lock_status(
    app_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<HardLockStatus, String> {
    let hard_locks = state.hard_locks.lock().unwrap();
    if let Some(lock) = hard_locks.get(&app_id) {
        Ok(HardLockStatus {
            is_locked: lock.locked,
            locked_at: lock.locked_at.map(|t| t.to_rfc3339()),
            app_id,
        })
    } else {
        Ok(HardLockStatus {
            is_locked: false,
            locked_at: None,
            app_id,
        })
    }
}
#[tauri::command]
pub fn get_new_recovery_key() -> String {
    generate_recovery_key()
}
#[tauri::command]
pub async fn verify_recovery_key(
    input: String,
    app_id: String,
    state: State<'_, Arc<AppState>>,
    app_handle: AppHandle,
) -> Result<RecoveryResult, String> {
    {
        let mut counters = state.recovery_fail_counter.lock().unwrap();
        let counter = counters.entry(app_id.clone()).or_insert((0, None));
        if let Some(until) = counter.1 {
            if Utc::now() < until {
                return Ok(RecoveryResult {
                    success: false,
                    access_restored: false,
                    failure_reason: Some(format!(
                        "Recovery locked. Try again in {} minutes.",
                        (until - Utc::now()).num_minutes() + 1
                    )),
                });
            } else {
                counter.1 = None;
            }
        }
    }
    let success = verify_recovery_key_internal(&input, &app_handle).await;

    if success.is_ok() && success.unwrap() {
        {
            let mut counters = state.recovery_fail_counter.lock().unwrap();
            counters.remove(&app_id);
        }

        let _ = reset_fail_counter(&app_id, &state);
        let _ = clear_hard_lock(&app_id, &state);
        let _ = clear_cooldown(&app_id, &state);
        if app_id != "dashboard_lock" {
            let app_name = {
                let config = state.config.lock().unwrap();
                config
                    .locked_apps
                    .iter()
                    .find(|a| a.id == app_id)
                    .map(|a| a.name.clone())
                    .unwrap_or_else(|| "Unknown App".to_string())
            };
            let _ =
                crate::grace_manager::start_grace_session(&app_id, &app_name, app_handle.clone())
                    .await;
        }
        let _ = log_event(
            &app_handle,
            "recovery_used",
            &format!("Access restored for app: {}", app_id),
        )
        .await;
        let _ = app_handle.emit(
            "recovery_key_verified",
            serde_json::json!({ "app_id": app_id, "success": true }),
        );
        let _ = app_handle.emit(
            "access_restored_via_recovery",
            serde_json::json!({ "app_id": app_id, "restored_at": Utc::now().to_rfc3339() }),
        );
        let _ = app_handle.emit(
            "fail_counter_reset",
            serde_json::json!({ "app_id": app_id }),
        );
        let _ = app_handle.emit("hard_lock_cleared", serde_json::json!({ "app_id": app_id }));

        Ok(RecoveryResult {
            success: true,
            access_restored: true,
            failure_reason: None,
        })
    } else {
        let (attempts, lockout) = {
            let mut counters = state.recovery_fail_counter.lock().unwrap();
            let counter = counters.get_mut(&app_id).unwrap();
            counter.0 += 1;
            if counter.0 >= 3 {
                counter.1 = Some(Utc::now() + ChronoDuration::hours(1));
            }
            (counter.0, counter.1)
        };

        let _ = app_handle.emit(
            "recovery_key_verified",
            serde_json::json!({ "app_id": app_id, "success": false }),
        );

        Ok(RecoveryResult {
            success: false,
            access_restored: false,
            failure_reason: Some(if lockout.is_some() {
                "Maximum attempts reached. 1 hour lockout initiated.".into()
            } else {
                format!("Invalid recovery key. {} of 3 attempts used.", attempts)
            }),
        })
    }
}
#[tauri::command]
pub async fn initiate_full_reset(
    method: String,
    input: String,
    state: State<'_, Arc<AppState>>,
    app_handle: AppHandle,
) -> Result<ResetVerifyResult, String> {
    let verified = if method == "credential" {
        verify_internal(&app_handle, &input).await.unwrap_or(false)
    } else if method == "recovery_key" {
        verify_recovery_key_internal(&input, &app_handle)
            .await
            .unwrap_or(false)
    } else {
        false
    };

    let _ = app_handle.emit(
        "full_reset_initiated",
        serde_json::json!({ "method": method, "verified": verified }),
    );

    if verified {
        let token = generate_reset_token();
        {
            let mut tokens = state.reset_tokens.lock().unwrap();
            tokens.insert(token.clone(), Utc::now() + ChronoDuration::seconds(60));
        }
        Ok(ResetVerifyResult {
            verified: true,
            reset_token: Some(token),
            failure_reason: None,
        })
    } else {
        let mut rl = state.rate_limit_state.lock().unwrap();
        crate::rate_limiter::update_lockout_state(false, &mut rl);
        let _ = crate::credential_verifier::persist_lockout_state(&app_handle, &rl);

        Ok(ResetVerifyResult {
            verified: false,
            reset_token: None,
            failure_reason: Some("Verification failed".into()),
        })
    }
}
#[tauri::command]
pub async fn perform_full_reset(
    token: String,
    state: State<'_, Arc<AppState>>,
    app_handle: AppHandle,
) -> Result<ResetResult, String> {
    if !validate_reset_token(&token, &state) {
        return Err(RecoveryError::InvalidToken.into());
    }
    let result = wipe_all_data(&state, &app_handle);
    let _ = app_handle.emit(
        "full_reset_complete",
        serde_json::json!({
            "files_deleted": result.files_deleted.len(),
            "restarting_onboarding": true
        }),
    );

    Ok(result)
}
#[tauri::command]
pub async fn store_recovery_key_hash(
    raw_key: String,
    _state: State<'_, Arc<AppState>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    hash_and_store_recovery_key(&raw_key, &app_handle)?;

    let _ = app_handle.emit(
        "recovery_key_stored",
        serde_json::json!({ "stored_at": Utc::now().to_rfc3339() }),
    );

    Ok(())
}
pub fn generate_recovery_key() -> String {
    let mut rng = thread_rng();
    let chars: Vec<char> = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect();

    let mut groups = Vec::new();
    for _ in 0..5 {
        let group: String = (0..4)
            .map(|_| {
                let idx = rng.gen_range(0..chars.len());
                chars[idx]
            })
            .collect();
        groups.push(group);
    }

    groups.join("-")
}
fn hash_and_store_recovery_key(raw: &str, app_handle: &AppHandle) -> Result<(), RecoveryError> {
    let normalized = normalize_recovery_key(raw);
    let salt = SaltString::generate(&mut thread_rng());
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(normalized.as_bytes(), &salt)
        .map_err(|e| RecoveryError::HashFailed(e.to_string()))?
        .to_string();

    write_encrypted_internal(app_handle, RECOVERY_FILE, &password_hash)
        .map_err(|e| RecoveryError::StorageError(e.to_string()))
}
async fn verify_recovery_key_internal(
    input: &str,
    app_handle: &AppHandle,
) -> Result<bool, RecoveryError> {
    let normalized = normalize_recovery_key(input);
    let stored_hash: String = read_encrypted_internal(app_handle, RECOVERY_FILE)
        .map_err(|e| RecoveryError::StorageError(e.to_string()))?;

    let is_valid = tokio::task::spawn_blocking(move || {
        let parsed_hash = PasswordHash::new(&stored_hash).map_err(|_| ())?;
        Argon2::default()
            .verify_password(normalized.as_bytes(), &parsed_hash)
            .map(|_| true)
            .map_err(|_| ())
    })
    .await
    .map_err(|_| RecoveryError::VerifyFailed)?
    .unwrap_or(false);

    Ok(is_valid)
}
fn reset_fail_counter(app_id: &str, state: &AppState) -> Result<(), RecoveryError> {
    if app_id == "dashboard_lock" {
        let mut rl = state.rate_limit_state.lock().unwrap();
        rl.consecutive_failures = 0;
        rl.attempt_timestamps.clear();
    }
    Ok(())
}
fn clear_hard_lock(app_id: &str, state: &AppState) -> Result<(), RecoveryError> {
    let mut locks = state.hard_locks.lock().unwrap();
    if let Some(lock) = locks.get_mut(app_id) {
        lock.locked = false;
        lock.locked_at = None;
    }
    Ok(())
}
fn clear_cooldown(app_id: &str, state: &AppState) -> Result<(), RecoveryError> {
    if app_id == "dashboard_lock" {
        let mut rl = state.rate_limit_state.lock().unwrap();
        rl.is_locked_out = false;
        rl.lockout_until = None;
    }
    Ok(())
}
fn wipe_all_data(state: &AppState, app_handle: &AppHandle) -> ResetResult {
    let mut files_deleted = Vec::new();
    let mut errors = Vec::new();

    let data_dir = app_handle.path().app_data_dir().unwrap();
    let files_to_delete = vec![
        CREDENTIALS_FILE,
        RECOVERY_FILE,
        LOCKED_APPS_FILE,
        SETTINGS_FILE,
        LOGS_FILE,
        "lockout.enc",
    ];

    for file in files_to_delete {
        let path = data_dir.join(file);
        if path.exists() {
            if let Err(e) = fs::remove_file(&path) {
                errors.push(format!("Failed to delete {}: {}", file, e));
            } else {
                files_deleted.push(file.to_string());
            }
        }
    }
    let mut registry_cleared = false;
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(key) = hkcu.open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            KEY_WRITE,
        ) {
            if key.delete_value("AppLock").is_ok() {
                registry_cleared = true;
            }
        }
    }
    {
        let mut locks = state.hard_locks.lock().unwrap();
        locks.clear();
    }
    {
        let mut counters = state.recovery_fail_counter.lock().unwrap();
        counters.clear();
    }
    {
        let mut rl = state.rate_limit_state.lock().unwrap();
        *rl = crate::rate_limiter::RateLimitState::default();
    }
    {
        let mut auth_pids = state.authorized_pids.lock().unwrap();
        auth_pids.clear();
    }
    {
        let mut auth_paths = state.authorized_paths.lock().unwrap();
        auth_paths.clear();
    }

    ResetResult {
        success: errors.is_empty(),
        files_deleted,
        registry_cleared,
        errors,
    }
}
fn normalize_recovery_key(input: &str) -> String {
    input.replace("-", "").to_uppercase()
}
fn generate_reset_token() -> String {
    Uuid::new_v4().to_string()
}
fn validate_reset_token(token: &str, state: &AppState) -> bool {
    let mut tokens = state.reset_tokens.lock().unwrap();
    if let Some(expiry) = tokens.get(token) {
        if Utc::now() < *expiry {
            tokens.remove(token);
            return true;
        }
    }
    tokens.remove(token);
    false
}
