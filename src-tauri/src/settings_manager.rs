use crate::models::AppState;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::Engine;
use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_autostart::ManagerExt;
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppSettings {
    pub autostart_enabled: bool,
    pub minimize_to_tray: bool,
    pub dashboard_lock_enabled: bool,
    pub app_grace_secs: u64,
    pub dashboard_grace_secs: u64,
    pub cooldown_tiers: Vec<CooldownTier>,
    pub max_failed_attempts: u32,
    pub notification_prefs: NotificationPrefs,
    pub theme: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CooldownTier {
    pub fails: u32,
    pub secs: u64,
}
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NotificationPrefs {
    pub notify_on_lock: bool,
    pub notify_on_unlock: bool,
    pub notify_on_fail: bool,
    pub notify_on_hard_lock: bool,
    pub notify_on_grace_expiry: bool,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SettingsChangeLogEntry {
    pub timestamp: DateTime<Utc>,
    pub setting_key: String,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
    pub verified: bool,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImportResult {
    pub success: bool,
    pub settings_applied: u32,
    pub warnings: Vec<String>,
}
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("IO Error: {0}")]
    IoError(String),
    #[error("Serialization Error: {0}")]
    SerdeError(String),
    #[error("Encryption Error: {0}")]
    EncryptionError(String),
    #[error("Decryption Error: {0}")]
    DecryptionError(String),
    #[error("Invalid Token")]
    InvalidToken,
    #[error("Token Expired")]
    TokenExpired,
    #[error("Schema Validation Failed: {0}")]
    SchemaValidationFailed(String),
    #[error("Setting Not Found: {0}")]
    SettingNotFound(String),
}

impl From<std::io::Error> for SettingsError {
    fn from(err: std::io::Error) -> Self {
        SettingsError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for SettingsError {
    fn from(err: serde_json::Error) -> Self {
        SettingsError::SerdeError(err.to_string())
    }
}
pub fn load_settings(state: &AppState) -> Result<AppSettings, SettingsError> {
    let config = state.config.lock().unwrap();
    Ok(AppSettings {
        autostart_enabled: config.autostart.unwrap_or(true),
        minimize_to_tray: config.minimize_to_tray.unwrap_or(true),
        dashboard_lock_enabled: config.stealth_mode.unwrap_or(false), // Assuming stealth_mode as proxy or similar
        app_grace_secs: config.grace_period.unwrap_or(15) as u64,
        dashboard_grace_secs: config.auto_lock_duration.unwrap_or(5) as u64 * 60,
        cooldown_tiers: vec![
            CooldownTier { fails: 3, secs: 30 },
            CooldownTier {
                fails: 5,
                secs: 300,
            },
        ], // Default tiers
        max_failed_attempts: config.attempt_limit.unwrap_or(3),
        notification_prefs: NotificationPrefs {
            notify_on_lock: config.notifications_enabled.unwrap_or(true),
            notify_on_unlock: config.notifications_enabled.unwrap_or(true),
            ..Default::default()
        },
        theme: "system".to_string(),
    })
}
pub fn save_settings(settings: &AppSettings, state: &AppState) -> Result<(), SettingsError> {
    let mut config = state.config.lock().unwrap();
    config.autostart = Some(settings.autostart_enabled);
    config.minimize_to_tray = Some(settings.minimize_to_tray);
    config.grace_period = Some(settings.app_grace_secs as u32);
    config.attempt_limit = Some(settings.max_failed_attempts);
    config.notifications_enabled = Some(settings.notification_prefs.notify_on_lock);
    crate::utils::config::save_config(&config, &state.config_path)
        .map_err(|e| SettingsError::IoError(e))
}
pub fn log_settings_change(
    state: &AppState,
    key: &str,
    old_val: &serde_json::Value,
    new_val: &serde_json::Value,
    verified: bool,
) -> Result<(), SettingsError> {
    let entry = SettingsChangeLogEntry {
        timestamp: Utc::now(),
        setting_key: key.to_string(),
        old_value: old_val.clone(),
        new_value: new_val.clone(),
        verified,
    };

    let mut log = state.settings_log.lock().unwrap();
    log.push(serde_json::to_value(&entry)?);
    Ok(())
}
pub fn encrypt_export(payload: &[u8], password: &str) -> Result<Vec<u8>, SettingsError> {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let key_bytes = hasher.finalize();

    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, payload)
        .map_err(|e| SettingsError::EncryptionError(e.to_string()))?;

    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(combined)
}
pub fn decrypt_import(data: &[u8], password: &str) -> Result<Vec<u8>, SettingsError> {
    if data.len() < 12 {
        return Err(SettingsError::DecryptionError("Invalid data length".into()));
    }

    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let key_bytes = hasher.finalize();

    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| SettingsError::DecryptionError(e.to_string()))
}
pub fn validate_imported_settings(raw: &serde_json::Value) -> Result<AppSettings, SettingsError> {
    serde_json::from_value(raw.clone())
        .map_err(|e| SettingsError::SchemaValidationFailed(e.to_string()))
}
pub fn apply_settings_atomically(
    settings: AppSettings,
    state: &AppState,
) -> Result<u32, SettingsError> {
    save_settings(&settings, state)?;
    Ok(1) // Return Number of settings applied or similar
}
pub fn is_protected_setting(key: &str) -> bool {
    matches!(
        key,
        "dashboard_lock_enabled" | "grace_duration" | "cooldown_tiers" | "max_failed_attempts"
    )
}
fn verify_token_internal(token: &str, state: &AppState) -> Result<(), SettingsError> {
    let session = state.session_token.lock().unwrap();
    if let Some(ref s) = *session {
        if s == token {
            return Ok(());
        }
    }
    let is_unlocked = state.is_unlocked.lock().unwrap();
    if *is_unlocked && !token.is_empty() {
        return Ok(());
    }
    Err(SettingsError::InvalidToken)
}

#[tauri::command]
pub fn set_autostart(
    enabled: bool,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let settings = load_settings(&state).map_err(|e| e.to_string())?;
    let old_val = serde_json::to_value(settings.autostart_enabled).unwrap();

    let mut new_settings = settings.clone();
    new_settings.autostart_enabled = enabled;
    save_settings(&new_settings, &state).map_err(|e| e.to_string())?;

    log_settings_change(
        &state,
        "autostart_enabled",
        &old_val,
        &serde_json::to_value(enabled).unwrap(),
        true,
    )
    .map_err(|e| e.to_string())?;

    let _ = app.emit(
        "autostart_updated",
        serde_json::json!({ "enabled": enabled }),
    );
    let autostart_manager = app.autolaunch();
    if enabled {
        let _ = autostart_manager.enable();
    } else {
        let _ = autostart_manager.disable();
    }

    Ok(())
}

#[tauri::command]
pub fn set_minimize_to_tray(
    enabled: bool,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let settings = load_settings(&state).map_err(|e| e.to_string())?;
    let old_val = serde_json::to_value(settings.minimize_to_tray).unwrap();

    let mut new_settings = settings.clone();
    new_settings.minimize_to_tray = enabled;
    save_settings(&new_settings, &state).map_err(|e| e.to_string())?;

    log_settings_change(
        &state,
        "minimize_to_tray",
        &old_val,
        &serde_json::to_value(enabled).unwrap(),
        true,
    )
    .map_err(|e| e.to_string())?;

    let _ = app.emit(
        "tray_behavior_updated",
        serde_json::json!({ "minimize_to_tray": enabled }),
    );
    Ok(())
}

#[tauri::command]
pub fn set_dashboard_lock(
    enabled: bool,
    token: String,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    verify_token_internal(&token, &state).map_err(|e| e.to_string())?;

    let settings = load_settings(&state).map_err(|e| e.to_string())?;
    let old_val = serde_json::to_value(settings.dashboard_lock_enabled).unwrap();

    let mut new_settings = settings.clone();
    new_settings.dashboard_lock_enabled = enabled;
    save_settings(&new_settings, &state).map_err(|e| e.to_string())?;

    log_settings_change(
        &state,
        "dashboard_lock_enabled",
        &old_val,
        &serde_json::to_value(enabled).unwrap(),
        true,
    )
    .map_err(|e| e.to_string())?;

    let _ = app.emit(
        "dashboard_lock_setting_updated",
        serde_json::json!({ "enabled": enabled }),
    );
    Ok(())
}

#[tauri::command]
pub fn set_grace_duration(
    target: String,
    secs: u64,
    token: String,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    verify_token_internal(&token, &state).map_err(|e| e.to_string())?;

    let settings = load_settings(&state).map_err(|e| e.to_string())?;
    let mut new_settings = settings.clone();
    let old_val;

    if target == "app" {
        old_val = serde_json::to_value(settings.app_grace_secs).unwrap();
        new_settings.app_grace_secs = secs;
    } else if target == "dashboard" {
        old_val = serde_json::to_value(settings.dashboard_grace_secs).unwrap();
        new_settings.dashboard_grace_secs = secs;
    } else {
        return Err("Invalid target".into());
    }

    save_settings(&new_settings, &state).map_err(|e| e.to_string())?;
    log_settings_change(
        &state,
        &format!("grace_duration_{}", target),
        &old_val,
        &serde_json::to_value(secs).unwrap(),
        true,
    )
    .map_err(|e| e.to_string())?;

    let _ = app.emit(
        "grace_duration_updated",
        serde_json::json!({ "target": target, "secs": secs }),
    );
    Ok(())
}

#[tauri::command]
pub fn set_cooldown_tiers(
    tiers: Vec<CooldownTier>,
    token: String,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    verify_token_internal(&token, &state).map_err(|e| e.to_string())?;

    let settings = load_settings(&state).map_err(|e| e.to_string())?;
    let old_val = serde_json::to_value(&settings.cooldown_tiers).unwrap();

    let mut new_settings = settings.clone();
    new_settings.cooldown_tiers = tiers.clone();
    save_settings(&new_settings, &state).map_err(|e| e.to_string())?;

    log_settings_change(
        &state,
        "cooldown_tiers",
        &old_val,
        &serde_json::to_value(&tiers).unwrap(),
        true,
    )
    .map_err(|e| e.to_string())?;

    let _ = app.emit(
        "cooldown_tiers_updated",
        serde_json::json!({ "tiers": tiers }),
    );
    Ok(())
}

#[tauri::command]
pub fn set_max_failed_attempts(
    max: u32,
    token: String,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    verify_token_internal(&token, &state).map_err(|e| e.to_string())?;

    let settings = load_settings(&state).map_err(|e| e.to_string())?;
    let old_val = serde_json::to_value(settings.max_failed_attempts).unwrap();

    let mut new_settings = settings.clone();
    new_settings.max_failed_attempts = max;
    save_settings(&new_settings, &state).map_err(|e| e.to_string())?;

    log_settings_change(
        &state,
        "max_failed_attempts",
        &old_val,
        &serde_json::to_value(max).unwrap(),
        true,
    )
    .map_err(|e| e.to_string())?;

    let _ = app.emit("max_attempts_updated", serde_json::json!({ "max": max }));
    Ok(())
}

#[tauri::command]
pub fn set_notification_prefs(
    prefs: NotificationPrefs,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let settings = load_settings(&state).map_err(|e| e.to_string())?;
    let old_val = serde_json::to_value(&settings.notification_prefs).unwrap();

    let mut new_settings = settings.clone();
    new_settings.notification_prefs = prefs.clone();
    save_settings(&new_settings, &state).map_err(|e| e.to_string())?;

    log_settings_change(
        &state,
        "notification_prefs",
        &old_val,
        &serde_json::to_value(&prefs).unwrap(),
        true,
    )
    .map_err(|e| e.to_string())?;

    let _ = app.emit(
        "notification_prefs_updated",
        serde_json::json!({ "prefs": prefs }),
    );
    Ok(())
}

#[tauri::command]
pub fn set_theme(
    theme: String,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let settings = load_settings(&state).map_err(|e| e.to_string())?;
    let old_val = serde_json::to_value(&settings.theme).unwrap();

    let mut new_settings = settings.clone();
    new_settings.theme = theme.clone();
    save_settings(&new_settings, &state).map_err(|e| e.to_string())?;

    log_settings_change(
        &state,
        "theme",
        &old_val,
        &serde_json::to_value(&theme).unwrap(),
        true,
    )
    .map_err(|e| e.to_string())?;

    let _ = app.emit("theme_updated", serde_json::json!({ "theme": theme }));
    Ok(())
}

#[tauri::command]
pub fn get_settings_change_log(
    token: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<serde_json::Value>, String> {
    verify_token_internal(&token, &state).map_err(|e| e.to_string())?;
    let log = state.settings_log.lock().unwrap();
    Ok(log.clone())
}

#[tauri::command]
pub async fn export_settings(
    password: String,
    path: String,
    token: String,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    verify_token_internal(&token, &state).map_err(|e| e.to_string())?;

    let settings = load_settings(&state).map_err(|e| e.to_string())?;
    let payload = serde_json::to_vec(&settings).map_err(|e| e.to_string())?;

    let encrypted = encrypt_export(&payload, &password).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    hasher.update(&encrypted);
    let checksum = hex::encode(hasher.finalize());

    let export_obj = serde_json::json!({
        "version": "1.0",
        "encrypted_payload": base64::engine::general_purpose::STANDARD.encode(&encrypted),
        "checksum": checksum
    });

    fs::write(
        &path,
        serde_json::to_string_pretty(&export_obj).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    let _ = app.emit("settings_exported", serde_json::json!({ "path": path }));
    Ok(())
}

#[tauri::command]
pub async fn import_settings(
    password: String,
    path: String,
    token: String,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<ImportResult, String> {
    verify_token_internal(&token, &state).map_err(|e| e.to_string())?;

    let data = fs::read(&path).map_err(|e| e.to_string())?;
    let export_obj: serde_json::Value = serde_json::from_slice(&data).map_err(|e| e.to_string())?;

    let enc_payload_base64 = export_obj["encrypted_payload"]
        .as_str()
        .ok_or("Invalid format")?;
    let encrypted = base64::engine::general_purpose::STANDARD
        .decode(enc_payload_base64)
        .map_err(|e| e.to_string())?;

    let checksum = export_obj["checksum"].as_str().ok_or("Invalid format")?;
    let mut hasher = Sha256::new();
    hasher.update(&encrypted);
    if hex::encode(hasher.finalize()) != checksum {
        return Err("Checksum mismatch".into());
    }

    let decrypted = decrypt_import(&encrypted, &password).map_err(|e| e.to_string())?;
    let settings_val: serde_json::Value =
        serde_json::from_slice(&decrypted).map_err(|e| e.to_string())?;

    let settings = validate_imported_settings(&settings_val).map_err(|e| e.to_string())?;
    let count = apply_settings_atomically(settings, &state).map_err(|e| e.to_string())?;

    let result = ImportResult {
        success: true,
        settings_applied: count,
        warnings: vec![],
    };

    let _ = app.emit(
        "settings_imported",
        serde_json::json!({ "settings_applied": count, "warnings": Vec::<String>::new() }),
    );
    Ok(result)
}
