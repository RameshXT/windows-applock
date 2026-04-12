import { invoke } from "@tauri-apps/api/core";
import { LockedApp } from "../types";

export async function getBlockedApp(): Promise<LockedApp | null> {
  return invoke<LockedApp | null>("get_blocked_app");
}

export async function releaseApp(): Promise<void> {
  return invoke("release_app");
}
