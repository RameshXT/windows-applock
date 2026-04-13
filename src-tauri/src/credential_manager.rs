use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use chrono::Utc;
use machine_uid;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use lazy_static::lazy_static;

lazy_static! {
    static ref REHASH_NEEDED: Mutex<bool> = Mutex::new(false);
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CredentialConfig {
    pub hash: String,
    pub cred_type: String,
    pub created_at: String,
    pub version: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CredentialLog {
    pub action: String,
    pub timestamp: String,
    pub credential_type: String,
}

const CONFIG_VERSION: u32 = 1;
const CONFIG_FILE: &str = "credentials.enc";
const LOG_FILE: &str = "logs.enc";
fn get_encryption_key() -> Result<[u8; 32], String> {
    let id = machine_uid::get().map_err(|e| format!("Failed to get machine UID: {}", e))?;
    let mut key = [0u8; 32];
    let id_bytes = id.as_bytes();
    for i in 0..32 {
        if i < id_bytes.len() {
            key[i] = id_bytes[i];
        } else {
            key[i] = (i as u8).wrapping_mul(31);
        }
    }
    Ok(key)
}
fn encrypt_data(data: &[u8]) -> Result<Vec<u8>, String> {
    let key_bytes = get_encryption_key()?;
    let key = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| e.to_string())?;
    let nonce_bytes = [0u8; 12]; 
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = key.encrypt(nonce, data).map_err(|e| e.to_string())?;
    Ok(ciphertext)
}
fn decrypt_data(data: &[u8]) -> Result<Vec<u8>, String> {
    let key_bytes = get_encryption_key()?;
    let key = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| e.to_string())?;
    let nonce_bytes = [0u8; 12];
    let nonce = Nonce::from_slice(&nonce_bytes);
    let plaintext = key.decrypt(nonce, data).map_err(|e| e.to_string())?;
    Ok(plaintext)
}

