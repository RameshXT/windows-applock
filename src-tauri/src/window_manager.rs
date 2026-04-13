use windows::Win32::Foundation::{HWND, LPARAM, RECT};
use windows::core::BOOL;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowThreadProcessId, ShowWindow, SW_HIDE,
    SetWindowPos, HWND_BOTTOM, SWP_NOSIZE, SWP_NOACTIVATE, GetWindowPlacement,
    WINDOWPLACEMENT, GetWindowRect, GetWindowLongPtrW, GWL_EXSTYLE, WS_EX_TOPMOST,
    HWND_TOPMOST, HWND_NOTOPMOST, SWP_SHOWWINDOW, SWP_FRAMECHANGED,
    SetForegroundWindow, BringWindowToTop, WS_EX_TOOLWINDOW, WS_EX_APPWINDOW,
    SetWindowPlacement, SetWindowLongPtrW,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_SUSPEND_RESUME,
};
use windows::Win32::Graphics::Gdi::{
    MonitorFromWindow, MONITOR_DEFAULTTONEAREST,
};
use ntapi::ntpsapi::{NtSuspendProcess, NtResumeProcess};
use crate::lock_session::WindowSnapshot;
use crate::fullscreen_handler::detect_fullscreen;

#[derive(Debug, thiserror::Error)]
pub enum WindowError {
    #[error("HWND not found for PID {0}")]
    HwndNotFound(u32),
    #[error("Freeze failed: {0}")]
    FreezeFailed(String),
    #[error("Resume failed: {0}")]
    ResumeFailed(String),
    #[error("Snapshot failed: {0}")]
    SnapshotFailed(String),
    #[error("Restore failed: {0}")]
    RestoreFailed(String),
    #[error("Hook installation failed")]
    HookInstallFailed,
    #[error("Monitor enumeration failed")]
    MonitorEnumFailed,
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Elevation required")]
    ElevationRequired,
    #[error("Fullscreen handling failed: {0}")]
    FullscreenHandlingFailed(String),
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

pub fn take_window_snapshot(hwnd: HWND, pid: u32, app_id: &str) -> Result<WindowSnapshot, WindowError> {
    unsafe {
        let mut placement = WINDOWPLACEMENT {
            length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
            ..Default::default()
        };
        if GetWindowPlacement(hwnd, &mut placement).is_err() {
            return Err(WindowError::SnapshotFailed("GetWindowPlacement failed".to_string()));
        }

        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return Err(WindowError::SnapshotFailed("GetWindowRect failed".to_string()));
        }

        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as usize;
        let was_topmost = (ex_style & WS_EX_TOPMOST.0 as usize) != 0;
        
        let was_fullscreen = detect_fullscreen(hwnd);
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);

        Ok(WindowSnapshot {
            app_id: app_id.to_string(),
            process_id: pid,
            hwnd: hwnd.0 as isize,
            original_x: rect.left,
            original_y: rect.top,
            original_width: rect.right - rect.left,
            original_height: rect.bottom - rect.top,
            original_show_state: placement.showCmd,
            was_fullscreen,
            monitor_handle: monitor.0 as isize,
            was_topmost,
            child_windows: Vec::new(),
        })
    }
}

pub fn hide_window(hwnd: HWND) -> Result<(), WindowError> {
    unsafe {
        let _ = ShowWindow(hwnd, SW_HIDE);
        SetWindowPos(
            hwnd,
            Some(HWND_BOTTOM),
            -32000,
            -32000,
            0,
            0,
            SWP_NOSIZE | SWP_NOACTIVATE,
        ).map_err(|e| WindowError::FreezeFailed(e.to_string()))?;
    }
    Ok(())
}

pub fn suspend_process(pid: u32) -> Result<(), WindowError> {
    unsafe {
        let handle = OpenProcess(PROCESS_SUSPEND_RESUME, false, pid)
            .map_err(|e| WindowError::FreezeFailed(e.to_string()))?;
        
        if handle.is_invalid() {
            return Err(WindowError::PermissionDenied);
        }

        let status = NtSuspendProcess(handle.0 as _);
        if status != 0 {
            return Err(WindowError::FreezeFailed(format!("NtSuspendProcess failed: 0x{:X}", status)));
        }
    }
    Ok(())
}

pub fn resume_process(pid: u32) -> Result<(), WindowError> {
    unsafe {
        let handle = OpenProcess(PROCESS_SUSPEND_RESUME, false, pid)
            .map_err(|e| WindowError::ResumeFailed(e.to_string()))?;
        
        if handle.is_invalid() {
            return Err(WindowError::PermissionDenied);
        }

        let status = NtResumeProcess(handle.0 as _);
        if status != 0 {
            return Err(WindowError::ResumeFailed(format!("NtResumeProcess failed: 0x{:X}", status)));
        }
    }
    Ok(())
}

pub fn restore_window_from_snapshot(snapshot: &WindowSnapshot) -> Result<(), WindowError> {
    unsafe {
        let hwnd = HWND(snapshot.hwnd as _);
        
        let current_ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as usize;
        let mut new_ex_style = current_ex_style;
        new_ex_style &= !(WS_EX_TOOLWINDOW.0 as usize);
        new_ex_style |= WS_EX_APPWINDOW.0 as usize;
        
        let _ = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex_style as isize);

        let placement = WINDOWPLACEMENT {
            length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
            showCmd: snapshot.original_show_state,
            ..Default::default()
        };
        
        let _ = SetWindowPlacement(hwnd, &placement);

        let z_order = if snapshot.was_topmost { HWND_TOPMOST } else { HWND_NOTOPMOST };
        
        SetWindowPos(
            hwnd,
            Some(z_order),
            snapshot.original_x,
            snapshot.original_y,
            snapshot.original_width,
            snapshot.original_height,
            SWP_SHOWWINDOW | SWP_FRAMECHANGED,
        ).map_err(|e| WindowError::RestoreFailed(e.to_string()))?;

        let _ = SetForegroundWindow(hwnd);
        let _ = BringWindowToTop(hwnd);
    }
    Ok(())
}
