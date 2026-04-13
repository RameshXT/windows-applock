import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

export interface GraceSettings {
  enabled: boolean;
  default_duration_secs: number;
  per_app_overrides: Record<string, number>;
  max_security_mode: boolean;
}

export interface GraceSessionView {
  app_id: string;
  app_name: string;
  seconds_remaining: number;
  grace_duration_secs: number;
  is_active: boolean;
}

export type GraceCheckResult = 
  | { type: 'Active', data: { seconds_remaining: number } }
  | { type: 'Expired' }
  | { type: 'NotFound' }
  | { type: 'Disabled' };

export const graceSessionService = {
  checkGraceSession: async (appId: string): Promise<GraceCheckResult> => {
    return await invoke("check_grace_session", { appId });
  },

  getAllGraceSessions: async (): Promise<GraceSessionView[]> => {
    return await invoke("get_all_grace_sessions");
  },

  reLockApp: async (appId: string): Promise<void> => {
    return await invoke("re_lock_app", { appId });
  },

  reLockAll: async (): Promise<void> => {
    return await invoke("re_lock_all");
  },

  getGraceSettings: async (): Promise<GraceSettings> => {
    return await invoke("get_grace_settings");
  },

  updateGraceSettings: async (settings: GraceSettings): Promise<void> => {
    return await invoke("update_grace_settings", { settings });
  },

  setMaxSecurityMode: async (enabled: boolean): Promise<void> => {
    return await invoke("set_max_security_mode", { enabled });
  },

  getMaxSecurityMode: async (): Promise<boolean> => {
    return await invoke("get_max_security_mode");
  },

  onGraceStarted: async (callback: (payload: any) => void): Promise<UnlistenFn> => {
    return await listen("grace_started", (event) => callback(event.payload));
  },

  onGraceExpired: async (callback: (payload: any) => void): Promise<UnlistenFn> => {
    return await listen("grace_expired", (event) => callback(event.payload));
  },

  onGraceBypassUsed: async (callback: (payload: any) => void): Promise<UnlistenFn> => {
    return await listen("grace_bypass_used", (event) => callback(event.payload));
  },

  onGraceSessionCleared: async (callback: (payload: any) => void): Promise<UnlistenFn> => {
    return await listen("grace_session_cleared", (event) => callback(event.payload));
  },

  onAllGraceSessionsReset: async (callback: (payload: any) => void): Promise<UnlistenFn> => {
    return await listen("all_grace_sessions_reset", (event) => callback(event.payload));
  },

  onSystemResumed: async (callback: () => void): Promise<UnlistenFn> => {
    return await listen("system_resumed", () => callback());
  },

  onMaxSecurityModeChanged: async (callback: (payload: { enabled: boolean }) => void): Promise<UnlistenFn> => {
    return await listen("max_security_mode_changed", (event) => callback(event.payload as { enabled: boolean }));
  }
};
