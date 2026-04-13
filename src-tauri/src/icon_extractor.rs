use std::path::Path;
use windows::Win32::UI::Shell::{SHGetFileInfoW, SHGFI_ICON, SHGFI_LARGEICON, SHFILEINFOW};
use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, HICON, GetIconInfo, ICONINFO};
use windows::Win32::Graphics::Gdi::{GetDC, ReleaseDC, GetDIBits, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, RGBQUAD};
use windows::core::PCWSTR;
use image::{RgbaImage, ImageFormat};
use tauri::{AppHandle, Manager};
use std::fs;

pub fn extract_icon_to_file(exe_path: &str, app_id: &str, app_handle: &AppHandle) -> Result<String, String> {
    let icon_dir = app_handle.path().app_data_dir()
        .map_err(|e: tauri::Error| e.to_string())?
        .join("icons");

    if !icon_dir.exists() {
        fs::create_dir_all(&icon_dir).map_err(|e| e.to_string())?;
    }

    let save_path = icon_dir.join(format!("{}.png", app_id));
    if save_path.exists() {
        return Ok(save_path.to_string_lossy().to_string());
    }

    unsafe {
        let mut shfi: SHFILEINFOW = std::mem::zeroed();
        let wide_path: Vec<u16> = exe_path.encode_utf16().chain(std::iter::once(0)).collect();
        let res = SHGetFileInfoW(
            PCWSTR(wide_path.as_ptr()),
            Default::default(),
            Some(&mut shfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );

        if res == 0 || shfi.hIcon.is_invalid() {
            return Err("Failed to get icon info".to_string());
        }

        let result = hicon_to_png(shfi.hIcon, &save_path);
        let _ = DestroyIcon(shfi.hIcon);
        
        result.map(|_| save_path.to_string_lossy().to_string())
    }
}

unsafe fn hicon_to_png(hicon: HICON, save_path: &Path) -> Result<(), String> {
    let mut ii: ICONINFO = std::mem::zeroed();
    if GetIconInfo(hicon, &mut ii).is_err() {
        return Err("GetIconInfo failed".to_string());
    }

    let hdc = GetDC(None);
    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: 32,
            biHeight: -32, // Top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: 0, // BI_RGB
            ..Default::default()
        },
        bmiColors: [RGBQUAD::default(); 1],
    };

    let mut buffer: Vec<u8> = vec![0; 32 * 32 * 4];
    let lines = GetDIBits(hdc, ii.hbmColor, 0, 32, Some(buffer.as_mut_ptr() as *mut _), &mut bmi, DIB_RGB_COLORS);
    
    ReleaseDC(None, hdc);

    if lines == 0 {
        return Err("GetDIBits failed".to_string());
    }
    for pixel in buffer.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }

    let img = RgbaImage::from_raw(32, 32, buffer)
        .ok_or("Failed to create image buffer")?;
    
    img.save_with_format(save_path, ImageFormat::Png)
        .map_err(|e| e.to_string())?;

    Ok(())
}
