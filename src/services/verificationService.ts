import { invoke } from "@tauri-apps/api/core";

export type VerifyContext = "app_lock" | "dashboard" | "credential_change" | "settings";

export interface VerifyResult {
  success: boolean;
}

export interface LockoutStatus {
  is_locked_out: boolean;
  seconds_remaining?: number;
}

/**
 * Verifies the user's credential against the stored Argon2 hash.
 * This is the ONLY entry point for verification from the frontend.
 * 
 * @param input The PIN or password entered by the user.
 * @param context The context where verification was requested.
 * @param appId Optional application ID if context is "app_lock".
 * @returns Promise<boolean> True if verification succeeded, false otherwise.
 */
export async function verifyCredential(
  input: string,
  context: VerifyContext,
  appId?: string
): Promise<boolean> {
  try {
    const result = await invoke<VerifyResult>("verify_credential", {
      input,
      context,
      appId: appId || null,
    });
    return result.success;
  } catch (error) {
    // Requirement 46: generic error strings returned on failure.
    // Frontend never receives internal details.
    console.error("[Verification] Generic error occurred");
    return false;
  }
}

/**
 * Gets the current lockout status from the backend.
 * Used to show cooldown UI when rate limited or locked out.
 */
export async function getLockoutStatus(): Promise<LockoutStatus> {
  try {
    return await invoke<LockoutStatus>("get_lockout_status");
  } catch (error) {
    console.error("[Verification] Failed to get lockout status");
    return { is_locked_out: false };
  }
}

/**
 * Resets the lockout state after a successful administrative recovery flow.
 * Only callable after recovery flow.
 */
export async function clearLockoutAdmin(): Promise<void> {
  try {
    await invoke("clear_lockout_admin");
  } catch (error) {
    console.error("[Verification] Failed to clear lockout");
    throw error;
  }
}
