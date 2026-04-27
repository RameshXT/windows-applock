use crate::models::AppState;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use chrono::Utc;
use password_hash::rand_core::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use thiserror::Error;
use winreg::enums::*;
use winreg::RegKey;
#[derive(Debug, Serialize, Deserialize)]
pub struct OnboardingPayload {
    pub raw_credential: String,
    pub cred_type: String,
    pub locked_apps: Vec<OnboardingAppEntry>,
    pub settings: OnboardingSettings,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OnboardingAppEntry {
    pub app_id: String,
    pub exe_path: String,
    pub display_name: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OnboardingSettings {
    pub autostart_enabled: bool,
    pub minimize_to_tray: bool,
    pub dashboard_lock_enabled: bool,
    pub app_grace_secs: u64,
    pub dashboard_grace_secs: u64,
    pub max_failed_attempts: u32,
    pub theme: String,
    pub notify_on_lock: bool,
    pub notify_on_unlock: bool,
    pub notify_on_fail: bool,
}
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum CredentialType {
    Pin,
    Password,
}
#[derive(Debug, Default, Clone)]
pub struct FinalizeArtifacts {
    pub credential_written: bool,
    pub apps_written: bool,
    pub settings_written: bool,
    pub autostart_written: bool,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct FinalizeResult {
    pub success: bool,
    pub step_failed: Option<String>,
    pub reason: Option<String>,
    pub rollback_ok: bool,
    pub apps_saved: u32,
    pub stale_apps: u32,
}
#[derive(Debug)]
pub struct SavedAppsResult {
    pub saved: u32,
    pub stale: u32,
}
#[derive(Debug, Error)]
pub enum FinalizeError {
    #[error("Credential Hash Failed: {0}")]
    CredentialHashFailed(String),
    #[error("Credential Write Failed: {0}")]
    CredentialWriteFailed(String),
    #[error("Apps Validation Failed: {0}")]
    AppsValidationFailed(String),
    #[error("Apps Write Failed: {0}")]
    AppsWriteFailed(String),
    #[error("Settings Validation Failed for field {field}: {reason}")]
    SettingsValidationFailed { field: String, reason: String },
    #[error("Settings Write Failed: {0}")]
    SettingsWriteFailed(String),
    #[error("Autostart Registration Failed: {0}")]
    AutostartFailed(String),
    #[error("Onboarding Flag Write Failed: {0}")]
    OnboardingFlagFailed(String),
    #[error("Rollback Failed: {0}")]
    RollbackFailed(String),
    #[error("IO Error: {0}")]
    IoError(String),
}
#[tauri::command]
pub async fn finalize_onboarding(
    payload: OnboardingPayload,
    _state: State<'_, Arc<AppState>>,
    app_handle: AppHandle,
) -> Result<FinalizeResult, String> {
    let mut artifacts = FinalizeArtifacts::default();
    let cred_type = match payload.cred_type.to_lowercase().as_str() {
        "pin" => CredentialType::Pin,
        _ => CredentialType::Password,
    };
    emit_finalize_progress(&app_handle, "Securing credential", "in_progress");
    match store_credential(&payload.raw_credential, cred_type) {
        Ok(_) => {
            artifacts.credential_written = true;
            emit_finalize_progress(&app_handle, "Securing credential", "done");
        }
        Err(e) => return handle_failure(&app_handle, "Securing credential", e, &artifacts).await,
    }
    emit_finalize_progress(&app_handle, "Saving apps", "in_progress");
    let apps_res = match save_locked_apps(payload.locked_apps) {
        Ok(res) => {
            artifacts.apps_written = true;
            emit_finalize_progress(&app_handle, "Saving apps", "done");
            res
        }
        Err(e) => return handle_failure(&app_handle, "Saving apps", e, &artifacts).await,
    };
    emit_finalize_progress(&app_handle, "Saving settings", "in_progress");
    match save_initial_settings(payload.settings.clone()) {
        Ok(_) => {
            artifacts.settings_written = true;
            emit_finalize_progress(&app_handle, "Saving settings", "done");
        }
        Err(e) => return handle_failure(&app_handle, "Saving settings", e, &artifacts).await,
    }
    emit_finalize_progress(&app_handle, "Registering autostart", "in_progress");
    let exe_path = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .to_str()
        .ok_or("Non-UTF8 executable path")?
        .to_string();

    match maybe_register_autostart(payload.settings.autostart_enabled, &exe_path) {
        Ok(_) => {
            if payload.settings.autostart_enabled {
                artifacts.autostart_written = true;
            }
            emit_finalize_progress(&app_handle, "Registering autostart", "done");
            let _ = app_handle.emit(
                "autostart_registered",
                serde_json::json!({ "enabled": payload.settings.autostart_enabled }),
            );
        }
        Err(e) => {
            eprintln!("Warning: Autostart registration failed: {}", e);
            emit_finalize_progress(&app_handle, "Registering autostart", "done");
        }
    }
    emit_finalize_progress(&app_handle, "Finalizing", "in_progress");
    match mark_onboarding_complete() {
        Ok(_) => {
            emit_finalize_progress(&app_handle, "Finalizing", "done");
        }
        Err(e) => return handle_failure(&app_handle, "Finalizing", e, &artifacts).await,
    }
    let result = FinalizeResult {
        success: true,
        step_failed: None,
        reason: None,
        rollback_ok: true,
        apps_saved: apps_res.saved,
        stale_apps: apps_res.stale,
    };

    let _ = app_handle.emit(
        "onboarding_complete",
        serde_json::json!({
            "launch_mode": "onboarding",
            "apps_loaded": apps_res.saved
        }),
    );

    Ok(result)
}
async fn handle_failure(
    app: &AppHandle,
    step: &str,
    error: FinalizeError,
    artifacts: &FinalizeArtifacts,
) -> Result<FinalizeResult, String> {
    let rollback_res = rollback_finalization(artifacts);
    let rollback_ok = rollback_res.is_ok();

    let reason = error.to_string();
    let _ = app.emit(
        "onboarding_finalization_failed",
        serde_json::json!({
            "step": step,
            "reason": reason.clone(),
            "rollback_ok": rollback_ok
        }),
    );

    Ok(FinalizeResult {
        success: false,
        step_failed: Some(step.to_string()),
        reason: Some(reason),
        rollback_ok,
        apps_saved: 0,
        stale_apps: 0,
    })
}
fn emit_finalize_progress(app: &AppHandle, step: &str, status: &str) {
    let _ = app.emit(
        "onboarding_step_progress",
        serde_json::json!({ "step": step, "status": status }),
    );
}
fn store_credential(raw: &str, cred_type: CredentialType) -> Result<(), FinalizeError> {
    let base_dir = get_fallback_config_dir();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(raw.as_bytes(), &salt)
        .map_err(|e| FinalizeError::CredentialHashFailed(e.to_string()))?
        .to_string();

    let data = serde_json::to_vec(&serde_json::json!({
        "hash": password_hash,
        "salt": salt.as_str(),
        "type": format!("{:?}", cred_type),
        "created_at": Utc::now()
    }))
    .map_err(|e| FinalizeError::CredentialWriteFailed(e.to_string()))?;

    let encrypted = encrypt_data(&data, "applock_master_key_iv_protected")
        .map_err(|e| FinalizeError::CredentialWriteFailed(e))?;

    let path = base_dir.join("credentials.enc");
    atomic_write(&path, &encrypted).map_err(|e| FinalizeError::CredentialWriteFailed(e.to_string()))
}
fn save_locked_apps(apps: Vec<OnboardingAppEntry>) -> Result<SavedAppsResult, FinalizeError> {
    let base_dir = get_fallback_config_dir();
    let mut saved = 0;
    let mut stale = 0;
    let mut validated_apps = Vec::new();

    for app in apps {
        let path_exists = Path::new(&app.exe_path).exists();
        if path_exists {
            saved += 1;
        } else {
            stale += 1;
        }

        validated_apps.push(serde_json::json!({
            "id": app.app_id,
            "name": app.display_name,
            "exe_path": app.exe_path,
            "stale": !path_exists
        }));
    }

    let data = serde_json::to_vec_pretty(&validated_apps)
        .map_err(|e| FinalizeError::AppsWriteFailed(e.to_string()))?;

    let path = base_dir.join("locked_apps.json");
    atomic_write(&path, &data).map_err(|e| FinalizeError::AppsWriteFailed(e.to_string()))?;

    Ok(SavedAppsResult { saved, stale })
}
fn save_initial_settings(settings: OnboardingSettings) -> Result<(), FinalizeError> {
    let base_dir = get_fallback_config_dir();
    if settings.app_grace_secs > 3600 {
        return Err(FinalizeError::SettingsValidationFailed {
            field: "app_grace_secs".into(),
            reason: "Grace period cannot exceed 1 hour".into(),
        });
    }
    if settings.max_failed_attempts < 1 || settings.max_failed_attempts > 10 {
        return Err(FinalizeError::SettingsValidationFailed {
            field: "max_failed_attempts".into(),
            reason: "Must be between 1 and 10".into(),
        });
    }

    let data = serde_json::to_vec_pretty(&settings)
        .map_err(|e| FinalizeError::SettingsWriteFailed(e.to_string()))?;

    let path = base_dir.join("settings.json");
    atomic_write(&path, &data).map_err(|e| FinalizeError::SettingsWriteFailed(e.to_string()))
}
fn maybe_register_autostart(enabled: bool, exe_path: &str) -> Result<(), FinalizeError> {
    if !enabled {
        return Ok(());
    }

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
    let (key, _) = hkcu
        .create_subkey(path)
        .map_err(|e| FinalizeError::AutostartFailed(e.to_string()))?;

    key.set_value("AppLock", &format!("\"{}\" --boot-launch", exe_path))
        .map_err(|e| FinalizeError::AutostartFailed(e.to_string()))
}
fn mark_onboarding_complete() -> Result<(), FinalizeError> {
    let base_dir = get_fallback_config_dir();
    let path = base_dir.join("settings.json");
    let content = fs::read_to_string(&path)
        .map_err(|e| FinalizeError::OnboardingFlagFailed(e.to_string()))?;

    let mut settings: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| FinalizeError::OnboardingFlagFailed(e.to_string()))?;

    settings["onboarding_complete"] = serde_json::json!(true);

    let data = serde_json::to_vec_pretty(&settings)
        .map_err(|e| FinalizeError::OnboardingFlagFailed(e.to_string()))?;

    atomic_write(&path, &data).map_err(|e| FinalizeError::OnboardingFlagFailed(e.to_string()))
}
fn rollback_finalization(artifacts: &FinalizeArtifacts) -> Result<(), FinalizeError> {
    let base_dir = get_fallback_config_dir();
    let mut errors = Vec::new();

    if artifacts.credential_written {
        if let Err(e) = fs::remove_file(base_dir.join("credentials.enc")) {
            errors.push(format!("Failed to delete credentials.enc: {}", e));
        }
    }
    if artifacts.apps_written {
        if let Err(e) = fs::remove_file(base_dir.join("locked_apps.json")) {
            errors.push(format!("Failed to delete locked_apps.json: {}", e));
        }
    }
    if artifacts.settings_written {
        if let Err(e) = fs::remove_file(base_dir.join("settings.json")) {
            errors.push(format!("Failed to delete settings.json: {}", e));
        }
    }
    if artifacts.autostart_written {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(key) = hkcu.open_subkey_with_flags(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
            KEY_SET_VALUE,
        ) {
            if let Err(e) = key.delete_value("AppLock") {
                errors.push(format!("Failed to remove registry key: {}", e));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(FinalizeError::RollbackFailed(errors.join("; ")))
    }
}
fn get_fallback_config_dir() -> PathBuf {
    let app_data = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let path = PathBuf::from(app_data).join("com.windows-applock.app");
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    path
}
fn atomic_write<P: AsRef<Path>>(path: P, data: &[u8]) -> std::io::Result<()> {
    let path = path.as_ref();
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, data)?;
    fs::rename(tmp_path, path)
}
fn encrypt_data(data: &[u8], key_str: &str) -> Result<Vec<u8>, String> {
    let mut hasher = Sha256::new();
    hasher.update(key_str.as_bytes());
    let key_bytes = hasher.finalize();
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, data).map_err(|e| e.to_string())?;

    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(combined)
}
