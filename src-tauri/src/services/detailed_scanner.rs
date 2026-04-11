use serde::Serialize;
use winreg::enums::*;
use winreg::RegKey;
use std::path::Path;
use base64::{Engine as _, engine::general_purpose};
use std::io::Cursor;
use image::ImageFormat;
use tauri::command;

// System package prefixes to skip
const SKIP_PREFIXES: &[&str] = &[
    "Microsoft.Windows.",
    "Microsoft.UI.",
    "Microsoft.NET.",
    "Microsoft.VCLibs",
    "Microsoft.DirectX",
    "Microsoft.Services.",
    "Windows.CBSPreview",
    "NcsiUwpApp",
    "Microsoft.LanguageExperiencePack",
    "InputApp",
    "MicrosoftWindows.",
    "Microsoft.SecHealthUI",
    "Microsoft.Winget.",
];

#[derive(Serialize, Clone, Debug)]
pub struct DetailedApp {
    pub name: String,
    pub publisher: String,
    pub version: String,
    pub install_date: String,
    pub size_kb: u64,
    pub icon_base64: String,
}

#[command]
pub async fn get_detailed_apps() -> Result<Vec<DetailedApp>, String> {
    let mut apps = Vec::new();
    let mut seen_names = std::collections::HashSet::new();

    // 1. Registry Scanning (4 paths) - Win32/classic apps
    let registry_paths = [
        (HKEY_LOCAL_MACHINE, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        (HKEY_LOCAL_MACHINE, "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        (HKEY_CURRENT_USER, "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        (HKEY_CURRENT_USER, "Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
    ];

    for (root, path) in registry_paths {
        let root_key = RegKey::predef(root);
        if let Ok(key) = root_key.open_subkey(path) {
            for name in key.enum_keys().filter_map(|x| x.ok()) {
                if let Ok(sub_key) = key.open_subkey(&name) {
                    let display_name: String = sub_key.get_value("DisplayName").unwrap_or_default();
                    if display_name.is_empty() || seen_names.contains(&display_name) {
                        continue;
                    }

                    // Skip GUID-like DisplayNames
                    let dn_check = display_name.trim_matches('{').trim_matches('}');
                    if dn_check.chars().filter(|c| *c == '-').count() >= 4 && dn_check.len() > 30 {
                        continue;
                    }

                    // Skip SystemComponent=1
                    let system_component: u32 = sub_key.get_value("SystemComponent").unwrap_or(0);
                    if system_component == 1 {
                        continue;
                    }

                    // Skip system install locations
                    let install_location: String = sub_key.get_value("InstallLocation").unwrap_or_default();
                    if install_location.to_lowercase().contains("\\systemapps\\")
                        || install_location.to_lowercase().contains("\\windowsapps\\microsoft.windows.")
                        || install_location.to_lowercase().contains("\\windowsapps\\microsoft.ui.")
                        || install_location.to_lowercase().contains("\\windowsapps\\microsoft.net.")
                        || install_location.to_lowercase().contains("\\windowsapps\\microsoftwindows.")
                    {
                        continue;
                    }

                    let publisher: String = sub_key.get_value("Publisher").unwrap_or_default();
                    let version: String = sub_key.get_value("DisplayVersion").unwrap_or_default();
                    let install_date: String = sub_key.get_value("InstallDate").unwrap_or_default();
                    let size_kb: u32 = sub_key.get_value("EstimatedSize").unwrap_or(0);
                    let display_icon: String = sub_key.get_value("DisplayIcon").unwrap_or_default();

                    let icon_base64 = if !display_icon.is_empty() {
                        extract_icon_to_base64(&display_icon).unwrap_or_default()
                    } else {
                        "".to_string()
                    };

                    seen_names.insert(display_name.clone());
                    apps.push(DetailedApp {
                        name: display_name,
                        publisher,
                        version,
                        install_date,
                        size_kb: size_kb as u64,
                        icon_base64,
                    });
                }
            }
        }
    }

    let ps_cmd = r#"Get-AppxPackage | Where-Object { $_.SignatureKind -eq 'Store' -or $_.SignatureKind -eq 'Developer' } | ForEach-Object {
        $m = $_ | Get-AppxPackageManifest;
        $dn = $m.Package.Properties.DisplayName;
        [PSCustomObject]@{
            Name = $_.Name;
            DisplayName = $dn;
            Publisher = $_.Publisher;
            Version = $_.Version;
            InstallLocation = $_.InstallLocation;
        }
    } | ConvertTo-Json -Compress"#;

    let ps_output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", ps_cmd])
        .output();

    if let Ok(output) = ps_output {
        if let Ok(json_str) = String::from_utf8(output.stdout) {
            let json_str = json_str.trim();
            let entries: Vec<serde_json::Value> = if json_str.starts_with('[') {
                serde_json::from_str(json_str).unwrap_or_default()
            } else if json_str.starts_with('{') {
                serde_json::from_str::<serde_json::Value>(json_str)
                    .map(|v| vec![v])
                    .unwrap_or_default()
            } else {
                vec![]
            };

            for entry in entries {
                let pkg_name = entry["Name"].as_str().unwrap_or("").to_string();
                let install_location = entry["InstallLocation"].as_str().unwrap_or("").to_string();

                // Skip system/framework packages by name prefix
                if SKIP_PREFIXES.iter().any(|p| pkg_name.starts_with(p)) {
                    continue;
                }

                // Skip anything in SystemApps
                if install_location.to_lowercase().contains("\\systemapps\\") {
                    continue;
                }

                // Get friendly display name
                let ps_display_name = entry["DisplayName"].as_str().unwrap_or("");
                let display_name = if !ps_display_name.is_empty() && !ps_display_name.starts_with("ms-resource:") {
                    ps_display_name.to_string()
                } else {
                    get_appx_display_name(&install_location, &pkg_name)
                };

                if display_name.is_empty() || seen_names.contains(&display_name) {
                    continue;
                }

                let publisher = entry["Publisher"].as_str().unwrap_or("").to_string();
                let version = entry["Version"].as_str().unwrap_or("").to_string();
                let icon_base64 = find_appx_icon(&install_location);

                seen_names.insert(display_name.clone());
                apps.push(DetailedApp {
                    name: display_name,
                    publisher,
                    version,
                    install_date: "".to_string(),
                    size_kb: 0,
                    icon_base64,
                });
            }
        }
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(apps)
}

// Resolve official DisplayName for Store apps
fn get_appx_display_name(install_location: &str, package_name: &str) -> String {
    let mut name = String::new();

    // 1. Try reading from AppxManifest.xml first (Fast)
    if !install_location.is_empty() {
        let manifest_path = format!("{}\\AppxManifest.xml", install_location);
        if let Ok(content) = std::fs::read_to_string(&manifest_path) {
            if let Some(start) = content.find("<DisplayName>") {
                let rest = &content[start + "<DisplayName>".len()..];
                if let Some(end) = rest.find("</DisplayName>") {
                    name = rest[..end].trim().to_string();
                }
            }
        }
    }

    // 2. If name is ms-resource or empty, resolve via absolute PowerShell manifest method
    if name.is_empty() || name.starts_with("ms-resource:") {
        let ps_cmd = format!(
            "(Get-AppxPackage -Name \"{}\" | Get-AppxPackageManifest).Package.Properties.DisplayName", 
            package_name
        );
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd])
            .output();

        if let Ok(out) = output {
            let resolved = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !resolved.is_empty() && !resolved.starts_with("ms-resource:") {
                return resolved;
            }
        }
    } else {
        return name;
    }

    // 3. Fallback: humanize the package name
    let after_dot = package_name.splitn(2, '.').nth(1).unwrap_or(package_name);
    let mut readable = String::new();
    for (i, ch) in after_dot.chars().enumerate() {
        if i > 0 && ch.is_uppercase() && !after_dot.chars().nth(i - 1).map(|c| c.is_uppercase()).unwrap_or(false) {
            readable.push(' ');
        }
        readable.push(ch);
    }
    readable
}

// Find icon PNG in appx package folder
fn find_appx_icon(install_location: &str) -> String {
    if install_location.is_empty() {
        return "".to_string();
    }
    let candidates = [
        "Assets\\StoreLogo.png",
        "Assets\\Square44x44Logo.png",
        "Assets\\Square150x150Logo.png",
        "Assets\\Logo.png",
        "Assets\\AppIcon.png",
        "logo.png",
        "icon.png",
    ];
    for candidate in &candidates {
        let full = format!("{}\\{}", install_location, candidate);
        if Path::new(&full).exists() {
            if let Some(b64) = load_image_to_base64(&full) {
                return b64;
            }
        }
    }
    "".to_string()
}

fn extract_icon_to_base64(path: &str) -> Option<String> {
    let clean_path = path.split(',').next()?.trim_matches('"');
    let p = Path::new(clean_path);
    if !p.exists() { return None; }
    let lower = clean_path.to_lowercase();
    if lower.ends_with(".ico") || lower.ends_with(".png") || lower.ends_with(".jpg") {
        return load_image_to_base64(clean_path);
    }
    if lower.ends_with(".exe") || lower.ends_with(".dll") {
        return extract_exe_icon(clean_path);
    }
    None
}

fn extract_exe_icon(path: &str) -> Option<String> {
    use windows::Win32::UI::Shell::ExtractIconExW;
    use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO};
    use windows::Win32::Graphics::Gdi::{
        GetDIBits, CreateCompatibleDC, DeleteDC, DeleteObject,
        BITMAPINFOHEADER, BITMAPINFO, DIB_RGB_COLORS, BI_RGB,
    };
    use windows::core::PCWSTR;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let wide: Vec<u16> = OsStr::new(path).encode_wide().chain(Some(0)).collect();
    let mut large_icon = windows::Win32::UI::WindowsAndMessaging::HICON::default();
    let mut small_icon = windows::Win32::UI::WindowsAndMessaging::HICON::default();

    let count = unsafe {
        ExtractIconExW(PCWSTR(wide.as_ptr()), 0, Some(&mut large_icon), Some(&mut small_icon), 1)
    };
    if count == 0 { return None; }

    let hicon = if !large_icon.is_invalid() { large_icon } else { small_icon };
    if hicon.is_invalid() { return None; }

    let result = (|| -> Option<String> {
        let mut icon_info = ICONINFO::default();
        unsafe { GetIconInfo(hicon, &mut icon_info).ok()? };
        let hdc = unsafe { CreateCompatibleDC(None) };
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: 32,
                biHeight: -32,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut pixels = vec![0u8; 32 * 32 * 4];
        let lines = unsafe {
            GetDIBits(hdc, icon_info.hbmColor, 0, 32, Some(pixels.as_mut_ptr() as *mut _), &mut bmi, DIB_RGB_COLORS)
        };
        unsafe { let _ = DeleteDC(hdc); };
        unsafe { let _ = DeleteObject(icon_info.hbmColor); };
        unsafe { let _ = DeleteObject(icon_info.hbmMask); };
        if lines == 0 { return None; }
        for chunk in pixels.chunks_exact_mut(4) { chunk.swap(0, 2); }
        let img = image::RgbaImage::from_raw(32, 32, pixels)?;
        let dynamic = image::DynamicImage::ImageRgba8(img);
        let mut buffer = Cursor::new(Vec::new());
        dynamic.write_to(&mut buffer, ImageFormat::Png).ok()?;
        Some(format!("data:image/png;base64,{}", general_purpose::STANDARD.encode(buffer.get_ref())))
    })();

    unsafe { let _ = DestroyIcon(large_icon); };
    unsafe { let _ = DestroyIcon(small_icon); };
    result
}

fn load_image_to_base64(path: &str) -> Option<String> {
    let img = image::open(path).ok()?;
    let mut buffer = Cursor::new(Vec::new());
    img.write_to(&mut buffer, ImageFormat::Png).ok()?;
    Some(format!("data:image/png;base64,{}", general_purpose::STANDARD.encode(buffer.get_ref())))
}