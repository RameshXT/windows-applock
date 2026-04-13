use serde::{Serialize, Deserialize};
use windows::Win32::Foundation::{HWND, RECT, POINT, LPARAM, HANDLE};
use windows::core::BOOL;
use windows::Win32::UI::WindowsAndMessaging::{
    ShowWindow, SW_MINIMIZE, GetWindowLongPtrW, SetWindowLongPtrW,
    GWL_EXSTYLE, WS_EX_TOOLWINDOW, WS_EX_APPWINDOW, GetWindowPlacement,
    SetWindowPlacement, WINDOWPLACEMENT, GetWindowRect, SetWindowPos,
    HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SWP_FRAMECHANGED,
    SetForegroundWindow, BringWindowToTop, EnumWindows, GetWindowThreadProcessId,
    HHOOK,
};
use windows::Win32::UI::Input::KeyboardAndMouse::EnableWindow;
use windows::Win32::Graphics::Gdi::{
    MonitorFromWindow, MONITOR_DEFAULTTONEAREST, GetMonitorInfoW, MONITORINFO,
};
use windows::Win32::System::Threading::{
    GetCurrentProcess, OpenProcessToken,
};
use windows::Win32::Security::{
    TOKEN_QUERY, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, SE_PRIVILEGE_ENABLED,
    AdjustTokenPrivileges, LUID_AND_ATTRIBUTES, LookupPrivilegeValueW,
    DACL_SECURITY_INFORMATION,
};
use windows::Win32::Security::Authorization::{
    SetSecurityInfo, SE_KERNEL_OBJECT,
};

