use windows::core::HSTRING;
pub struct IPackageDebugSettings;

impl IPackageDebugSettings {
    pub unsafe fn suspend(&self, _package_full_name: &HSTRING) -> windows::core::Result<()> {
        Ok(())
    }
    pub unsafe fn resume(&self, _package_full_name: &HSTRING) -> windows::core::Result<()> {
        Ok(())
    }
}

pub fn is_uwp_app(path: &str) -> bool {
    path.to_lowercase()
        .contains("c:\\program files\\windowsapps\\")
}

pub struct UwpHandler {
    debug_settings: Option<IPackageDebugSettings>,
}

impl UwpHandler {
    pub fn new() -> Self {
        UwpHandler {
            debug_settings: None,
        }
    }

    pub fn suspend_app(&self, package_family_name: &str) -> Result<(), String> {
        if let Some(settings) = &self.debug_settings {
            unsafe {
                settings
                    .suspend(&HSTRING::from(package_family_name))
                    .map_err(|e| e.to_string())
            }
        } else {
            Err("IPackageDebugSettings not available".to_string())
        }
    }

    pub fn resume_app(&self, package_family_name: &str) -> Result<(), String> {
        if let Some(settings) = &self.debug_settings {
            unsafe {
                settings
                    .resume(&HSTRING::from(package_family_name))
                    .map_err(|e| e.to_string())
            }
        } else {
            Err("IPackageDebugSettings not available".to_string())
        }
    }
}

pub fn get_uwp_package_family_name(_pid: u32) -> Option<String> {
    None
}
