import { useState, useEffect } from "react";
import { AppConfig, AuthMode } from "../types";
import { updateSettings } from "../services/config.service";

const DEFAULT_CONFIG: AppConfig = {
  locked_apps: [],
  auth_mode: "PIN",
  attempt_limit: 5,
  lockout_duration: 60,
};

interface UseConfigResult {
  config: AppConfig;
  setConfig: React.Dispatch<React.SetStateAction<AppConfig>>;
  authMode: AuthMode;
  setAuthMode: React.Dispatch<React.SetStateAction<AuthMode>>;
  updateConfig: (updates: Partial<AppConfig>) => Promise<void>;
}

export function useConfig(
  setError?: (err: string | null) => void
): UseConfigResult {
  const [config, setConfig] = useState<AppConfig>(DEFAULT_CONFIG);
  const [authMode, setAuthMode] = useState<AuthMode>("PIN");

  useEffect(() => {
    if (config.animations_intensity === "low") {
      document.documentElement.setAttribute("data-reduced-motion", "true");
    } else {
      document.documentElement.removeAttribute("data-reduced-motion");
    }
  }, [config.animations_intensity]);

  const updateConfig = async (updates: Partial<AppConfig>) => {
    const newConfig = { ...config, ...updates };
    setConfig(newConfig);
    if (updates.auth_mode) setAuthMode(updates.auth_mode);
    try {
      await updateSettings(newConfig);
    } catch (err) {
      if (setError) setError(String(err));
    }
  };

  return { config, setConfig, authMode, setAuthMode, updateConfig };
}
