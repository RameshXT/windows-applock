import { useState, useEffect } from "react";
import { View, Tab, AppConfig, AuthMode, LockedApp } from "../types";
import { getConfig, checkSetup, getIsUnlocked } from "../services/auth.service";
import { getLockedApps } from "../services/apps.service";
import { getBlockedApp } from "../services/system.service";
import { listen } from "@tauri-apps/api/event";
import { STORAGE_KEYS, RESTORABLE_VIEWS } from "../constants";

interface AppInitOptions {
  setConfig: (cfg: AppConfig) => void;
  setAuthMode: (mode: AuthMode) => void;
  setLockedApps: React.Dispatch<React.SetStateAction<LockedApp[]>>;
  fetchDetailedApps: () => Promise<void>;
  setGatekeeperPIN: (pin: string) => void;
  setError: (err: string | null) => void;
  onSetView: (view: View) => void;
}

interface AppInitResult {
  blockedApp: LockedApp | null;
  activeTab: Tab;
  setActiveTab: React.Dispatch<React.SetStateAction<Tab>>;
  settingsTab: string;
  setSettingsTab: React.Dispatch<React.SetStateAction<string>>;
}

/**
 * Bootstraps the application on startup and manages global event listeners.
 * Takes external setters so it can hydrate other hooks without owning their state.
 */
export function useAppInit(options: AppInitOptions): AppInitResult {
  const [blockedApp, setBlockedApp] = useState<LockedApp | null>(null);
  const [activeTab, setActiveTab] = useState<Tab>(
    () => (localStorage.getItem(STORAGE_KEYS.TAB) as Tab) || "home"
  );
  const [settingsTab, setSettingsTab] = useState(
    () => localStorage.getItem(STORAGE_KEYS.SETTINGS_TAB) || "account"
  );

  useEffect(() => {
    const init = async () => {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        const currentWin = getCurrentWindow();

        if (currentWin.label === "gatekeeper") {
          const cfg = await getConfig();
          options.setConfig(cfg);
          if (cfg.auth_mode) options.setAuthMode(cfg.auth_mode);
          const blocked = await getBlockedApp();
          if (blocked) {
            setBlockedApp(blocked);
            options.onSetView("gatekeeper");
          } else {
            currentWin.close();
          }
          return;
        }

        const cfg = await getConfig();
        options.setConfig(cfg);
        if (cfg.auth_mode) options.setAuthMode(cfg.auth_mode);

        const isSetup = await checkSetup();
        if (!isSetup) { options.onSetView("onboarding"); return; }

        const isUnlocked = await getIsUnlocked();
        if (isUnlocked) {
          const persistedView = localStorage.getItem(STORAGE_KEYS.VIEW) as View;
          if (persistedView && (RESTORABLE_VIEWS as readonly string[]).includes(persistedView)) {
            options.onSetView(persistedView);
          } else {
            options.onSetView("dashboard");
          }
          const persistedTab = localStorage.getItem(STORAGE_KEYS.TAB) as Tab;
          const persistedSettingsTab = localStorage.getItem(STORAGE_KEYS.SETTINGS_TAB);
          if (persistedTab) setActiveTab(persistedTab);
          if (persistedSettingsTab) setSettingsTab(persistedSettingsTab);
        } else {
          options.onSetView("unlock");
        }

        const locked = await getLockedApps();
        options.setLockedApps(locked);
        options.fetchDetailedApps();
      } catch (err) {
        console.error(err);
      }
    };
    init();
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Monitor events: blocked app and full reload
  useEffect(() => {
    const unlisten = listen<LockedApp>("app-blocked", async (event) => {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const currentWin = getCurrentWindow();
      if (currentWin.label === "gatekeeper") {
        setBlockedApp(event.payload);
        options.setGatekeeperPIN("");
        options.setError(null);
        currentWin.unminimize();
        currentWin.setFocus();
      }
    });

    const unlistenReload = listen("reload-app", () => window.location.reload());

    return () => {
      unlisten.then(f => f());
      unlistenReload.then(f => f());
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return { blockedApp, activeTab, setActiveTab, settingsTab, setSettingsTab };
}
