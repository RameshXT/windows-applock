use crate::crypto;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{AppHandle, Manager};

#[derive(Debug, serde::Serialize)]
pub enum StorageError {
    NotFound,
    Tampered(String),
    DecryptionFailed(String),
    WriteFailure(String),
    PermissionDenied(String),
    BackupFailed(String),
    SerializationError(String),
    PathError(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::NotFound => write!(f, "Storage file not found"),
            StorageError::Tampered(e) => write!(f, "Integrity check failed: {}", e),
            StorageError::DecryptionFailed(e) => write!(f, "Decryption failed: {}", e),
            StorageError::WriteFailure(e) => write!(f, "Failed to write file: {}", e),
            StorageError::PermissionDenied(e) => write!(f, "Permission denied (ACL): {}", e),
            StorageError::BackupFailed(e) => write!(f, "Failed to create backup: {}", e),
            StorageError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            StorageError::PathError(e) => write!(f, "Path resolution error: {}", e),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct IntegrityWrapper<T> {
    pub data: T,
    pub checksum: String,
}

#[derive(serde::Serialize)]
pub struct StorageStatus {
    pub credentials_valid: bool,
    pub settings_valid: bool,
    pub apps_valid: bool,
    pub logs_valid: bool,
}

const CREDENTIALS_FILE: &str = "credentials.enc";
const SETTINGS_FILE: &str = "settings.json";
const LOCKED_APPS_FILE: &str = "locked-apps.json";
const LOGS_FILE: &str = "logs.enc";
fn resolve_path(app_handle: &AppHandle, filename: &str) -> Result<PathBuf, StorageError> {
    let dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| StorageError::PathError(e.to_string()))?;

    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| StorageError::WriteFailure(e.to_string()))?;
    }

    Ok(dir.join(filename))
}
pub fn harden_file_permissions(path: &Path) -> Result<(), StorageError> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let status = Command::new("icacls")
            .arg(path)
            .arg("/inheritance:r")
            .arg("/grant:r")
            .arg(format!(
                "{}:F",
                std::env::var("USERNAME").unwrap_or_else(|_| "Users".to_string())
            ))
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .status()
            .map_err(|e| StorageError::PermissionDenied(e.to_string()))?;

        if !status.success() {
            return Err(StorageError::PermissionDenied("icacls failed".to_string()));
        }
    }
    Ok(())
}
pub fn backup_file(path: &Path) -> Result<(), StorageError> {
    if !path.exists() {
        return Ok(());
    }
    let mut bak_path = path.to_path_buf();
    bak_path.set_extension("bak");

    fs::copy(path, &bak_path).map_err(|e| StorageError::BackupFailed(e.to_string()))?;
    harden_file_permissions(&bak_path)?;
    Ok(())
}
pub fn restore_from_backup(path: &Path) -> Result<(), StorageError> {
    let mut bak_path = path.to_path_buf();
    bak_path.set_extension("bak");

    if !bak_path.exists() {
        return Err(StorageError::BackupFailed(
            "Backup file does not exist".to_string(),
        ));
    }

    fs::copy(&bak_path, path).map_err(|e| StorageError::BackupFailed(e.to_string()))?;
    Ok(())
}
fn write_atomic(path: &Path, data: &[u8], is_encrypted: bool) -> Result<(), StorageError> {
    backup_file(path)?;
    let mut tmp_path = path.to_path_buf();
    tmp_path.set_extension("tmp");
    fs::write(&tmp_path, data).map_err(|e| StorageError::WriteFailure(e.to_string()))?;
    harden_file_permissions(&tmp_path)?;
    let verify_result = if is_encrypted {
        crypto::decrypt_with_integrity(&fs::read(&tmp_path).unwrap_or_default())
            .map(|_| ())
            .map_err(|e| StorageError::Tampered(e.to_string()))
    } else {
        let content = fs::read_to_string(&tmp_path).unwrap_or_default();
        serde_json::from_str::<IntegrityWrapper<serde_json::Value>>(&content)
            .map(|wrapper| {
                let actual_check = hex::encode(crypto::calculate_checksum(
                    serde_json::to_string(&wrapper.data).unwrap().as_bytes(),
                ));
                if actual_check != wrapper.checksum {
                    panic!("Integrity fail");
                }
            })
            .map_err(|e| StorageError::Tampered(e.to_string()))
    };

    if verify_result.is_err() {
        let _ = fs::remove_file(&tmp_path);
        return Err(StorageError::WriteFailure(
            "Verification of .tmp failed".to_string(),
        ));
    }
    if let Err(e) = fs::rename(&tmp_path, path) {
        let _ = restore_from_backup(path);
        return Err(StorageError::WriteFailure(e.to_string()));
    }

    harden_file_permissions(path)?;
    Ok(())
}
pub fn read_encrypted_internal<T: DeserializeOwned>(
    app_handle: &AppHandle,
    filename: &str,
) -> Result<T, StorageError> {
    let path = resolve_path(app_handle, filename)?;
    if !path.exists() {
        return Err(StorageError::NotFound);
    }

    let data = fs::read(&path).map_err(|e| StorageError::WriteFailure(e.to_string()))?;
    let decrypted = crypto::decrypt_with_integrity(&data).map_err(|e| match e {
        crypto::CryptoError::IntegrityCheckFailed => {
            StorageError::Tampered("Checksum mismatch".to_string())
        }
        _ => StorageError::DecryptionFailed(e.to_string()),
    })?;

    serde_json::from_slice(&decrypted).map_err(|e| StorageError::SerializationError(e.to_string()))
}
pub fn write_encrypted_internal<T: Serialize>(
    app_handle: &AppHandle,
    filename: &str,
    data: &T,
) -> Result<(), StorageError> {
    let path = resolve_path(app_handle, filename)?;
    let json =
        serde_json::to_vec(data).map_err(|e| StorageError::SerializationError(e.to_string()))?;
    let encrypted = crypto::encrypt_with_integrity(&json)
        .map_err(|e| StorageError::WriteFailure(e.to_string()))?;

    write_atomic(&path, &encrypted, true)
}
pub fn read_json_internal<T: DeserializeOwned + Serialize>(
    app_handle: &AppHandle,
    filename: &str,
) -> Result<T, StorageError> {
    let path = resolve_path(app_handle, filename)?;
    if !path.exists() {
        return Err(StorageError::NotFound);
    }

    let content =
        fs::read_to_string(&path).map_err(|e| StorageError::WriteFailure(e.to_string()))?;
    let wrapper: IntegrityWrapper<T> = serde_json::from_str(&content)
        .map_err(|e| StorageError::SerializationError(e.to_string()))?;
    let data_json = serde_json::to_string(&wrapper.data)
        .map_err(|e| StorageError::SerializationError(e.to_string()))?;
    let actual_checksum = hex::encode(crypto::calculate_checksum(data_json.as_bytes()));

    if actual_checksum != wrapper.checksum {
        return Err(StorageError::Tampered(
            "Plaintext checksum mismatch".to_string(),
        ));
    }

    Ok(wrapper.data)
}
pub fn write_json_internal<T: Serialize + Clone>(
    app_handle: &AppHandle,
    filename: &str,
    data: &T,
) -> Result<(), StorageError> {
    let path = resolve_path(app_handle, filename)?;
    let data_json =
        serde_json::to_string(data).map_err(|e| StorageError::SerializationError(e.to_string()))?;
    let checksum = hex::encode(crypto::calculate_checksum(data_json.as_bytes()));

    let wrapper = IntegrityWrapper {
        data: data.clone(),
        checksum,
    };

    let final_json = serde_json::to_vec(&wrapper)
        .map_err(|e| StorageError::SerializationError(e.to_string()))?;
    write_atomic(&path, &final_json, false)
}

