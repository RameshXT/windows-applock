use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct KillProtectionStatus {
    pub enabled: bool,
    pub method: String,
}

pub fn is_elevated() -> bool {
    unsafe {
        let mut token: HANDLE = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;

        let success = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            size,
            &mut size,
        )
        .is_ok();

        let _ = CloseHandle(token);
        success && elevation.TokenIsElevated != 0
    }
}

pub fn apply_kill_protection() -> Result<KillProtectionStatus, String> {
    if !is_elevated() {
        return Ok(KillProtectionStatus {
            enabled: false,
            method: "none".to_string(),
        });
    }

    if let Err(e) = set_break_on_termination(true) {
        return Err(e);
    }

    Ok(KillProtectionStatus {
        enabled: true,
        method: "critical_process".to_string(),
    })
}

extern "system" {
    fn NtSetInformationProcess(
        process_handle: HANDLE,
        process_information_class: i32,
        process_information: *const std::ffi::c_void,
        process_information_length: u32,
    ) -> i32;
}

const PROCESS_BREAK_ON_TERMINATION: i32 = 29;

pub fn set_break_on_termination(enabled: bool) -> Result<(), String> {
    if !is_elevated() {
        return Err("Elevation required".to_string());
    }

    unsafe {
        let val: u32 = if enabled { 1 } else { 0 };
        let status = NtSetInformationProcess(
            GetCurrentProcess(),
            PROCESS_BREAK_ON_TERMINATION,
            &val as *const _ as *const _,
            4,
        );

        if status != 0 {
            return Err(format!("NtSetInformationProcess failed: 0x{:X}", status));
        }
    }
    Ok(())
}
