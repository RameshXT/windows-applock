import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

export interface Rect {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

export interface MonitorInfo {
  handle: number;
  work_area: Rect;
  full_rect: Rect;
  is_primary: boolean;
  dpi: number;
}

export interface WindowSnapshotView {
  app_id: string;
  process_id: number;
  original_x: number;
  original_y: number;
  original_width: number;
  original_height: number;
  was_fullscreen: boolean;
}

export interface KillProtectionStatus {
  enabled: boolean;
  method: "dacl" | "watchdog" | "none";
}

export type FreezeResult = 
  | { Success: null }
  | { PartialSuccess: { reason: string } }
  | { Failed: { reason: string } };

export class WindowManagerService {
  /**
   * Freeze a target application's windows.
   */
  static async freezeAppWindow(processId: number, appId: string): Promise<FreezeResult> {
    return await invoke("freeze_app_window", { processId, appId });
  }

  /**
   * Restore a target application's windows.
   */
  static async restoreAppWindow(processId: number): Promise<boolean> {
    return await invoke("restore_app_window", { processId });
  }

  /**
   * Get the current monitor layout.
   */
  static async getMonitorLayout(): Promise<MonitorInfo[]> {
    return await invoke("get_monitor_layout");
  }

  /**
   * Get a snapshot of a locked window.
   */
  static async getWindowSnapshot(appId: string): Promise<WindowSnapshotView> {
    return await invoke("get_window_snapshot", { appId });
  }

  /**
   * Get the status of kill protection.
   */
  static async getKillProtectionStatus(): Promise<KillProtectionStatus> {
    return await invoke("get_kill_protection_status");
  }

  /**
   * Start the low-level keyboard hook.
   */
  static async startInputBlocker(): Promise<void> {
    return await invoke("start_input_blocker");
  }

  /**
   * Stop the low-level keyboard hook.
   */
  static async stopInputBlocker(): Promise<void> {
    return await invoke("stop_input_blocker");
  }

  /**
   * Reposition the overlay to a specific monitor.
   */
  static async repositionOverlay(monitorHandle: number): Promise<void> {
    return await invoke("reposition_overlay", { monitorHandle });
  }

  // Event listeners

  static async onWindowFrozen(callback: (payload: any) => void): Promise<UnlistenFn> {
    return await listen("window_frozen", (event) => callback(event.payload));
  }

  static async onAppRestored(callback: (payload: any) => void): Promise<UnlistenFn> {
    return await listen("app_restored", (event) => callback(event.payload));
  }

  static async onLockOverlayPositioned(callback: (payload: any) => void): Promise<UnlistenFn> {
    return await listen("lock_overlay_positioned", (event) => callback(event.payload));
  }

  static async onBypassAttemptBlocked(callback: (payload: any) => void): Promise<UnlistenFn> {
    return await listen("bypass_attempt_blocked", (event) => callback(event.payload));
  }

  static async onFullscreenAppLocked(callback: (payload: any) => void): Promise<UnlistenFn> {
    return await listen("fullscreen_app_locked", (event) => callback(event.payload));
  }

  static async onAppExitedDuringLock(callback: (payload: any) => void): Promise<UnlistenFn> {
    return await listen("app_exited_during_lock", (event) => callback(event.payload));
  }

  static async onMonitorConfigChanged(callback: (payload: any) => void): Promise<UnlistenFn> {
    return await listen("monitor_config_changed", (event) => callback(event.payload));
  }
}
