import React from "react";
import { AppConfig } from "../../types";
import styles from "../../styles/App.module.css";
import clsx from "clsx";

interface SystemStyleProps {
  config: AppConfig;
  updateConfig: (updates: Partial<AppConfig>) => void;
  appName: string;
}

export const SystemStyle: React.FC<SystemStyleProps> = ({
  config,
  updateConfig,
  appName,
}) => {
  return (
    <section className={styles.settingsGroup}>
      <div className={styles.settingsHeader}>
        <h2>System & Style</h2>
        <p>Personalize your workspace and {appName} behavior.</p>
      </div>
      <div className={styles.settingRow}>
        <div className={styles.settingLabel}>
          <span>Launch at Startup</span>
          <span>Automatically wake {appName} when Windows starts.</span>
        </div>
        <div className={styles.settingControl}>
          <div className={styles.pillSwitch}>
            <button
              className={clsx(
                styles.pillSwitchBtn,
                config.autostart && styles.pillSwitchBtnActive
              )}
              onClick={() => updateConfig({ autostart: true })}
            >
              ON
            </button>
            <button
              className={clsx(
                styles.pillSwitchBtn,
                !config.autostart && styles.pillSwitchBtnActive
              )}
              onClick={() => updateConfig({ autostart: false })}
            >
              OFF
            </button>
          </div>
        </div>
      </div>

      <div className={styles.settingRow}>
        <div className={styles.settingLabel}>
          <span>Background Behavior</span>
          <span>Manage how {appName} stays active in your system.</span>
        </div>
        <div className={styles.settingControl}>
          <div className={styles.stackCheck}>
            <button
              className={clsx(
                styles.checkBtn,
                config.minimize_to_tray && styles.checkBtnActive
              )}
              onClick={() =>
                updateConfig({ minimize_to_tray: !config.minimize_to_tray })
              }
            >
              {config.minimize_to_tray ? "Minimize to Tray" : "Standard Exit"}
            </button>
            <button
              className={clsx(
                styles.checkBtn,
                config.stealth_mode && styles.checkBtnActive
              )}
              onClick={() =>
                updateConfig({ stealth_mode: !config.stealth_mode })
              }
            >
              {config.stealth_mode ? "Taskbar Hidden" : "Taskbar Visible"}
            </button>
          </div>
        </div>
      </div>

      <div className={styles.settingRow}>
        <div className={styles.settingLabel}>
          <span>Experience Quality</span>
          <span>Optimize responsiveness and interaction feel.</span>
        </div>
        <div className={styles.settingControl}>
          <div className={styles.pillSwitch}>
            <button
              className={clsx(
                styles.pillSwitchBtn,
                (config.animations_intensity === "high" ||
                  !config.animations_intensity) &&
                styles.pillSwitchBtnActive
              )}
              onClick={() => updateConfig({ animations_intensity: "high" })}
            >
              High
            </button>
            <button
              className={clsx(
                styles.pillSwitchBtn,
                config.animations_intensity === "low" &&
                styles.pillSwitchBtnActive
              )}
              onClick={() => updateConfig({ animations_intensity: "low" })}
            >
              Low
            </button>
          </div>
        </div>
      </div>

      <div className={styles.settingRow}>
        <div className={styles.settingLabel}>
          <span>Automated Triggers</span>
          <span>Automate security triggers based on system state.</span>
        </div>
        <div className={styles.settingControl}>
          <div className={styles.stackCheck}>
            <button
              className={clsx(
                styles.checkBtn,
                config.autolock_on_sleep && styles.checkBtnActive
              )}
              onClick={() =>
                updateConfig({ autolock_on_sleep: !config.autolock_on_sleep })
              }
            >
              {config.autolock_on_sleep ? "Auto-Lock on Sleep" : "Ignore Sleep"}
            </button>
            <button
              className={clsx(
                styles.checkBtn,
                (config.notifications_enabled ||
                  config.notifications_enabled === undefined) &&
                styles.checkBtnActive
              )}
              onClick={() =>
                updateConfig({
                  notifications_enabled: !config.notifications_enabled,
                })
              }
            >
              {config.notifications_enabled ||
                config.notifications_enabled === undefined
                ? "Notifications On"
                : "Notifications Off"}
            </button>
          </div>
        </div>
      </div>
    </section>
  );
};
