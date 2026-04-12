export type View =
  | "onboarding"
  | "setup"
  | "unlock"
  | "dashboard"
  | "gatekeeper"
  | "verify";
export type AuthMode = "Password" | "PIN";
export type Tab = "home" | "all" | "system" | "settings";

export interface InstalledApp {
  name: string;
  path: string | null;
  icon?: string | null;
}

export interface LockedApp {
  id: string;
  name: string;
  exec_name: string;
  icon?: string | null;
}

export interface AppConfig {
  hashed_password?: string;
  locked_apps: LockedApp[];
  auth_mode?: AuthMode;
  attempt_limit?: number;
  lockout_duration?: number;
  autostart?: boolean;
  minimize_to_tray?: boolean;
  stealth_mode?: boolean;
  notifications_enabled?: boolean;
  animations_intensity?: "high" | "low";
  autolock_on_sleep?: boolean;
  auto_lock_duration?: number;
  panic_key?: string;
  grace_period?: number;
  strict_enforcement?: boolean;
  immediate_relock?: boolean;
  protection_persistence?: boolean;
  wrong_attempts?: number;
  lockout_until?: number;
  recovery_hint?: string;
  display_name?: string;
  profile_picture?: string;
  biometrics_enabled?: boolean;
  last_credential_change?: number;
  recovery_key?: string;
}
