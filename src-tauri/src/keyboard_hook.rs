use crate::window_manager::WindowError;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{VK_LWIN, VK_RWIN, VK_TAB};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, WH_KEYBOARD_LL,
    WM_KEYDOWN, WM_SYSKEYDOWN,
};

pub static mut HOOK_HANDLE: Option<HHOOK> = None;
pub fn install_keyboard_hook() -> Result<HHOOK, WindowError> {
    unsafe {
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0)
            .map_err(|_| WindowError::HookFailed)?;

        HOOK_HANDLE = Some(hook);
        Ok(hook)
    }
}
pub fn uninstall_keyboard_hook(hook: HHOOK) -> Result<(), WindowError> {
    unsafe {
        if UnhookWindowsHookEx(hook).is_ok() {
            HOOK_HANDLE = None;
            Ok(())
        } else {
            Err(WindowError::HookFailed)
        }
    }
}
unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kbd = *(lparam.0 as *const KBDLLHOOKSTRUCT);
        let key = kbd.vkCode as u16;
        let is_alt = (kbd.flags.0 & 0x20) != 0; // LLKHF_ALTDOWN
        if is_alt && key == VK_TAB.0 {
            return LRESULT(1);
        }
        static mut WIN_PRESSED: bool = false;
        if key == VK_LWIN.0 || key == VK_RWIN.0 {
            if wparam.0 == WM_KEYDOWN as usize || wparam.0 == WM_SYSKEYDOWN as usize {
                WIN_PRESSED = true;
            } else {
                WIN_PRESSED = false;
            }
        }

        if WIN_PRESSED && key == 0x44 {
            // 'D' key
            return LRESULT(1);
        }
    }

    CallNextHookEx(None, code, wparam, lparam)
}
