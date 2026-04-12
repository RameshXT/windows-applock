export const APP_NAME = "Windows AppLock" as const;

export const STORAGE_KEYS = {
  VIEW: "applock_view",
  TAB: "applock_tab",
  SETTINGS_TAB: "applock_settings_tab",
} as const;

export const RESTORABLE_VIEWS = ["dashboard", "setup", "verify"] as const;