pub async fn verify_storage_integrity_internal(app_handle: AppHandle) -> Result<bool, String> {
    let mut all_valid = true;

    let files = vec![
        (CREDENTIALS_FILE, true),
        (SETTINGS_FILE, false),
        (LOCKED_APPS_FILE, false),
        (LOGS_FILE, true),
    ];

    for (file, is_enc) in files {
        let path = match resolve_path(&app_handle, file) {
            Ok(p) => p,
            Err(_) => {
                all_valid = false;
                continue;
            }
        };

        if !path.exists() {
            continue;
        }

        let valid = if is_enc {
            crypto::decrypt_with_integrity(&fs::read(&path).unwrap_or_default()).is_ok()
        } else {
            let content = fs::read_to_string(&path).unwrap_or_default();
            if let Ok(wrapper) =
                serde_json::from_str::<IntegrityWrapper<serde_json::Value>>(&content)
            {
                let data_json = serde_json::to_string(&wrapper.data).unwrap_or_default();
                hex::encode(crypto::calculate_checksum(data_json.as_bytes())) == wrapper.checksum
            } else {
                false
            }
        };

        if !valid {
            all_valid = false;
            let _ = log_event(
                &app_handle,
                "INTEGRITY_FAILURE",
                &format!("File {} tampered", file),
            )
            .await;
        }
    }

    Ok(all_valid)
}

pub async fn get_storage_status_internal(app_handle: AppHandle) -> Result<StorageStatus, String> {
    let creds = resolve_path(&app_handle, CREDENTIALS_FILE)
        .map(|p| {
            p.exists() && crypto::decrypt_with_integrity(&fs::read(p).unwrap_or_default()).is_ok()
        })
        .unwrap_or(false);
    let settings = resolve_path(&app_handle, SETTINGS_FILE)
        .map(|p| p.exists())
        .unwrap_or(false);
    let apps = resolve_path(&app_handle, LOCKED_APPS_FILE)
        .map(|p| p.exists())
        .unwrap_or(false);
    let logs = resolve_path(&app_handle, LOGS_FILE)
        .map(|p| p.exists())
        .unwrap_or(false);

    Ok(StorageStatus {
        credentials_valid: creds,
        settings_valid: settings,
        apps_valid: apps,
        logs_valid: logs,
    })
}
pub async fn log_event(app_handle: &AppHandle, action: &str, details: &str) -> Result<(), String> {
    let mut logs: Vec<serde_json::Value> =
        read_encrypted_internal(app_handle, LOGS_FILE).unwrap_or_default();
    logs.push(serde_json::json!({
        "action": action,
        "details": details,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));
    write_encrypted_internal(app_handle, LOGS_FILE, &logs).map_err(|e| e.to_string())
}
