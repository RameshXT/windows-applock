use serde::Serialize;
use std::process::Command;
use std::os::windows::process::CommandExt;
use winreg::enums::*;
use winreg::RegKey;
use std::collections::{HashSet, HashMap};
use std::path::Path;

#[derive(Serialize, Clone, Debug)]
pub struct InstalledApp {
    pub name: String,
    pub path: Option<String>,
    pub icon: Option<String>,
}

pub fn get_apps() -> Vec<InstalledApp> {
    let mut apps_map: HashMap<String, InstalledApp> = HashMap::new();
    let mut seen_paths = HashSet::new();

    // 1. Scan Registry (Native - Very fast & reliable for uninstalls)
    scan_registry(&mut apps_map, &mut seen_paths);
    
    // 2. Scan PowerShell (Start Menu & UWP) - Complements registry with Store apps & correct icons
    let ps_apps = get_apps_powershell();
    for app in ps_apps {
        if let Some(ref path) = app.path {
            let path_lower = path.to_lowercase();
            // If we already have this executable, we prefer the name/metadata from StartApps
            // unless the registry name is already solid.
            if !seen_paths.contains(&path_lower) {
                seen_paths.insert(path_lower);
                apps_map.insert(app.name.clone(), app);
            }
        }
    }

    let mut final_apps: Vec<InstalledApp> = apps_map.into_values().collect();
    
    // Sort alphabetically by display name
    final_apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    final_apps
}

