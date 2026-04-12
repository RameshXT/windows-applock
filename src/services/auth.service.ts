import { invoke } from "@tauri-apps/api/core";
import { AppConfig, AuthMode } from "../types";

export async function verifyPassword(password: string): Promise<boolean> {
  return invoke<boolean>("verify_password", { password });
}

export async function setupPassword(
  password: string,
  mode: AuthMode
): Promise<void> {
  return invoke("setup_password", { password, mode });
}

export async function lockSession(): Promise<void> {
  return invoke("lock_session");
}

export async function verifyGatekeeper(password: string): Promise<boolean> {
  return invoke<boolean>("verify_gatekeeper", { password });
}

export async function checkSetup(): Promise<boolean> {
  return invoke<boolean>("check_setup");
}

export async function getIsUnlocked(): Promise<boolean> {
  return invoke<boolean>("get_is_unlocked");
}

export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("get_config");
}
