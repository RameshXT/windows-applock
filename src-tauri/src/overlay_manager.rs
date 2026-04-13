use windows::Win32::Foundation::{HWND, RECT, LPARAM};
use windows::core::BOOL;
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowPos, HWND_TOPMOST, SWP_SHOWWINDOW, SWP_NOACTIVATE, SetForegroundWindow,
    BringWindowToTop, GetWindowRect, GetWindowLongPtrW, GWL_EXSTYLE, WS_EX_TOPMOST,
    SWP_NOSIZE, SWP_NOMOVE,
};
use windows::Win32::Graphics::Gdi::{
    HDC, MonitorFromRect, MONITOR_DEFAULTTONEAREST,
    EnumDisplayMonitors, GetMonitorInfoW, MONITORINFOEXW, HMONITOR,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use crate::lock_session::{MonitorInfo, Rect};

pub fn enumerate_monitors() -> Result<Vec<MonitorInfo>, String> {
    let mut monitors: Vec<MonitorInfo> = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(
            Some(HDC::default()),
            None,
            Some(monitor_enum_proc),
            LPARAM(&mut monitors as *mut Vec<MonitorInfo> as isize),
        );
    }
    Ok(monitors)
}

extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let monitors = unsafe { &mut *(lparam.0 as *mut Vec<MonitorInfo>) };
    
    let mut info = MONITORINFOEXW {
        monitorInfo: windows::Win32::Graphics::Gdi::MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFOEXW>() as u32,
            ..Default::default()
        },
        ..Default::default()
    };

    unsafe {
        if GetMonitorInfoW(hmonitor, &mut info.monitorInfo).as_bool() {
            let mut dpi_x: u32 = 0;
            let mut dpi_y: u32 = 0;
            let _ = GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);

            monitors.push(MonitorInfo {
                handle: hmonitor.0 as isize,
                work_area: Rect {
                    left: info.monitorInfo.rcWork.left,
                    top: info.monitorInfo.rcWork.top,
                    right: info.monitorInfo.rcWork.right,
                    bottom: info.monitorInfo.rcWork.bottom,
                },
                full_rect: Rect {
                    left: info.monitorInfo.rcMonitor.left,
                    top: info.monitorInfo.rcMonitor.top,
                    right: info.monitorInfo.rcMonitor.right,
                    bottom: info.monitorInfo.rcMonitor.bottom,
                },
                is_primary: (info.monitorInfo.dwFlags & 1) != 0,
                dpi: dpi_x,
            });
        }
    }
    BOOL::from(true)
}

pub fn get_window_monitor(hwnd: HWND) -> Result<MonitorInfo, String> {
    unsafe {
        let mut rect = RECT::default();
        let _ = GetWindowRect(hwnd, &mut rect);
        let hmonitor = MonitorFromRect(&rect, MONITOR_DEFAULTTONEAREST);
        
        let mut info = MONITORINFOEXW {
            monitorInfo: windows::Win32::Graphics::Gdi::MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFOEXW>() as u32,
                ..Default::default()
            },
            ..Default::default()
        };

        if GetMonitorInfoW(hmonitor, &mut info.monitorInfo).as_bool() {
            let mut dpi_x: u32 = 0;
            let mut dpi_y: u32 = 0;
            let _ = GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);

            Ok(MonitorInfo {
                handle: hmonitor.0 as isize,
                work_area: Rect {
                    left: info.monitorInfo.rcWork.left,
                    top: info.monitorInfo.rcWork.top,
                    right: info.monitorInfo.rcWork.right,
                    bottom: info.monitorInfo.rcWork.bottom,
                },
                full_rect: Rect {
                    left: info.monitorInfo.rcMonitor.left,
                    top: info.monitorInfo.rcMonitor.top,
                    right: info.monitorInfo.rcMonitor.right,
                    bottom: info.monitorInfo.rcMonitor.bottom,
                },
                is_primary: (info.monitorInfo.dwFlags & 1) != 0,
                dpi: dpi_x,
            })
        } else {
            Err("Failed to get monitor info".to_string())
        }
    }
}

pub fn position_overlay(overlay_hwnd: HWND, monitor: &MonitorInfo) -> Result<(), String> {
    unsafe {
        SetWindowPos(
            overlay_hwnd,
            Some(HWND_TOPMOST),
            monitor.full_rect.left,
            monitor.full_rect.top,
            monitor.full_rect.right - monitor.full_rect.left,
            monitor.full_rect.bottom - monitor.full_rect.top,
            SWP_SHOWWINDOW | SWP_NOACTIVATE,
        ).map_err(|e| e.to_string())?;

        let _ = SetForegroundWindow(overlay_hwnd);
        let _ = BringWindowToTop(overlay_hwnd);
    }
    Ok(())
}

pub fn assert_topmost(overlay_hwnd: HWND) -> Result<(), String> {
    unsafe {
        let ex_style = GetWindowLongPtrW(overlay_hwnd, GWL_EXSTYLE) as usize;
        if (ex_style & WS_EX_TOPMOST.0 as usize) == 0 {
            SetWindowPos(
                overlay_hwnd,
                Some(HWND_TOPMOST),
                0, 0, 0, 0,
                SWP_NOSIZE | SWP_NOMOVE | SWP_NOACTIVATE,
            ).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
