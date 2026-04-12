import { invoke } from "@tauri-apps/api/core";
import { AppConfig } from "../types";

export async function updateSettings(newConfig: AppConfig): Promise<void> {
  return invoke("update_settings", { newConfig });
}

export async function resetApp(): Promise<void> {
  return invoke("reset_app");
}
