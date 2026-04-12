import { useState, Dispatch, SetStateAction } from "react";
import { View, AuthMode, AppConfig } from "../types";
import {
  setupPassword,
  verifyPassword,
  verifyGatekeeper,
  getConfig,
} from "../services/auth.service";

interface SetupContext {
  isUpdating: boolean;
  setView: Dispatch<SetStateAction<View | null>>;
  setIsUpdatingFromSettings: Dispatch<SetStateAction<boolean>>;
  setShowUpdateSuccess: Dispatch<SetStateAction<boolean>>;
}

interface UseAuthResult {
  password: string;
  setPassword: Dispatch<SetStateAction<string>>;
  confirmPassword: string;
  setConfirmPassword: Dispatch<SetStateAction<string>>;
  gatekeeperPIN: string;
  setGatekeeperPIN: Dispatch<SetStateAction<string>>;
  error: string | null;
  setError: Dispatch<SetStateAction<string | null>>;
  isCompleting: boolean;
  completingStep: number;
  handleSetup: (
    e: React.FormEvent,
    authMode: AuthMode,
    ctx: SetupContext
  ) => Promise<void>;
  handleUnlock: (
    e: React.FormEvent,
    view: View | null,
    setView: Dispatch<SetStateAction<View | null>>,
    override?: string
  ) => Promise<void>;
  handleGatekeeperUnlock: (
    e: React.FormEvent,
    blockedApp: unknown,
    setConfig: Dispatch<SetStateAction<AppConfig>>,
    override?: string
  ) => Promise<void>;
}

export function useAuth(): UseAuthResult {
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [gatekeeperPIN, setGatekeeperPIN] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isCompleting, setIsCompleting] = useState(false);
  const [completingStep, setCompletingStep] = useState(0);

  const handleSetup = async (
    e: React.FormEvent,
    authMode: AuthMode,
    ctx: SetupContext
  ) => {
    e.preventDefault();
    if (authMode === "PIN" && !/^\d{4}$/.test(password)) {
      setError("PIN must be exactly 4 digits");
      return;
    }
    if (password !== confirmPassword) {
      setError("Passwords do not match");
      return;
    }
    try {
      await setupPassword(password, authMode);
      setError(null);
      if (ctx.isUpdating) {
        ctx.setShowUpdateSuccess(true);
        ctx.setView("dashboard");
        ctx.setIsUpdatingFromSettings(false);
        setTimeout(() => ctx.setShowUpdateSuccess(false), 3000);
      } else {
        setIsCompleting(true);
        const messages = [
          "Great! You're all set.",
          "We're preparing your perimeter..",
          "One moment..",
          "Here we go!",
        ];
        for (let i = 0; i < messages.length; i++) {
          setCompletingStep(i);
          await new Promise((r) => setTimeout(r, 1400));
        }
        ctx.setView("dashboard");
        setIsCompleting(false);
      }
    } catch (err) {
      setError(String(err));
    }
  };

  const handleUnlock = async (
    e: React.FormEvent,
    view: View | null,
    setView: Dispatch<SetStateAction<View | null>>,
    override?: string
  ) => {
    if (e) e.preventDefault();
    const passwordToVerify = override || password;
    try {
      const isValid = await verifyPassword(passwordToVerify);
      if (isValid) {
        setView(view === "verify" ? "setup" : "dashboard");
        setError(null);
        setPassword("");
      } else {
        setError("Invalid security credentials");
        setPassword("");
      }
    } catch (err) {
      setError(String(err));
    }
  };

  const handleGatekeeperUnlock = async (
    e: React.FormEvent,
    blockedApp: unknown,
    setConfig: Dispatch<SetStateAction<AppConfig>>,
    override?: string
  ) => {
    if (e) e.preventDefault();
    const pinToVerify = override || gatekeeperPIN;
    if (!blockedApp) return;
    try {
      const isValid = await verifyGatekeeper(pinToVerify);
      if (isValid) {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        getCurrentWindow().close();
      } else {
        const cfg = await getConfig();
        setConfig(cfg);
        setError("Invalid security credentials");
        setGatekeeperPIN("");
      }
    } catch (err) {
      setError(String(err).replace("Error: ", ""));
      setGatekeeperPIN("");
      const cfg = await getConfig().catch(() => null);
      if (cfg) setConfig(cfg);
    }
  };

  return {
    password,
    setPassword,
    confirmPassword,
    setConfirmPassword,
    gatekeeperPIN,
    setGatekeeperPIN,
    error,
    setError,
    isCompleting,
    completingStep,
    handleSetup,
    handleUnlock,
    handleGatekeeperUnlock,
  };
}
