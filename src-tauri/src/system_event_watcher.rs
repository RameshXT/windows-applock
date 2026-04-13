use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::{AppHandle, Manager, Emitter};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM, HINSTANCE};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW, 
    CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, MSG, WINDOW_EX_STYLE, WINDOW_STYLE, 
    WNDCLASSW, HWND_MESSAGE, WM_POWERBROADCAST, WM_WTSSESSION_CHANGE,
    PBT_APMSUSPEND, PBT_APMRESUMEAUTOMATIC,
};
use windows::Win32::System::RemoteDesktop::{
    WTSRegisterSessionNotification, NOTIFY_FOR_THIS_SESSION,
};
use windows::core::PCWSTR;
use crate::grace_manager::{reset_all_grace_sessions, GraceSessionStore, SystemEvent};

static mut APP_HANDLE: Option<AppHandle> = None;

const WTS_CONSOLE_CONNECT: u32 = 1;
const WTS_CONSOLE_DISCONNECT: u32 = 2;
const WTS_REMOTE_DISCONNECT: u32 = 4;
const WTS_SESSION_LOGOFF: u32 = 6;
const WTS_SESSION_LOCK: u32 = 7;

pub fn start_system_event_watcher(app_handle: AppHandle) {
    unsafe {
        APP_HANDLE = Some(app_handle.clone());
    }

    std::thread::spawn(move || {
        unsafe {
            let class_name = "AppLockSystemEventWatcher\0".encode_utf16().collect::<Vec<u16>>();
            let instance = windows::Win32::System::LibraryLoader::GetModuleHandleW(None).unwrap_or_default();
            
            let wnd_class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                hInstance: HINSTANCE(instance.0),
                lpszClassName: PCWSTR(class_name.as_ptr()),
                ..Default::default()
            };

            RegisterClassW(&wnd_class);

            let hwnd_res = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                PCWSTR(class_name.as_ptr()),
                PCWSTR("SystemEventWatcher\0".encode_utf16().collect::<Vec<u16>>().as_ptr()),
                WINDOW_STYLE::default(),
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                Some(HWND_MESSAGE), 
                None,
                Some(wnd_class.hInstance),
                None,
            );

            let hwnd = match hwnd_res {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("Failed to create hidden window for system event watcher: {}", e);
                    return;
                }
            };

            if let Err(e) = WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION) {
                 eprintln!("Failed to register session notification: {}", e);
            }

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                DispatchMessageW(&msg);
            }
        }
    });

    let app_handle_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(5));
        loop {
            ticker.tick().await;
            unsafe {
                let mut is_running: windows::core::BOOL = false.into();
                let spi_res = windows::Win32::UI::WindowsAndMessaging::SystemParametersInfoW(
                    windows::Win32::UI::WindowsAndMessaging::SPI_GETSCREENSAVERRUNNING,
                    0,
                    Some(&mut is_running as *mut _ as *mut _),
                    windows::Win32::UI::WindowsAndMessaging::SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
                );
                
                if spi_res.is_ok() && is_running.as_bool() {
                    let store = app_handle_clone.state::<Arc<RwLock<GraceSessionStore>>>();
                    reset_all_grace_sessions(SystemEvent::ScreensaverStarted, &store, &app_handle_clone).await;
                }
            }
        }
    });
}

extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_WTSSESSION_CHANGE => {
                let event = wparam.0 as u32;
                handle_session_change(event);
            }
            WM_POWERBROADCAST => {
                let event = wparam.0 as u32;
                handle_power_broadcast(event);
            }
            _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
    LRESULT(0)
}

unsafe fn handle_session_change(event: u32) {
    let app_handle = if let Some(h) = (&raw const APP_HANDLE).as_ref().and_then(|h| h.as_ref()) { h } else { return };
    let store = app_handle.state::<Arc<RwLock<GraceSessionStore>>>();
    
    match event {
        WTS_SESSION_LOCK => {
            tauri::async_runtime::spawn({
                let app_handle = app_handle.clone();
                let store = store.clone();
                async move {
                    reset_all_grace_sessions(SystemEvent::ScreenLocked, &store, &app_handle).await;
                }
            });
        }
        e if e == WTS_SESSION_LOGOFF || e == WTS_REMOTE_DISCONNECT || e == WTS_CONSOLE_DISCONNECT => {
            tauri::async_runtime::spawn({
                let app_handle = app_handle.clone();
                let store = store.clone();
                async move {
                    reset_all_grace_sessions(SystemEvent::UserSwitched, &store, &app_handle).await;
                }
            });
        }
        WTS_CONSOLE_CONNECT => {
            tauri::async_runtime::spawn({
                let app_handle = app_handle.clone();
                let store = store.clone();
                async move {
                    reset_all_grace_sessions(SystemEvent::UserSwitched, &store, &app_handle).await;
                }
            });
        }
        _ => {}
    }
}

unsafe fn handle_power_broadcast(event: u32) {
    let app_handle = if let Some(h) = (&raw const APP_HANDLE).as_ref().and_then(|h| h.as_ref()) { h } else { return };
    let store = app_handle.state::<Arc<RwLock<GraceSessionStore>>>();

    match event {
        PBT_APMSUSPEND => {
            tauri::async_runtime::spawn({
                let app_handle = app_handle.clone();
                let store = store.clone();
                async move {
                    reset_all_grace_sessions(SystemEvent::SessionSuspend, &store, &app_handle).await;
                }
            });
        }
        PBT_APMRESUMEAUTOMATIC => {
            let _ = app_handle.emit("system_resumed", serde_json::json!({}));
        }
        _ => {}
    }
}
