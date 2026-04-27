use crate::icon_extractor;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};
use winreg::RegKey;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScannedApp {
    pub id: String,
    pub name: String,
    pub executable_path: String,
    pub icon_path: String,
    pub version: String,
    pub publisher: String,
    pub source: ScanSource,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ScanSource {
    RegistryHKLM,
    RegistryHKCU,
    ProgramFiles,
    ProgramFilesX86,
    AppDataLocal,
    AppDataRoaming,
    StartMenu,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanCache {
    pub scan_timestamp: u64,
    pub app_count: usize,
    pub entries: Vec<ScannedApp>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "status", content = "data")]
pub enum ScanStatus {
    Idle,
    Scanning { percent: u8, current_source: String },
    Complete,
    Failed { reason: String },
}

pub enum ScanError {
    RegistryError(String),
    IoError(String),
    CacheError(String),
    SystemError(String),
}

const CACHE_FILE: &str = "scan_cache.json";
pub fn start_scan_internal(app_handle: AppHandle) -> Result<String, String> {
    let scan_id = Uuid::new_v4().to_string();
    let app_handle_clone = app_handle.clone();

    std::thread::spawn(move || {
        let _ = run_full_scan(app_handle_clone);
    });

    Ok(scan_id)
}

fn run_full_scan(app_handle: AppHandle) -> Result<(), String> {
    let start_time = Instant::now();
    app_handle
        .emit(
            "scan_progress",
            serde_json::json!({ "percent": 5, "current_source": "Registry" }),
        )
        .unwrap();
    let sources = rayon::join(
        || rayon::join(scan_registry_hklm, scan_registry_hkcu),
        || {
            rayon::join(
                || rayon::join(scan_program_files, scan_program_files_x86),
                || rayon::join(scan_appdata_local, scan_start_menu),
            )
        },
    );
    let mut all_results = Vec::new();
    let ((hklm, hkcu), ((pf, pfx), (al, sm))) = sources;

    all_results.extend(hklm.unwrap_or_default());
    all_results.extend(hkcu.unwrap_or_default());
    all_results.extend(pf.unwrap_or_default());
    all_results.extend(pfx.unwrap_or_default());
    all_results.extend(al.unwrap_or_default());
    all_results.extend(sm.unwrap_or_default());

    app_handle
        .emit(
            "scan_progress",
            serde_json::json!({ "percent": 70, "current_source": "Deduplicating" }),
        )
        .unwrap();
    let mut apps = deduplicate_apps(all_results);
    apps.retain(|app| !is_system_process(app));
    app_handle
        .emit(
            "scan_progress",
            serde_json::json!({ "percent": 85, "current_source": "Extracting Icons" }),
        )
        .unwrap();

    let handle_ref = &app_handle;
    apps.par_iter_mut().for_each(|app| {
        if let Ok(ip) =
            icon_extractor::extract_icon_to_file(&app.executable_path, &app.id, handle_ref)
        {
            app.icon_path = ip;
        }
    });
    let cache = ScanCache {
        scan_timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        app_count: apps.len(),
        entries: apps.clone(),
    };
    let _ = save_cache(&app_handle, &cache);
    app_handle
        .emit(
            "scan_complete",
            serde_json::json!({
                "app_count": apps.len(),
                "scan_duration_ms": start_time.elapsed().as_millis() as u64
            }),
        )
        .unwrap();

    Ok(())
}

fn scan_registry(
    root: RegKey,
    path: &str,
    source: ScanSource,
) -> Result<Vec<ScannedApp>, ScanError> {
    let uninstall_key = root
        .open_subkey_with_flags(path, KEY_READ)
        .map_err(|e| ScanError::RegistryError(e.to_string()))?;

    let mut apps = Vec::new();
    for name in uninstall_key.enum_keys().map(|x| x.unwrap()) {
        if let Ok(subkey) = uninstall_key.open_subkey_with_flags(&name, KEY_READ) {
            let display_name: String = subkey.get_value("DisplayName").unwrap_or_default();
            let install_loc: String = subkey.get_value("InstallLocation").unwrap_or_default();
            let icon_str: String = subkey.get_value("DisplayIcon").unwrap_or_default();
            let mut exe_path = clean_icon_path(&icon_str);

            if exe_path.is_empty() && !install_loc.is_empty() {
                if let Some(found) = find_exe_in_dir(&PathBuf::from(&install_loc), 1) {
                    exe_path = found.to_string_lossy().to_string();
                }
            }

            if !display_name.is_empty() && !exe_path.is_empty() {
                apps.push(ScannedApp {
                    id: Uuid::new_v4().to_string(),
                    name: display_name,
                    executable_path: exe_path,
                    icon_path: String::new(),
                    version: subkey.get_value("DisplayVersion").unwrap_or_default(),
                    publisher: subkey.get_value("Publisher").unwrap_or_default(),
                    source: source.clone(),
                });
            }
        }
    }
    Ok(apps)
}

fn scan_registry_hklm() -> Result<Vec<ScannedApp>, ScanError> {
    let mut apps = scan_registry(
        RegKey::predef(HKEY_LOCAL_MACHINE),
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        ScanSource::RegistryHKLM,
    )?;
    if let Ok(wow) = scan_registry(
        RegKey::predef(HKEY_LOCAL_MACHINE),
        "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        ScanSource::RegistryHKLM,
    ) {
        apps.extend(wow);
    }
    Ok(apps)
}

fn scan_registry_hkcu() -> Result<Vec<ScannedApp>, ScanError> {
    scan_registry(
        RegKey::predef(HKEY_CURRENT_USER),
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        ScanSource::RegistryHKCU,
    )
}

fn scan_dir_for_exes(base_path: &str, source: ScanSource) -> Result<Vec<ScannedApp>, ScanError> {
    let path = Path::new(base_path);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let mut apps = Vec::new();
    let entries = fs::read_dir(path).map_err(|e| ScanError::IoError(e.to_string()))?;

    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            if let Ok(sub) = fs::read_dir(&p) {
                for sub_entry in sub.flatten() {
                    let sp = sub_entry.path();
                    if sp.is_file() && sp.extension().and_then(|s| s.to_str()) == Some("exe") {
                        apps.push(exe_to_app(sp, source.clone()));
                    }
                }
            }
        }
    }
    Ok(apps)
}

