use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum AuthMode {
    Password,
    PIN,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LockedApp {
    pub id: String,
    pub name: String,
    pub exec_name: String,
    pub icon: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub hashed_password: String,
    pub locked_apps: Vec<LockedApp>,
    pub auth_mode: Option<AuthMode>,
    pub attempt_limit: Option<u32>,
    pub lockout_duration: Option<u32>,
    pub autostart: Option<bool>,
    pub minimize_to_tray: Option<bool>,
    pub stealth_mode: Option<bool>,
    pub notifications_enabled: Option<bool>,
    pub animations_intensity: Option<String>,
    pub autolock_on_sleep: Option<bool>,
    pub auto_lock_duration: Option<u32>,
    pub panic_key: Option<String>,
    pub grace_period: Option<u32>,
    pub strict_enforcement: Option<bool>,
    pub immediate_relock: Option<bool>,
    pub protection_persistence: Option<bool>,
    pub wrong_attempts: Option<u32>,
    pub lockout_until: Option<u64>,
    pub recovery_hint: Option<String>,
    pub display_name: Option<String>,
    pub profile_picture: Option<String>,
    pub biometrics_enabled: Option<bool>,
    pub last_credential_change: Option<u64>,
    pub recovery_key: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hashed_password: String::new(),
            locked_apps: Vec::new(),
            auth_mode: Some(AuthMode::PIN),
            attempt_limit: Some(3),
            lockout_duration: Some(30),
            autostart: Some(true),
            minimize_to_tray: Some(true),
            stealth_mode: Some(false),
            notifications_enabled: Some(true),
            animations_intensity: Some("high".to_string()),
            autolock_on_sleep: Some(true),
            auto_lock_duration: Some(5),
            panic_key: Some("Ctrl+Alt+L".to_string()),
            grace_period: Some(15),
            strict_enforcement: Some(true),
            immediate_relock: Some(true),
            protection_persistence: Some(true),
            wrong_attempts: Some(0),
            lockout_until: None,
            recovery_hint: None,
            display_name: Some("User".to_string()),
            profile_picture: None,
            biometrics_enabled: Some(false),
            last_credential_change: None,
            recovery_key: None,
        }
    }
}
