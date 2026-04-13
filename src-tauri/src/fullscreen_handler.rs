use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowRect, GetWindowLongW, GWL_STYLE, WS_CAPTION, WS_POPUP,
    GetWindowLongPtrW, GWL_EXSTYLE, WS_EX_TOPMOST,
};
use windows::Win32::Graphics::Gdi::{
    MonitorFromWindow, GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO,
};

/// Detect if a window is in fullscreen mode.
pub fn detect_fullscreen(hwnd: HWND) -> bool {
    unsafe {
        // 1. Check window bounds against monitor bounds
        let mut window_rect = RECT::default();
        if GetWindowRect(hwnd, &mut window_rect).is_err() {
            return false;
        }

        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut monitor_info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };

        if GetMonitorInfoW(monitor, &mut monitor_info).as_bool() {
            let m_rect = monitor_info.rcMonitor;
            let is_same_size = window_rect.left == m_rect.left &&
                               window_rect.top == m_rect.top &&
                               window_rect.right == m_rect.right &&
                               window_rect.bottom == m_rect.bottom;
            
            if is_same_size {
                return true;
            }
        }

        // 2. Check window styles: WS_POPUP without WS_CAPTION = borderless fullscreen
        let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
        if (style & WS_POPUP.0 != 0) && (style & WS_CAPTION.0 == 0) {
            return true;
        }

        // 3. Check WS_EX_TOPMOST extended style
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as usize;
        if (ex_style & WS_EX_TOPMOST.0 as usize) != 0 {
            // This isn't definitive but often true for fullscreen
            // Let's rely more on size and basic style
        }
    }
    false
}
