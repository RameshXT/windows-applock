import React from "react";
import { AppConfig, AuthMode } from "../../types";
import styles from "../../styles/App.module.css";
import clsx from "clsx";

interface AccountSetupProps {
  config: AppConfig;
  updateConfig: (updates: Partial<AppConfig>) => void;
  authMode: AuthMode;
  setAuthMode: (mode: AuthMode) => void;
  setView: (view: any) => void;
  setIsUpdatingFromSettings: (val: boolean) => void;
  appName: string;
}

export const AccountSetup: React.FC<AccountSetupProps> = ({
  config,
  updateConfig,
  authMode,
  setAuthMode,
  setView,
  setIsUpdatingFromSettings,
  appName
}) => {
  return (
    <section className={styles.settingsGroup}>
      <div className={styles.settingsHeader}>
        <h2>Account & Setup</h2>
        <p>Manage your entry protocol and core identity.</p>
      </div>

      <div className={styles.settingRow}>
        <div className={styles.settingLabel}>
          <span>Display Identity</span>
          <span>Your public name shown on the lock screen.</span>
        </div>
        <div className={styles.settingControl}>
          <input 
            type="text" 
            className={styles.settingsInput} 
            style={{ maxWidth: '220px' }}
            placeholder="Display Name"
            value={config.display_name || ""}
            onChange={(e) => updateConfig({ display_name: e.target.value })}
          />
        </div>
      </div>

      <div className={styles.settingRow}>
        <div className={styles.settingLabel}>
          <span>Authentication Mode</span>
          <span>Choose between a numeric PIN or a text password.</span>
        </div>
        <div className={styles.settingControl}>
          <div className={styles.pillSwitch}>
            <button className={clsx(styles.pillSwitchBtn, authMode === "PIN" && styles.pillSwitchBtnActive)} onClick={() => setAuthMode("PIN")}>PIN</button>
            <button className={clsx(styles.pillSwitchBtn, authMode === "Password" && styles.pillSwitchBtnActive)} onClick={() => setAuthMode("Password")}>Password</button>
          </div>
        </div>
      </div>

      <div className={styles.settingRow}>
        <div className={styles.settingLabel}>
          <span>Security Credential</span>
          <span>Update your {authMode} to keep your {appName} secure.</span>
        </div>
        <div className={styles.settingControl}>
          <button className={styles.iconBtn} onClick={() => { setIsUpdatingFromSettings(true); setView("setup"); }}>Update {authMode}</button>
        </div>
      </div>

      <div className={styles.settingRow}>
        <div className={styles.settingLabel}>
          <span>Security Recovery</span>
          <span>Setup a hint or key to recover access if forgotten.</span>
        </div>
        <div className={styles.settingControl} style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', width: '220px' }}>
          <input 
            type="text" 
            className={styles.settingsInput} 
            placeholder="Recovery Hint"
            value={config.recovery_hint || ""}
            onChange={(e) => updateConfig({ recovery_hint: e.target.value })}
          />
          <button 
            className={styles.checkBtn} 
            style={{ width: '100%', fontSize: '0.7rem' }}
            onClick={() => {
              const key = Math.random().toString(36).substring(2, 10).toUpperCase() + "-" + Math.random().toString(36).substring(2, 10).toUpperCase();
              updateConfig({ recovery_key: key });
              alert(`Your Recovery Key is: ${key}\n\nPLEASE SAVE THIS SOMEWHERE SAFE!`);
            }}
          >
            {config.recovery_key ? "Regenerate Key" : "Generate Key"}
          </button>
        </div>
      </div>

      <div className={styles.settingRow}>
        <div className={styles.settingLabel}>
          <span>Biometric Login <span className={styles.comingSoonBadge}>Coming Soon</span></span>
          <span>Use Windows Hello (Fingerprint/Face) to unlock.</span>
        </div>
        <div className={styles.settingControl}>
          <div className={styles.pillSwitch} style={{ opacity: 0.5, cursor: 'not-allowed' }} title="Windows Hello support coming soon">
            <button className={clsx(styles.pillSwitchBtn)} disabled>ON</button>
            <button className={clsx(styles.pillSwitchBtn, styles.pillSwitchBtnActive)} disabled>OFF</button>
          </div>
        </div>
      </div>

      <div className={styles.settingRow}>
        <div className={styles.settingLabel}>
          <span>Data & Portability <span className={styles.comingSoonBadge}>Coming Soon</span></span>
          <span>Export or import your entire configuration.</span>
        </div>
        <div className={styles.settingControl} style={{ display: 'flex', gap: '0.75rem', opacity: 0.5 }}>
            <button className={styles.iconBtn} disabled style={{ cursor: 'not-allowed' }}>Export Config</button>
            <button className={styles.iconBtn} disabled style={{ cursor: 'not-allowed' }}>Import Config</button>
        </div>
      </div>

      <div style={{ marginTop: '1rem', padding: '1rem', background: 'rgba(255,255,255,0.02)', borderRadius: '12px', border: '1px solid rgba(255,255,255,0.05)' }}>
        <div style={{ fontSize: '0.7rem', color: 'var(--text-secondary)', display: 'flex', justifyContent: 'space-between' }}>
          <span>Security Audit</span>
          <span>Last Changed: {config.last_credential_change ? new Date(config.last_credential_change).toLocaleDateString() : "Never"}</span>
        </div>
      </div>
    </section>
  );
};
