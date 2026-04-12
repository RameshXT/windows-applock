import React from "react";
import styles from "../../styles/App.module.css";
import clsx from "clsx";
import { User, ShieldCheck, Monitor, RotateCcw } from "lucide-react";
import { GithubIcon } from "../GithubIcon";
import { AppConfig, AuthMode } from "../../types";
import { AccountSetup } from "./AccountSetup";
import { SecurityPolicy } from "./SecurityPolicy";
import { SystemStyle } from "./SystemStyle";
import { Contribution } from "./Contribution";

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
  setShowResetConfirm
}) => {
  return (
    <div className={styles.settingsContainer}>
      <aside className={styles.settingsSidebar}>
        <button 
          className={clsx(styles.settingsNavBtn, settingsTab === "account" && styles.settingsNavBtnActive)} 
          onClick={() => setSettingsTab("account")}
        >
          <User size={18} /> Account & Setup
        </button>
        <button 
          className={clsx(styles.settingsNavBtn, settingsTab === "security" && styles.settingsNavBtnActive)} 
          onClick={() => setSettingsTab("security")}
        >
          <ShieldCheck size={18} color="#888" /> Security Policy
        </button>
        <button 
          className={clsx(styles.settingsNavBtn, settingsTab === "system" && styles.settingsNavBtnActive)} 
          onClick={() => setSettingsTab("system")}
        >
          <Monitor size={18} /> System & Style
        </button>
        <button 
          className={clsx(styles.settingsNavBtn, settingsTab === "contribution" && styles.settingsNavBtnActive)} 
          onClick={() => setSettingsTab("contribution")}
        >
          <GithubIcon size={18} /> Contribution
        </button>
        <div style={{ flex: 1 }} />
        <button className={styles.dangerBtnMinimal} onClick={() => setShowResetConfirm(true)}>
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
          <SecurityPolicy config={config} updateConfig={updateConfig} appName={appName} />
        )}

        {settingsTab === "system" && (
          <SystemStyle config={config} updateConfig={updateConfig} appName={appName} />
        )}

        {settingsTab === "contribution" && (
          <Contribution appName={appName} />
        )}

        <footer className={styles.settingsFooter}>
          <div style={{ flex: 1, display: 'flex', alignItems: 'baseline', gap: '0.6rem' }}>
            <span style={{ fontSize: '0.85rem', fontWeight: 600, color: '#fff', opacity: 0.8, letterSpacing: '-0.01em' }}>{appName}</span>
            <span style={{ 
              display: 'inline-flex',
              alignItems: 'center',
              justifyContent: 'center',
              fontSize: '0.6rem', 
              fontWeight: 700, 
              padding: '1px 6px',
              background: 'rgba(255,255,255,0.03)', 
              border: '1px solid rgba(255,255,255,0.08)',
              borderRadius: '4px',
              color: 'var(--accent-color)',
              textTransform: 'lowercase',
              transform: 'translateY(-1px)'
            }}>v1.0.4</span>
          </div>
          <div style={{ flex: 1, display: 'flex', justifyContent: 'center', alignItems: 'center', gap: '0.5rem' }}>
            <ShieldCheck size={12} color="#888" />
            <span style={{ fontSize: '0.6rem', fontWeight: 700, letterSpacing: '0.1em', color: '#fff', opacity: 0.3 }}>VERIFIED</span>
          </div>
          <div style={{ flex: 1, display: 'flex', justifyContent: 'flex-end', alignItems: 'center', gap: '0.4rem', fontSize: '0.7rem' }}>
            <span style={{ opacity: 0.4 }}>Designed & Developed by</span>
            <a href="https://rameshxt.pages.dev/" target="_blank" rel="noopener noreferrer" className={styles.developerLink}>Ramesh XT</a>
          </div>
        </footer>
      </div>
    </div>
  );
};
