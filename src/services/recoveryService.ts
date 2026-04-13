import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface HardLockStatus {
  is_locked: boolean;
  locked_at: string | null;
  app_id: string;
}

export interface RecoveryResult {
  success: boolean;
  access_restored: boolean;
  failure_reason: string | null;
}

export interface ResetVerifyResult {
  verified: boolean;
  reset_token: string | null;
  failure_reason: string | null;
}

export interface ResetResult {
  success: boolean;
  files_deleted: string[];
  registry_cleared: boolean;
  errors: string[];
}

class RecoveryService {
  async getHardLockStatus(appId: string): Promise<HardLockStatus> {
    return await invoke<HardLockStatus>("get_hard_lock_status", { appId });
  }

  async verifyRecoveryKey(input: string, appId: string): Promise<RecoveryResult> {
    return await invoke<RecoveryResult>("verify_recovery_key", { input, appId });
  }

  async initiateFullReset(method: "credential" | "recovery_key", input: string): Promise<ResetVerifyResult> {
    return await invoke<ResetVerifyResult>("initiate_full_reset", { method, input });
  }

  async performFullReset(token: string): Promise<ResetResult> {
    return await invoke<ResetResult>("perform_full_reset", { token });
  }

  async storeRecoveryKeyHash(rawKey: string): Promise<void> {
    return await invoke<void>("store_recovery_key_hash", { rawKey });
  }

  // Event listeners
  onHardLockActive(callback: (payload: { app_id: string; locked_at: string }) => void) {
    return listen<{ app_id: string; locked_at: string }>("hard_lock_active", (event) => {
      callback(event.payload);
    });
  }

  onRecoveryKeyVerified(callback: (payload: { app_id: string; success: boolean }) => void) {
    return listen<{ app_id: string; success: boolean }>("recovery_key_verified", (event) => {
      callback(event.payload);
    });
  }

  onAccessRestored(callback: (payload: { app_id: string; restored_at: string }) => void) {
    return listen<{ app_id: string; restored_at: string }>("access_restored_via_recovery", (event) => {
      callback(event.payload);
    });
  }

  onFullResetComplete(callback: (payload: { files_deleted: number; restarting_onboarding: boolean }) => void) {
    return listen<{ files_deleted: number; restarting_onboarding: boolean }>("full_reset_complete", (event) => {
      callback(event.payload);
    });
  }
}

export const recoveryService = new RecoveryService();
