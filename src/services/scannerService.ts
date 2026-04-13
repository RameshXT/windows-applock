import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

export interface ScannedApp {
    id: String;
    name: String;
    executable_path: String;
    icon_path: String;
    version: String;
    publisher: String;
    source: "RegistryHKLM" | "RegistryHKCU" | "ProgramFiles" | "ProgramFilesX86" | "AppDataLocal" | "AppDataRoaming" | "StartMenu";
}

export type ScanStatus = 
    | { status: "Idle" }
    | { status: "Scanning", data: { percent: number, current_source: string } }
    | { status: "Complete" }
    | { status: "Failed", data: { reason: string } };

export interface ScanProgress {
    percent: number;
    current_source: string;
}

export interface ScanComplete {
    app_count: number;
    scan_duration_ms: number;
}

/**
 * Starts a full application scan in the background.
 * Returns a unique scan_id.
 */
export async function startAppScan(): Promise<string> {
    return invoke<string>("start_app_scan");
}

/**
 * Fetches the last completed scan results from the cache.
 */
export async function getScanResults(): Promise<ScannedApp[]> {
    return invoke<ScannedApp[]>("get_scan_results");
}

/**
 * Forces a fresh rescan, invalidating the current cache.
 */
export async function refreshScan(): Promise<string> {
    return invoke<string>("refresh_scan");
}

/**
 * Starts the real-time file watcher to detect new installations.
 */
export async function startFileWatcher(): Promise<void> {
    return invoke("start_file_watcher");
}

/**
 * Stops the file watcher.
 */
export async function stopFileWatcher(): Promise<void> {
    return invoke("stop_file_watcher");
}

/**
 * Listeners for real-time scanning events.
 */
export class ScannerEvents {
    static onProgress(callback: (p: ScanProgress) => void): Promise<UnlistenFn> {
        return listen<ScanProgress>("scan_progress", (event) => callback(event.payload));
    }

    static onComplete(callback: (c: ScanComplete) => void): Promise<UnlistenFn> {
        return listen<ScanComplete>("scan_complete", (event) => callback(event.payload));
    }

    static onNewAppDetected(callback: (app: ScannedApp) => void): Promise<UnlistenFn> {
        return listen<ScannedApp>("new_app_detected", (event) => callback(event.payload));
    }

    static onFailed(callback: (reason: string) => void): Promise<UnlistenFn> {
        return listen<{ reason: string }>("scan_failed", (event) => callback(event.payload.reason));
    }
}
