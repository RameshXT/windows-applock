import { invoke } from "@tauri-apps/api/core";
import { InstalledApp, LockedApp } from "../types";

export async function getDetailedApps(): Promise<InstalledApp[]> {
  return invoke<InstalledApp[]>("get_detailed_apps");
}

export async function getLockedApps(): Promise<LockedApp[]> {
  return invoke<LockedApp[]>("get_apps");
}

export async function saveSelection(apps: LockedApp[]): Promise<void> {
  return invoke("save_selection", { apps });
}
