export type View = "onboarding" | "setup" | "unlock" | "dashboard" | "gatekeeper" | "verify";
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
  theme?: "dark" | "light";
  wrong_attempts?: number;
  lockout_until?: number;
}
