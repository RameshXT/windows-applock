use std::fs;
use std::path::PathBuf;
use crate::models::{AppConfig, AuthMode};
use crate::services::security;

pub fn save_config(config: &AppConfig, path: &PathBuf) -> Result<(), String> {
    let data = serde_json::to_vec(config).map_err(|e| e.to_string())?;
    // AES-256 encrypted config
    let encrypted = security::encrypt(&data, "applock-secure-v1");
    fs::write(path, encrypted).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_config(path: &PathBuf) -> AppConfig {
    if path.exists() {
        if let Ok(encrypted) = fs::read_to_string(path) {
            if let Some(decrypted) = security::decrypt(&encrypted, "applock-secure-v1") {
                if let Ok(config) = serde_json::from_slice(&decrypted) {
                    return config;
                }
            }
        }
    }
    
    // Initial Development Default: PIN 8424
    let mut config = AppConfig::default();
    config.auth_mode = Some(AuthMode::PIN);
    config.hashed_password = security::hash_password("8424");
    config.autostart = Some(false);
    config
}
