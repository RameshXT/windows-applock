use lazy_static::lazy_static;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{Arc, RwLock};
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{VK_ESCAPE, VK_LWIN, VK_RWIN, VK_TAB};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MSG,
    WH_KEYBOARD_LL,
};

lazy_static! {
    static ref HOOK_HANDLE: AtomicIsize = AtomicIsize::new(0);
    static ref ACTIVE_LOCKS_COUNT: AtomicIsize = AtomicIsize::new(0);
}

pub struct InputBlocker {}

impl InputBlocker {
    pub fn start(_has_active_locks: Arc<RwLock<bool>>) -> Result<(), String> {
        if HOOK_HANDLE.load(Ordering::SeqCst) != 0 {
            return Ok(());
        }

        std::thread::spawn(move || unsafe {
            let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), None, 0)
                .expect("Failed to install keyboard hook");

            HOOK_HANDLE.store(hook.0 as isize, Ordering::SeqCst);

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {}

            let _ = UnhookWindowsHookEx(hook);
            HOOK_HANDLE.store(0, Ordering::SeqCst);
        });

        Ok(())
    }

    pub fn set_active_locks(count: usize) {
        ACTIVE_LOCKS_COUNT.store(count as isize, Ordering::SeqCst);
    }

    pub fn stop() {}
}

extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kbd = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
        let active_locks = ACTIVE_LOCKS_COUNT.load(Ordering::SeqCst) > 0;

        if active_locks {
            let alt_down = (kbd.flags.0 & 0x20) != 0;
            let vk = kbd.vkCode as u16;

            if alt_down && (vk == VK_TAB.0 || vk == VK_ESCAPE.0) {
                return LRESULT(1);
            }

            if vk == VK_LWIN.0 || vk == VK_RWIN.0 {
                return LRESULT(1);
            }
        }
    }

    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}