fn scan_program_files() -> Result<Vec<ScannedApp>, ScanError> {
    let path = env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
    scan_dir_for_exes(&path, ScanSource::ProgramFiles)
}

fn scan_program_files_x86() -> Result<Vec<ScannedApp>, ScanError> {
    let path =
        env::var("ProgramFiles(x86)").unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());
    scan_dir_for_exes(&path, ScanSource::ProgramFilesX86)
}

fn scan_appdata_local() -> Result<Vec<ScannedApp>, ScanError> {
    let path = env::var("LOCALAPPDATA").map_err(|e| ScanError::SystemError(e.to_string()))?;
    scan_dir_for_exes(&path, ScanSource::AppDataLocal)
}

fn scan_start_menu() -> Result<Vec<ScannedApp>, ScanError> {
    let mut apps = Vec::new();
    if let Ok(pd) = env::var("ProgramData") {
        let path = Path::new(&pd).join("Microsoft\\Windows\\Start Menu\\Programs");
        walk_start_menu(&path, &mut apps);
    }
    Ok(apps)
}

fn walk_start_menu(dir: &Path, apps: &mut Vec<ScannedApp>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                walk_start_menu(&p, apps);
            } else if p.extension().and_then(|s| s.to_str()) == Some("exe") {
                apps.push(exe_to_app(p, ScanSource::StartMenu));
            }
        }
    }
}

fn deduplicate_apps(all: Vec<ScannedApp>) -> Vec<ScannedApp> {
    let mut map: HashMap<String, ScannedApp> = HashMap::new();
    for app in all {
        let norm_path = app.executable_path.to_lowercase().replace("/", "\\");

        if let Some(existing) = map.get(&norm_path) {
            match (&app.source, &existing.source) {
                (ScanSource::RegistryHKLM | ScanSource::RegistryHKCU, _) => {
                    map.insert(norm_path, app);
                }
                _ => {}
            }
        } else {
            map.insert(norm_path, app);
        }
    }
    map.into_values().collect()
}

fn is_system_process(app: &ScannedApp) -> bool {
    let p = app.executable_path.to_lowercase();
    if p.contains("c:\\windows") {
        return true;
    }

    let system_exes = vec![
        "svchost.exe",
        "lsass.exe",
        "csrss.exe",
        "winlogon.exe",
        "taskhost.exe",
        "dwm.exe",
        "explorer.exe",
        "conhost.exe",
        "runtimebroker.exe",
        "searchindexer.exe",
        "spoolsv.exe",
    ];

    system_exes.iter().any(|sys| p.ends_with(sys))
}
fn clean_icon_path(path: &str) -> String {
    let p = path.trim_matches('\"').split(',').next().unwrap_or("");
    if p.to_lowercase().ends_with(".exe") {
        p.to_string()
    } else {
        String::new()
    }
}

fn find_exe_in_dir(dir: &Path, depth: u8) -> Option<PathBuf> {
    if depth == 0 {
        return None;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("exe") {
                return Some(p);
            }
        }
    }
    None
}

fn exe_to_app(path: PathBuf, source: ScanSource) -> ScannedApp {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();
    ScannedApp {
        id: Uuid::new_v4().to_string(),
        name,
        executable_path: path.to_string_lossy().to_string(),
        icon_path: String::new(),
        version: String::new(),
        publisher: String::new(),
        source,
    }
}

fn save_cache(app_handle: &AppHandle, cache: &ScanCache) -> Result<(), String> {
    let path = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join(CACHE_FILE);
    let json = serde_json::to_string_pretty(cache).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_cached_results(app_handle: &AppHandle) -> Result<Vec<ScannedApp>, String> {
    let path = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join(CACHE_FILE);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let json = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let cache: ScanCache = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(cache.entries)
}