fn get_app_dir(app_handle: &AppHandle) -> Result<PathBuf, String> {
    app_handle
        .path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))
}
fn validate_alphanumeric(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters long".to_string());
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err("Password must contain at least one uppercase letter".to_string());
    }
    Ok(())
}
fn validate_pin(pin: &str, length: usize) -> Result<(), String> {
    if pin.len() != length {
        return Err(format!("PIN must be exactly {} digits", length));
    }
    if !pin.chars().all(|c| c.is_digit(10)) {
        return Err("PIN must contain only digits".to_string());
    }
    let common_pins = vec!["0000", "1111", "2222", "3333", "4444", "5555", "6666", "7777", "8888", "9999", "1234"];
    if length == 4 && common_pins.contains(&pin) {
        return Err("Common PINs are not allowed".to_string());
    }
    let digits: Vec<i32> = pin.chars().map(|c| c.to_digit(10).unwrap() as i32).collect();
    let mut incremental = true;
    let mut decremental = true;

    for i in 1..digits.len() {
        if digits[i] != digits[i - 1] + 1 { incremental = false; }
        if digits[i] != digits[i - 1] - 1 { decremental = false; }
    }

    if incremental || decremental {
        return Err("Sequential PINs are not allowed".to_string());
    }

    Ok(())
}
fn log_action(app_handle: &AppHandle, action: &str, cred_type: &str) -> Result<(), String> {
    let app_dir = get_app_dir(app_handle)?;
    let log_path = app_dir.join(LOG_FILE);
    
    let mut logs: Vec<CredentialLog> = if log_path.exists() {
        let encrypted = fs::read(&log_path).map_err(|e| e.to_string())?;
        let decrypted = decrypt_data(&encrypted)?;
        serde_json::from_slice(&decrypted).unwrap_or_default()
    } else {
        Vec::new()
    };

    logs.push(CredentialLog {
        action: action.to_string(),
        timestamp: Utc::now().to_rfc3339(),
        credential_type: cred_type.to_string(),
    });

    let json = serde_json::to_vec(&logs).map_err(|e| e.to_string())?;
    let encrypted = encrypt_data(&json)?;
    fs::write(log_path, encrypted).map_err(|e| e.to_string())?;

    Ok(())
}
pub fn set_credential_internal(app_handle: &AppHandle, pin_or_password: String, cred_type: String) -> Result<(), String> {
    match cred_type.as_str() {
        "pin_4" => validate_pin(&pin_or_password, 4)?,
        "pin_6" => validate_pin(&pin_or_password, 6)?,
        "alphanumeric" => validate_alphanumeric(&pin_or_password)?,
        _ => return Err("Invalid credential type".to_string()),
    }

    let salt = SaltString::generate(&mut rand::thread_rng());
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(pin_or_password.as_bytes(), &salt)
        .map_err(|e| e.to_string())?
        .to_string();

    let config = CredentialConfig {
        hash: password_hash,
        cred_type: cred_type.clone(),
        created_at: Utc::now().to_rfc3339(),
        version: CONFIG_VERSION,
    };

    let app_dir = get_app_dir(app_handle)?;
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    }

    let json = serde_json::to_vec(&config).map_err(|e| e.to_string())?;
    let encrypted = encrypt_data(&json)?;
    fs::write(app_dir.join(CONFIG_FILE), encrypted).map_err(|e| e.to_string())?;

    log_action(app_handle, "set_credential", &cred_type)?;

    Ok(())
}
pub fn verify_credential_internal(app_handle: &AppHandle, input: String) -> Result<bool, String> {
    let app_dir = get_app_dir(app_handle)?;
    let config_path = app_dir.join(CONFIG_FILE);

    if !config_path.exists() {
        return Ok(false);
    }

    let encrypted = fs::read(&config_path).map_err(|e| e.to_string())?;
    let decrypted = decrypt_data(&encrypted)?;
    let mut config: CredentialConfig = serde_json::from_slice(&decrypted).map_err(|e| e.to_string())?;

    let parsed_hash = PasswordHash::new(&config.hash).map_err(|e| e.to_string())?;
    let is_valid = Argon2::default()
        .verify_password(input.as_bytes(), &parsed_hash)
        .is_ok();

    if is_valid {
        let mut rehash_needed = REHASH_NEEDED.lock().unwrap();
        if *rehash_needed || config.version < CONFIG_VERSION {
            let salt = SaltString::generate(&mut rand::thread_rng());
            let new_hash = Argon2::default()
                .hash_password(input.as_bytes(), &salt)
                .map_err(|e| e.to_string())?
                .to_string();

            config.hash = new_hash;
            config.version = CONFIG_VERSION;

            let json = serde_json::to_vec(&config).map_err(|e| e.to_string())?;
            let encrypted = encrypt_data(&json)?;
            fs::write(config_path, encrypted).map_err(|e| e.to_string())?;
            
            *rehash_needed = false;
        }
    }

    Ok(is_valid)
}
pub fn update_credential_internal(app_handle: &AppHandle, old_input: String, new_input: String, cred_type: String) -> Result<(), String> {
    let is_valid = verify_credential_internal(app_handle, old_input)?;
    if !is_valid {
        return Err("Current credential verification failed".to_string());
    }

    set_credential_internal(app_handle, new_input, cred_type.clone())?;
    log_action(app_handle, "update_credential", &cred_type)?;
    
    Ok(())
}
pub fn initialize_rehash_status(app_handle: &AppHandle) {
    if let Ok(app_dir) = get_app_dir(app_handle) {
        let config_path = app_dir.join(CONFIG_FILE);
        if config_path.exists() {
            if let Ok(encrypted) = fs::read(&config_path) {
                if let Ok(decrypted) = decrypt_data(&encrypted) {
                    if let Ok(config) = serde_json::from_slice::<CredentialConfig>(&decrypted) {
                        if config.version < CONFIG_VERSION {
                            let mut rehash = REHASH_NEEDED.lock().unwrap();
                            *rehash = true;
                        }
                    }
                }
            }
        }
    }
}

pub fn get_rehash_needed() -> bool {
    *REHASH_NEEDED.lock().unwrap()
}

pub fn get_credential_type_internal(app_handle: &AppHandle) -> Result<String, String> {
    let app_dir = get_app_dir(app_handle)?;
    let config_path = app_dir.join(CONFIG_FILE);

    if !config_path.exists() {
        return Ok("none".to_string());
    }

    let encrypted = fs::read(&config_path).map_err(|e| e.to_string())?;
    let decrypted = decrypt_data(&encrypted)?;
    let config: CredentialConfig = serde_json::from_slice(&decrypted).map_err(|e| e.to_string())?;

    Ok(config.cred_type)
}
