/** Application name — single source of truth */
export const APP_NAME = "Windows AppLock" as const;

/** localStorage key constants — prevent typos across the codebase */
export const STORAGE_KEYS = {
  VIEW: "applock_view",
  TAB: "applock_tab",
  SETTINGS_TAB: "applock_settings_tab",
} as const;

/** Valid views that can be safely persisted and restored */
export const RESTORABLE_VIEWS = ["dashboard", "setup", "verify"] as const;
