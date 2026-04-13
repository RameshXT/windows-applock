use std::sync::Arc;
use crate::models::{AppConfig, AppState};
use crate::services::security;
use crate::utils::config::save_config;
pub fn verify_impl(password: &str, config: &mut AppConfig, state: &Arc<AppState>) -> Result<bool, String> {
    println!("[Auth] Attempting verification for password length: {}", password.len());

    if let Some(until) = config.lockout_until {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();
        if now < until {
            let remaining = until - now;
            println!("[Auth] Denied: Security lockout active for {} more seconds", remaining);
            return Err(format!("Security lockout active. Try again in {} seconds.", remaining));
        } else {
            println!("[Auth] Lockout expired, resetting attempts");
            config.lockout_until = None;
            config.wrong_attempts = Some(0);
        }
    }

    let mut is_valid = security::verify_password(password, &config.hashed_password);

    if !is_valid {
        if let Some(ref rkey) = config.recovery_key {
            if password.trim().to_uppercase() == rkey.to_uppercase() {
                println!("[Auth] Recovery Key matched! Granting access.");
                is_valid = true;
            }
        }
    }

    if is_valid {
        println!("[Auth] Access granted.");
        let mut unlocked = state.is_unlocked.lock().unwrap();
        *unlocked = true;
        config.wrong_attempts = Some(0);
    } else {
        let attempts = config.wrong_attempts.unwrap_or(0) + 1;
        let limit = config.attempt_limit.unwrap_or(3);
        println!("[Auth] Credentials MISMATCH. Attempts: {}/{}", attempts, limit);
        config.wrong_attempts = Some(attempts);

        if attempts >= limit {
            let duration = config.lockout_duration.unwrap_or(30);
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_secs();
            println!("[Auth] LOCKOUT ACTIVATED for {} seconds", duration);
            config.lockout_until = Some(now + duration as u64);
        }
    }

    save_config(config, &state.config_path)?;
    Ok(is_valid)
}
