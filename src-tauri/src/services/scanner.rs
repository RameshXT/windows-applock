use serde::Serialize;
use std::process::Command;
use std::os::windows::process::CommandExt;
use winreg::enums::*;
use winreg::RegKey;

#[derive(Serialize, Clone, Debug)]
pub struct InstalledApp {
    pub name: String,
    pub path: Option<String>,
    pub icon: Option<String>,
}

pub fn get_apps() -> Vec<InstalledApp> {
    let mut apps = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();
    let mut seen_names = std::collections::HashSet::new();

    // 1. Scan PowerShell (Start Menu) - Priority as these usually have icons
    let ps_apps = get_apps_powershell();
    for app in ps_apps {
        let name_lower = app.name.to_lowercase();
        // Extract a "clean" name for deduplication (strip anything after a parenthesis or a version-like dash/space)
        let clean_name = name_lower.split(" (").next().unwrap_or(&name_lower).trim().to_string();
            
        if !seen_names.contains(&clean_name) {
            if let Some(ref path) = app.path {
                let path_lower = path.to_lowercase();
                if !seen_paths.contains(&path_lower) {
                    seen_paths.insert(path_lower);
                    seen_names.insert(clean_name);
                    apps.push(app);
                }
            }
        }
    }

    // 2. Scan Registry (Native Fallback)
    let mut registry_apps = Vec::new();
    scan_registry(&mut registry_apps, &mut seen_paths, &mut seen_names);
    
    // For registry apps, we only keep them if they have icons
    // and aren't duplicates.
    for app in registry_apps {
        if app.icon.is_some() && !app.icon.as_ref().unwrap().is_empty() {
             apps.push(app);
        }
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

fn scan_registry(apps: &mut Vec<InstalledApp>, seen_paths: &mut std::collections::HashSet<String>, seen_names: &mut std::collections::HashSet<String>) {
    let paths = [
        (HKEY_LOCAL_MACHINE, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        (HKEY_LOCAL_MACHINE, "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        (HKEY_CURRENT_USER, "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
    ];

    let junk_patterns = [
        "uninst", "setup", "helper", "convert", "report", "crash", "extra", "update",
        "redistributable", "sdk", "runtime", "driver", ".net", "framework", "microsoft visual c++",
        "kb", "security update", "service pack", "language pack", "windows driver",
        "component", "library", "directx", "vulkan", "nvidia physx", "intel(r) management",
        "host service", "client service", "engine", "module", "plugin", "python 2.", "python 3."
    ];

    for (root_hkey, path) in paths {
        let root = RegKey::predef(root_hkey);
        if let Ok(key) = root.open_subkey(path) {
            for name in key.enum_keys().filter_map(|x: Result<String, _>| x.ok()) {
                if let Ok(sub_key) = key.open_subkey(&name) {
                    let display_name: String = sub_key.get_value("DisplayName").unwrap_or_default();
                    let install_location: String = sub_key.get_value("InstallLocation").unwrap_or_default();
                    let display_icon: String = sub_key.get_value("DisplayIcon").unwrap_or_default();

                    let name_lower = display_name.to_lowercase();
                    
                    // Filter out empty names or junk patterns
                    if display_name.is_empty() || junk_patterns.iter().any(|&j| name_lower.contains(j)) {
                        continue;
                    }

                    // Strict deduplication
                    let clean_name = name_lower.split(" (").next().unwrap_or(&name_lower).trim().to_string();
                    if seen_names.contains(&clean_name) {
                        continue;
                    }

                    let mut exec_path = None;
                    if !display_icon.is_empty() {
                        let icon_path = display_icon.split(',').next().unwrap_or(&display_icon).trim_matches('"');
                        if icon_path.to_lowercase().ends_with(".exe") && std::path::Path::new(icon_path).exists() {
                             exec_path = Some(icon_path.to_string());
                        }
                    }

                    if exec_path.is_none() && !install_location.is_empty() {
                        if let Ok(read_dir) = std::fs::read_dir(&install_location) {
                            for entry in read_dir.filter_map(|e| e.ok()) {
                                let p = entry.path();
                                if p.is_file() && p.extension().map_or(false, |ext| ext == "exe") {
                                    let p_str = p.to_string_lossy().to_string();
                                    let file_name = p.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                                    if !junk_patterns.iter().any(|&j| file_name.contains(j)) {
                                        exec_path = Some(p_str);
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if let Some(path) = exec_path {
                        let path_lower = path.to_lowercase();
                        if !seen_paths.contains(&path_lower) {
                            seen_paths.insert(path_lower);
                            seen_names.insert(clean_name);
                            apps.push(InstalledApp {
                                name: display_name,
                                path: Some(path),
                                icon: None, // Will attempt to filter later if no icon is found
                            });
                        }
                    }
                }
            }
        }
    }
}

pub fn get_apps_powershell() -> Vec<InstalledApp> {
    let script = r#"
        Add-Type -AssemblyName System.Drawing
        $Shell = New-Object -ComObject WScript.Shell
        
        $Blacklist = @("*Uninstall*", "*Setup*", "*Manual*", "*Documentation*", "*Reset*", "*Link*", "*Website*", "*ReadMe*", "*Update*", "*Helps*", "*Configuration*", "*Command Prompt*", "*PowerShell*", "*Cross Tools*", "*Native Tools*", "*Verifier*", "*Cert Kit*", "*SDK*", "*Tools for*", "*Database Compare*", "*Spreadsheet Compare*", "*Telemetry Log*", "*Visual C++*", "*Redistributable*", "*Runtime*", "*Framework*", "*Build Tools*", "*Windows Backup*")
        
        $Results = @()
        $SeenNames = @{}
        $SeenPaths = @{}

        # Get both Desktop and Microsoft Store Apps
        Get-StartApps | ForEach-Object {
            $App = $_
            $skip = $false
            foreach ($pattern in $Blacklist) {
                if ($App.Name -like $pattern) { $skip = $true; break }
            }
            if ($skip) { return }

            $CleanName = $App.Name -replace '\s\d+(\.\d+)*.*', ''
            $CleanName = $CleanName.ToLower().Trim()
            if ($SeenNames.ContainsKey($CleanName)) { return }

            $TargetPath = ""
            $IconBase64 = ""

            # Check if it's a Win32 App (usually has a path in AppID or is a shortcut)
            if ($App.AppID -like "*\*") {
                $TargetPath = $App.AppID.Split('\')[-1]
                if ($App.AppID -like "*:*") {
                    $TargetPath = $App.AppID
                }
            }

            # Try to resolve shortcut if AppID isn't a direct path
            if (-not (Test-Path $TargetPath)) {
                $lnk = Get-ChildItem -Path "$env:APPDATA\Microsoft\Windows\Start Menu\Programs", "$env:SystemDrive\ProgramData\Microsoft\Windows\Start Menu\Programs" -Filter "$($App.Name).lnk" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
                if ($lnk) {
                    try {
                        $shortcut = $Shell.CreateShortcut($lnk.FullName)
                        $TargetPath = $shortcut.TargetPath
                    } catch {}
                }
            }

            # Handle Microsoft Store Apps (UWP)
            if (-not $TargetPath -or $App.AppID -like "*!*" -or $App.AppID -notlike "*\*") {
                $pkgName = $App.AppID.Split('!')[0]
                $pkg = Get-AppxPackage -Name "*$pkgName*" -ErrorAction SilentlyContinue | Select-Object -First 1
                
                # If name match failed, try matching the AppID directly
                if (-not $pkg) {
                    $pkg = Get-AppxPackage | Where-Object { $_.PackageFamilyName -eq $pkgName } | Select-Object -First 1
                }

                if ($pkg) {
                    # Try to find the executable from the manifest
                    try {
                        $manifestPath = Join-Path $pkg.InstallLocation "AppxManifest.xml"
                        if (Test-Path $manifestPath) {
                            $manifest = [xml](Get-Content -Path $manifestPath -ErrorAction SilentlyContinue)
                            $exec = $manifest.Package.Applications.Application.Executable | Select-Object -First 1
                            if ($exec) {
                                $TargetPath = Join-Path $pkg.InstallLocation $exec
                            }
                        }
                    } catch {}

                    # Fallback to searching if manifest parsing failed
                    if (-not $TargetPath -or -not (Test-Path $TargetPath)) {
                        $TargetPath = (Get-ChildItem -Path $pkg.InstallLocation -Filter *.exe -ErrorAction SilentlyContinue | Sort-Object Length -Descending | Select-Object -First 1).FullName
                    }
                }
            }

            # Final validation: If it's still empty, try one last shell-based resolution
            if (-not $TargetPath -or -not (Test-Path $TargetPath)) {
                 return 
            }

            if ($SeenPaths.ContainsKey($TargetPath.ToLower())) { return }

            # Extract Icon
            try {
                $icon = [System.Drawing.Icon]::ExtractAssociatedIcon($TargetPath)
                if ($icon) {
                    $bitmap = $icon.ToBitmap()
                    $ms = New-Object System.IO.MemoryStream
                    $bitmap.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
                    $IconBase64 = [Convert]::ToBase64String($ms.ToArray())
                    $icon.Dispose(); $bitmap.Dispose(); $ms.Dispose()
                }
            } catch {}

            # If no icon found, we still add it but with a null icon (the UI will handle it)
            # This ensures important apps like WhatsApp aren't skipped just because of icon issues.
            $SeenNames[$CleanName] = $true
            $SeenPaths[$TargetPath.ToLower()] = $true
            $Results += [PSCustomObject]@{
                Name = $App.Name
                Path = $TargetPath
                Icon = if ($IconBase64) { "data:image/png;base64," + $IconBase64 } else { "" }
            }
        }

        if ($Results.Count -eq 0) {
            Write-Output "[]"
        } else {
            $Results | ConvertTo-Json -Compress
        }
    "#;

    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    if let Ok(out) = output {
        let json = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if json.is_empty() || json == "[]" {
            return Vec::new();
        }
        
        if let Ok(apps) = serde_json::from_str::<Vec<serde_json::Value>>(&json) {
            return apps.into_iter().map(|v| InstalledApp {
                name: v["Name"].as_str().unwrap_or("Unknown").to_string(),
                path: v["Path"].as_str().map(|s| s.to_string()),
                icon: v["Icon"].as_str().map(|s| s.to_string()),
            }).collect();
        } else if let Ok(app) = serde_json::from_str::<serde_json::Value>(&json) {
            return vec![InstalledApp {
                name: app["Name"].as_str().unwrap_or("Unknown").to_string(),
                path: app["Path"].as_str().map(|s| s.to_string()),
                icon: app["Icon"].as_str().map(|s| s.to_string()),
            }];
        }
    }

    Vec::new()
}