use std::sync::Arc;
use tauri::{AppHandle, Manager, Emitter, State};
use crate::models::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitorBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SerializablePlacement {
    pub show_cmd: u32,
    pub pt_min_position_x: i32,
    pub pt_min_position_y: i32,
    pub pt_max_position_x: i32,
    pub pt_max_position_y: i32,
    pub rc_normal_position_left: i32,
    pub rc_normal_position_top: i32,
    pub rc_normal_position_right: i32,
    pub rc_normal_position_bottom: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WindowSnapshot {
    pub hwnd: isize,
    pub was_fullscreen: bool,
    pub placement: SerializablePlacement,
    pub extended_style: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WindowError {
    HookFailed,
    Win32Error(u32),
    NotElevated,
    SnapshotFailed,
    RestoreFailed,
    InvalidHwnd,
}

impl From<windows::core::Error> for WindowError {
    fn from(err: windows::core::Error) -> Self {
        WindowError::Win32Error(err.code().0 as u32)
    }
}
#[tauri::command]
pub fn freeze_target_window(
    state: State<'_, Arc<AppState>>,
    app_handle: AppHandle,
    hwnd: isize
) -> Result<(), String> {
    freeze_window_logic(HWND(hwnd as _), &state, &app_handle).map_err(|e| format!("{:?}", e))
}
pub fn freeze_window_logic(hwnd: HWND, state: &AppState, app_handle: &AppHandle) -> Result<(), WindowError> {
    unsafe {
        if hwnd.0.is_null() {
            return Err(WindowError::InvalidHwnd);
        }
        if let Ok(snapshot) = snapshot_window_state(hwnd) {
            let mut snapshots = state.window_snapshots.write().map_err(|_| WindowError::SnapshotFailed)?;
            snapshots.insert(hwnd.0 as isize, snapshot);
        }
        let _ = hide_from_alt_tab(hwnd);
        let _ = ShowWindow(hwnd, SW_MINIMIZE);
        let _ = EnableWindow(hwnd, false);
        
        let _ = app_handle.emit("window_frozen", serde_json::json!({ "hwnd": hwnd.0 as isize }));
        Ok(())
    }
}
#[tauri::command]
pub fn assert_overlay_topmost(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let hwnd = HWND(window.hwnd().map_err(|e| e.to_string())?.0 as _);
        unsafe {
            let _ = SetWindowPos(
                hwnd,
                Some(HWND_TOPMOST),
                0, 0, 0, 0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW
            );
            let _ = SetForegroundWindow(hwnd);
            let _ = BringWindowToTop(hwnd);
        }
        let _ = app.emit("overlay_asserted_topmost", ());
        Ok(())
    } else {
        Err("Overlay window not found".into())
    }
}
#[tauri::command]
pub fn get_target_monitor_bounds(hwnd: isize) -> Result<MonitorBounds, String> {
    let hwnd = HWND(hwnd as _);
    unsafe {
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        
        if GetMonitorInfoW(monitor, &mut info).as_bool() {
            Ok(MonitorBounds {
                x: info.rcMonitor.left,
                y: info.rcMonitor.top,
                width: (info.rcMonitor.right - info.rcMonitor.left) as u32,
                height: (info.rcMonitor.bottom - info.rcMonitor.top) as u32,
            })
        } else {
            Err("Failed to get monitor info".into())
        }
    }
}
#[tauri::command]
pub async fn restore_locked_window(
    state: State<'_, Arc<AppState>>,
    app_handle: AppHandle,
    hwnd: isize,
) -> Result<(), String> {
    let h_wnd = HWND(hwnd as _);
    
    let snapshot = {
        let snapshots = state.window_snapshots.read().map_err(|e| e.to_string())?;
        snapshots.get(&hwnd).cloned()
    };
    
    if let Some(snapshot) = snapshot {
        restore_window_state(h_wnd, &snapshot).map_err(|e| format!("{:?}", e))?;
        
        let mut snapshots = state.window_snapshots.write().map_err(|e| e.to_string())?;
        snapshots.remove(&hwnd);
        
        let _ = app_handle.emit("window_restored", serde_json::json!({ "hwnd": hwnd, "was_fullscreen": snapshot.was_fullscreen }));
        Ok(())
    } else {
        Err("Snapshot not found for window".into())
    }
}
pub fn hide_from_alt_tab(hwnd: HWND) -> Result<(), WindowError> {
    unsafe {
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        let mut new_style = ex_style;
        new_style |= WS_EX_TOOLWINDOW.0;
        new_style &= !WS_EX_APPWINDOW.0;
        let _ = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style as isize);
        let _ = SetWindowPos(hwnd, None, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_FRAMECHANGED);
        Ok(())
    }
}
pub fn restore_alt_tab(hwnd: HWND) -> Result<(), WindowError> {
    unsafe {
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        let mut new_style = ex_style;
        new_style &= !WS_EX_TOOLWINDOW.0;
        new_style |= WS_EX_APPWINDOW.0;
        let _ = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style as isize);
        let _ = SetWindowPos(hwnd, None, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_FRAMECHANGED);
        Ok(())
    }
}
pub fn is_fullscreen(hwnd: HWND) -> Result<bool, WindowError> {
    unsafe {
        let mut rect = RECT::default();
        let _ = GetWindowRect(hwnd, &mut rect);
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let _ = GetMonitorInfoW(monitor, &mut info);
        let is_fs = rect.left <= info.rcMonitor.left && rect.top <= info.rcMonitor.top &&
                   rect.right >= info.rcMonitor.right && rect.bottom >= info.rcMonitor.bottom;
        Ok(is_fs)
    }
}
pub fn snapshot_window_state(hwnd: HWND) -> Result<WindowSnapshot, WindowError> {
    unsafe {
        let mut placement = WINDOWPLACEMENT {
            length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
            ..Default::default()
        };
        if GetWindowPlacement(hwnd, &mut placement).is_err() {
            return Err(WindowError::SnapshotFailed);
        }
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        let was_fullscreen = is_fullscreen(hwnd).unwrap_or(false);
        Ok(WindowSnapshot {
            hwnd: hwnd.0 as isize,
            was_fullscreen,
            extended_style: ex_style,
            placement: SerializablePlacement {
                show_cmd: placement.showCmd,
                pt_min_position_x: placement.ptMinPosition.x,
                pt_min_position_y: placement.ptMinPosition.y,
                pt_max_position_x: placement.ptMaxPosition.x,
                pt_max_position_y: placement.ptMaxPosition.y,
                rc_normal_position_left: placement.rcNormalPosition.left,
                rc_normal_position_top: placement.rcNormalPosition.top,
                rc_normal_position_right: placement.rcNormalPosition.right,
                rc_normal_position_bottom: placement.rcNormalPosition.bottom,
            },
        })
    }
}
pub fn restore_window_state(hwnd: HWND, snapshot: &WindowSnapshot) -> Result<(), WindowError> {
    unsafe {
        let _ = EnableWindow(hwnd, true);
        let _ = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, snapshot.extended_style as isize);
        let placement = WINDOWPLACEMENT {
            length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
            showCmd: snapshot.placement.show_cmd,
            ptMinPosition: POINT { x: snapshot.placement.pt_min_position_x, y: snapshot.placement.pt_min_position_y },
            ptMaxPosition: POINT { x: snapshot.placement.pt_max_position_x, y: snapshot.placement.pt_max_position_y },
            rcNormalPosition: RECT {
                left: snapshot.placement.rc_normal_position_left,
                top: snapshot.placement.rc_normal_position_top,
                right: snapshot.placement.rc_normal_position_right,
                bottom: snapshot.placement.rc_normal_position_bottom,
            },
            ..Default::default()
        };
        if SetWindowPlacement(hwnd, &placement).is_err() {
            return Err(WindowError::RestoreFailed);
        }
        let _ = SetWindowPos(hwnd, None, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_FRAMECHANGED);
        Ok(())
    }
}
pub fn protect_process() -> Result<(), WindowError> {
    unsafe {
        let mut h_token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY | TOKEN_ADJUST_PRIVILEGES, &mut h_token).is_err() {
            return Err(WindowError::NotElevated);
        }
        let mut luid = windows::Win32::Foundation::LUID::default();
        let priv_name: Vec<u16> = "SeDebugPrivilege\0".encode_utf16().collect();
        if LookupPrivilegeValueW(None, windows::core::PCWSTR(priv_name.as_ptr()), &mut luid).is_ok() {
            let tp = TOKEN_PRIVILEGES {
                PrivilegeCount: 1,
                Privileges: [LUID_AND_ATTRIBUTES { Luid: luid, Attributes: SE_PRIVILEGE_ENABLED }],
            };
            let _ = AdjustTokenPrivileges(h_token, false, Some(&tp), 0, None, None);
        }
        let _ = SetSecurityInfo(GetCurrentProcess(), SE_KERNEL_OBJECT, DACL_SECURITY_INFORMATION, None, None, None, None);
        Ok(())
    }
}
pub fn get_process_windows(pid: u32) -> Vec<HWND> {
    let mut windows: Vec<HWND> = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(enum_window_proc), LPARAM(&mut windows as *mut Vec<HWND> as isize));
    }
    let mut process_windows = Vec::new();
    for hwnd in windows {
        let mut window_pid = 0;
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut window_pid)); }
        if window_pid == pid {
            process_windows.push(hwnd);
        }
    }
    process_windows
}

extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = unsafe { &mut *(lparam.0 as *mut Vec<HWND>) };
    windows.push(hwnd);
    BOOL::from(true)
}
pub fn hide_window(hwnd: HWND) -> Result<(), WindowError> {
    unsafe {
        let _ = ShowWindow(hwnd, SW_MINIMIZE);
        Ok(())
    }
}
pub fn suspend_process(_pid: u32) -> Result<(), WindowError> {
    Ok(())
}

#[tauri::command]
pub fn install_hook(state: State<'_, Arc<AppState>>, app_handle: AppHandle) -> Result<(), String> {
    let hook = crate::keyboard_hook::install_keyboard_hook().map_err(|e| format!("{:?}", e))?;
    let mut h = state.keyboard_hook.lock().unwrap();
    *h = Some(SendHhook(hook));
    let _ = app_handle.emit("keyboard_hook_installed", ());
    Ok(())
}

#[tauri::command]
pub fn uninstall_hook(state: State<'_, Arc<AppState>>, app_handle: AppHandle) -> Result<(), String> {
    let mut h = state.keyboard_hook.lock().unwrap();
    if let Some(hook_wrapper) = h.take() {
        crate::keyboard_hook::uninstall_keyboard_hook(hook_wrapper.0).map_err(|e| format!("{:?}", e))?;
    }
    let _ = app_handle.emit("keyboard_hook_removed", ());
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct SendHhook(pub HHOOK);
unsafe impl Send for SendHhook {}
unsafe impl Sync for SendHhook {}
