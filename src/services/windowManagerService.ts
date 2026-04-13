import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface MonitorBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface WindowSnapshot {
  hwnd: number;
  was_fullscreen: boolean;
  placement: SerializablePlacement;
  extended_style: number;
}

export interface SerializablePlacement {
  show_cmd: number;
  pt_min_position_x: number;
  pt_min_position_y: number;
  pt_max_position_x: number;
  pt_max_position_y: number;
  rc_normal_position_left: number;
  rc_normal_position_top: number;
  rc_normal_position_right: number;
  rc_normal_position_bottom: number;
}

export class WindowManagerService {
  /**
   * Freeze target app window immediately on detection.
   */
  static async freezeTargetWindow(hwnd: number): Promise<void> {
    return await invoke("freeze_target_window", { hwnd });
  }

  /**
   * Bring lock overlay to foreground (always on top).
   */
  static async assertOverlayTopmost(): Promise<void> {
    return await invoke("assert_overlay_topmost");
  }

  /**
   * Get monitor bounds for a target application window.
   */
  static async getTargetMonitorBounds(hwnd: number): Promise<MonitorBounds> {
    return await invoke("get_target_monitor_bounds", { hwnd });
  }

  /**
   * Restore app to original state after unlock.
   */
  static async restoreLockedWindow(hwnd: number): Promise<void> {
    return await invoke("restore_locked_window", { hwnd });
  }

  /**
   * Listen for window frozen event.
   */
  static onWindowFrozen(callback: (payload: { hwnd: number }) => void) {
    return listen<{ hwnd: number }>("window_frozen", (event) => {
      callback(event.payload);
    });
  }

  /**
   * Listen for overlay asserted topmost event.
   */
  static onOverlayAssertedTopmost(callback: () => void) {
    return listen("overlay_asserted_topmost", () => {
      callback();
    });
  }

  /**
   * Listen for window restored event.
   */
  static onWindowRestored(callback: (payload: { hwnd: number; was_fullscreen: boolean }) => void) {
    return listen<{ hwnd: number; was_fullscreen: boolean }>("window_restored", (event) => {
      callback(event.payload);
    });
  }

  /**
   * Listen for keyboard hook events.
   */
  static onKeyboardHookInstalled(callback: () => void) {
    return listen("keyboard_hook_installed", () => {
      callback();
    });
  }

  static onKeyboardHookRemoved(callback: () => void) {
    return listen("keyboard_hook_removed", () => {
      callback();
    });
  }

  /**
   * Listen for process protection event.
   */
  static onProcessProtectionSet(callback: (payload: { elevated: boolean }) => void) {
    return listen<{ elevated: boolean }>("process_protection_set", (event) => {
      callback(event.payload);
    });
  }

  /**
   * Listen for full-screen app detection.
   */
  static onFullscreenAppDetected(callback: (payload: { hwnd: number }) => void) {
    return listen<{ hwnd: number }>("fullscreen_app_detected", (event) => {
      callback(event.payload);
    });
  }
}