fn scan_registry(apps: &mut HashMap<String, InstalledApp>, seen_paths: &mut HashSet<String>) {
    let registry_locations = [
        (HKEY_LOCAL_MACHINE, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        (HKEY_LOCAL_MACHINE, "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        (HKEY_CURRENT_USER, "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
    ];

    let exclude_keywords = [
        "redistributable", "update", "hotfix", "service pack", "language pack", 
        "driver", "framework", "runtime", "sdk", "library", "directx", "vulkan", 
        "nvidia physx", "intel(r) management", "host service", "client service", 
        "engine", "module", "plugin", "python", "node.js", "java", "webview2"
    ];

    for (root_hkey, path) in registry_locations {
        let root = RegKey::predef(root_hkey);
        if let Ok(key) = root.open_subkey(path) {
            for name in key.enum_keys().filter_map(|x: Result<String, _>| x.ok()) {
                if let Ok(sub_key) = key.open_subkey(&name) {
                    // Inclusion Rules
                    let display_name: String = sub_key.get_value("DisplayName").unwrap_or_default();
                    if display_name.is_empty() { continue; }

                    let name_lower = display_name.to_lowercase();
                    
                    // Exclusion Rules
                    if exclude_keywords.iter().any(|&k| name_lower.contains(k)) { continue; }
                    
                    let system_component: u32 = sub_key.get_value("SystemComponent").unwrap_or(0);
                    if system_component == 1 { continue; }

                    let release_type: String = sub_key.get_value("ReleaseType").unwrap_or_default();
                    let rt_lower = release_type.to_lowercase();
                    if rt_lower.contains("update") || rt_lower.contains("hotfix") || rt_lower.contains("pack") { continue; }

                    let display_icon: String = sub_key.get_value("DisplayIcon").unwrap_or_default();
                    let install_location: String = sub_key.get_value("InstallLocation").unwrap_or_default();

                    let mut exec_path = None;

                    // 1. Try to extract from DisplayIcon if it points to an .exe
                    if !display_icon.is_empty() {
                        let icon_path = display_icon.split(',').next().unwrap_or(&display_icon).trim_matches('"');
                        if icon_path.to_lowercase().ends_with(".exe") && Path::new(icon_path).exists() {
                            exec_path = Some(icon_path.to_string());
                        }
                    }

                    // 2. Try to find main exe in InstallLocation
                    if exec_path.is_none() && !install_location.is_empty() && Path::new(&install_location).exists() {
                        if let Ok(read_dir) = std::fs::read_dir(&install_location) {
                            let mut exes: Vec<_> = read_dir.filter_map(|e| e.ok())
                                .filter(|e| e.path().extension().map_or(false, |ext| ext == "exe"))
                                .collect();
                            
                            // Heuristic: Prefer the exe that matches the display name, or the largest one
                            exes.sort_by_key(|e| std::fs::metadata(e.path()).map(|m| m.len()).unwrap_or(0));
                            if let Some(entry) = exes.last() {
                                exec_path = Some(entry.path().to_string_lossy().to_string());
                            }
                        }
                    }

                    if let Some(path) = exec_path {
                        let path_lower = path.to_lowercase();
                        // Ignore uninstallers or setup tools
                        if path_lower.contains("uninst") || path_lower.contains("setup") || path_lower.contains("helper") {
                            continue;
                        }

                        if !seen_paths.contains(&path_lower) {
                            seen_paths.insert(path_lower.clone());
                            apps.insert(display_name.clone(), InstalledApp {
                                name: display_name,
                                path: Some(path),
                                icon: None, // PowerShell will add icons later or we live without them for rare apps
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
        
        # Less restrictive - we want to find everything user-facing
        $ExcludePatterns = @("*Uninstall*", "*Manual*", "*Documentation*", "*Build Tools*", "*Hotfix*", "*Service Pack*", "*Language Pack*", "*Redistributable*")
        
        $Apps = @{} 
        $StartAppsMap = @{}
        $AllStartApps = Get-StartApps
        $AllStartApps | ForEach-Object {
            if ($_.AppID -match "!") {
                $pfn = $_.AppID.Split("!")[0]
                if (-not $StartAppsMap.ContainsKey($pfn)) { $StartAppsMap[$pfn] = $_.Name }
            }
        }

        function Get-IconBase64($Path) {
            if (-not $Path -or -not (Test-Path $Path)) { return "" }
            try {
                $icon = $null; $bitmap = $null
                if ($Path.ToLower().EndsWith(".exe") -or $Path.ToLower().EndsWith(".dll") -or $Path.ToLower().EndsWith(".ico")) {
                    $icon = [System.Drawing.Icon]::ExtractAssociatedIcon($Path)
                    if ($icon) { $bitmap = $icon.ToBitmap() }
                } else {
                    $bitmap = [System.Drawing.Bitmap]::FromFile($Path)
                }
                if ($bitmap) {
                    $ms = New-Object System.IO.MemoryStream
                    $bitmap.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
                    $base64 = [Convert]::ToBase64String($ms.ToArray())
                    $ms.Dispose(); if ($icon) { $icon.Dispose() }; $bitmap.Dispose()
                    return "data:image/png;base64," + $base64
                }
            } catch {}
            return ""
        }

        # 1. Broad Registry Scan (Win32 & PWAs)
        $UninstallKeys = @(
            "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*",
            "HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*",
            "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*"
        )
        foreach ($Key in $UninstallKeys) {
            Get-ItemProperty $Key -ErrorAction SilentlyContinue | ForEach-Object {
                if (-not $_.DisplayName -or $_.SystemComponent -eq 1) { return }
                
                $Path = ""
                if ($_.DisplayIcon) { $Path = $_.DisplayIcon.Split(',')[0].Trim('"') }
                if (-not (Test-Path $Path) -and $_.InstallLocation) {
                    $Path = (Get-ChildItem -Path $_.InstallLocation -Filter *.exe -ErrorAction SilentlyContinue | Sort-Object Length -Descending | Select-Object -First 1).FullName
                }
                if (-not (Test-Path $Path)) { return }

                $Apps[$_.DisplayName] = @{ Name = $_.DisplayName; Path = $Path; Icon = Get-IconBase64 $Path }
            }
        }

        # 2. Start Menu Scan (Start Menu Shortcuts & Pinned Apps)
        $AllStartApps | ForEach-Object {
            if ($Apps.ContainsKey($_.Name)) { return }
            
            $Path = ""; $IconBase64 = ""
            if ($_.AppID -match "^[A-Z]:\\.*\.exe$") { $Path = $_.AppID }
            
            if (-not (Test-Path $Path)) {
                $lnk = Get-ChildItem -Path "$env:APPDATA\Microsoft\Windows\Start Menu\Programs", "$env:SystemDrive\ProgramData\Microsoft\Windows\Start Menu\Programs" -Filter "$($_.Name).lnk" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
                if ($lnk) { try { $Path = ($Shell.CreateShortcut($lnk.FullName)).TargetPath } catch {} }
            }

            if (-not (Test-Path $Path)) { return }
            $Apps[$_.Name] = @{ Name = $_.Name; Path = $Path; Icon = Get-IconBase64 $Path }
        }

        # 3. Dedicated UWP Scan (The "Instagram" Solver)
        Get-AppxPackage | Where-Object { $_.IsFramework -eq $false -and $_.IsResourcePackage -eq $false -and $_.IsBundle -eq $false } | ForEach-Object {
            $Pkg = $_
            $DisplayName = if ($StartAppsMap.ContainsKey($Pkg.PackageFamilyName)) { $StartAppsMap[$Pkg.PackageFamilyName] } else { $Pkg.Name }
            if ($Apps.ContainsKey($DisplayName)) { return } 
            
            try {
                $ManifestPath = Join-Path $Pkg.InstallLocation "AppxManifest.xml"
                if (Test-Path $ManifestPath) {
                    [xml]$Manifest = Get-Content $ManifestPath
                    $AppNode = $Manifest.Package.Applications.Application
                    $Exec = $AppNode.Executable | Select-Object -First 1
                    if ($Exec) {
                        $Path = Join-Path $Pkg.InstallLocation $Exec
                        if (Test-Path $Path) {
                            # Extract Icon from Assets or Exe
                            $LogoRelative = $AppNode.VisualElements.Square44x44Logo
                            if (-not $LogoRelative) { $LogoRelative = $AppNode.VisualElements.Logo }
                            $IconBase64 = ""
                            if ($LogoRelative) {
                                $LogoPath = Join-Path $Pkg.InstallLocation $LogoRelative
                                if (-not (Test-Path $LogoPath)) {
                                    $LogoDir = [System.IO.Path]::GetDirectoryName($LogoPath); $LogoName = [System.IO.Path]::GetFileNameWithoutExtension($LogoPath)
                                    if (Test-Path $LogoDir) { $LogoPath = (Get-ChildItem -Path $LogoDir -Filter "$LogoName*.png" -ErrorAction SilentlyContinue | Sort-Object Length -Descending | Select-Object -First 1).FullName }
                                }
                                $IconBase64 = Get-IconBase64 $LogoPath
                            }
                            if (-not $IconBase64) { $IconBase64 = Get-IconBase64 $Path }
                            
                            $Apps[$DisplayName] = @{ Name = $DisplayName; Path = $Path; Icon = $IconBase64 }
                        }
                    }
                }
            } catch {}
        }

        # Final Formatting
        $Results = @()
        foreach ($App in $Apps.Values) {
            $skip = $false
            foreach ($pattern in $ExcludePatterns) { if ($App.Name -like $pattern) { $skip = $true; break } }
            if (-not $skip) { $Results += [PSCustomObject]$App }
        }

        if ($Results.Count -eq 0) { Write-Output "[]" } 
        else { $Results | Sort-Object Name | ConvertTo-Json -Compress }
    "#;

    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    if let Ok(out) = output {
        let json = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if json.is_empty() || json == "[]" { return Vec::new(); }
        
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
