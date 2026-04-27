import React from "react";
import styles from "../../styles/App.module.css";
import clsx from "clsx";
import { User, ShieldCheck, Monitor, RotateCcw, Star } from "lucide-react";
import { GithubIcon } from "../GithubIcon";
import { AppConfig, AuthMode } from "../../types";
import { AccountSetup } from "./AccountSetup";
import { SecurityPolicy } from "./SecurityPolicy";
import { SystemStyle } from "./SystemStyle";
import { Contribution } from "./Contribution";
import { Credits } from "./Credits";
import { UpdatesSection } from "./UpdatesSection";

interface SettingsPageProps {
  appName: string;
  config: AppConfig;
  updateConfig: (updates: Partial<AppConfig>) => void;
  settingsTab: string;
  setSettingsTab: (tab: string) => void;
  authMode: AuthMode;
  setAuthMode: (mode: AuthMode) => void;
  setView: (view: any) => void;
  setIsUpdatingFromSettings: (val: boolean) => void;
  setShowResetConfirm: (val: boolean) => void;
}

export const SettingsPage: React.FC<SettingsPageProps> = ({
  appName,
  config,
  updateConfig,
  settingsTab,
  setSettingsTab,
  authMode,
  setAuthMode,
  setView,
  setIsUpdatingFromSettings,
  setShowResetConfirm,
}) => {
  return (
    <div className={styles.settingsContainer}>
      <aside className={styles.settingsSidebar}>
        <button
          className={clsx(
            styles.settingsNavBtn,
            settingsTab === "account" && styles.settingsNavBtnActive
          )}
          onClick={() => setSettingsTab("account")}
        >
          <User size={18} /> Account & Setup
        </button>
        <button
          className={clsx(
            styles.settingsNavBtn,
            settingsTab === "security" && styles.settingsNavBtnActive
          )}
          onClick={() => setSettingsTab("security")}
        >
          <ShieldCheck size={18} color="#888" /> Security Policy
        </button>
        <button
          className={clsx(
            styles.settingsNavBtn,
            settingsTab === "system" && styles.settingsNavBtnActive
          )}
          onClick={() => setSettingsTab("system")}
        >
          <Monitor size={18} /> System & Style
        </button>
        <button
          className={clsx(
            styles.settingsNavBtn,
            settingsTab === "contribution" && styles.settingsNavBtnActive
          )}
          onClick={() => setSettingsTab("contribution")}
        >
          <GithubIcon size={18} /> Contribution
        </button>
        <button
          className={clsx(
            styles.settingsNavBtn,
            settingsTab === "credits" && styles.settingsNavBtnActive
          )}
          onClick={() => setSettingsTab("credits")}
        >
          <Star size={18} /> Credits
        </button>
        <button
          className={clsx(
            styles.settingsNavBtn,
            settingsTab === "updates" && styles.settingsNavBtnActive
          )}
          onClick={() => setSettingsTab("updates")}
        >
          <RotateCcw size={18} /> Updates
        </button>
        <div style={{ flex: 1 }} />
        <button
          className={styles.dangerBtnMinimal}
          onClick={() => setShowResetConfirm(true)}
        >
          <RotateCcw size={18} /> Factory Reset
        </button>
      </aside>

      <div className={styles.settingsContent}>
        {settingsTab === "account" && (
          <AccountSetup
            config={config}
            updateConfig={updateConfig}
            authMode={authMode}
            setAuthMode={setAuthMode}
            setView={setView}
            setIsUpdatingFromSettings={setIsUpdatingFromSettings}
            appName={appName}
          />
        )}

        {settingsTab === "security" && (
          <SecurityPolicy
            config={config}
            updateConfig={updateConfig}
            appName={appName}
          />
        )}

        {settingsTab === "system" && (
          <SystemStyle
            config={config}
            updateConfig={updateConfig}
            appName={appName}
          />
        )}


        {settingsTab === "contribution" && <Contribution appName={appName} />}

        {settingsTab === "credits" && <Credits appName={appName} />}

        {settingsTab === "updates" && <UpdatesSection appName={appName} />}
      </div>
    </div>
  );
};
