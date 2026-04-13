import React from "react";
import { AppConfig } from "../../types";
import styles from "../../styles/App.module.css";
import clsx from "clsx";
import { ModernSelect } from "../ModernSelect";
import { graceSessionService, GraceSettings } from "../../services/graceSessionService";
import { useEffect, useState } from "react";

const GraceSettingsControl: React.FC = () => {
    const [settings, setSettings] = useState<GraceSettings | null>(null);

    useEffect(() => {
        graceSessionService.getGraceSettings().then(setSettings);
    }, []);

    if (!settings) return null;

    const update = async (updates: Partial<GraceSettings>) => {
        const newSettings = { ...settings, ...updates };
        setSettings(newSettings);
        await graceSessionService.updateGraceSettings(newSettings);
    };

    return (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', alignItems: 'flex-end' }}>
            <div style={{ display: 'flex', gap: '0.75rem' }}>
                <ModernSelect 
                    value={String(settings.default_duration_secs)}
                    onChange={(val) => update({ default_duration_secs: parseInt(val) })}
                    options={[
                        { label: "Off", value: "0" },
                        { label: "30s", value: "30" },
                        { label: "5m", value: "300" },
                        { label: "15m", value: "900" },
                        { label: "60m", value: "3600" },
                    ]}
                />
                <div className={styles.pillSwitch}>
                    <button
                        className={clsx(styles.pillSwitchBtn, settings.max_security_mode && styles.pillSwitchBtnActive)}
                        onClick={() => update({ max_security_mode: true })}
                        title="Maximum Security Mode"
                    >
                        MAX
                    </button>
                    <button
                        className={clsx(styles.pillSwitchBtn, !settings.max_security_mode && styles.pillSwitchBtnActive)}
                        onClick={() => update({ max_security_mode: false })}
                    >
                        STD
                    </button>
                </div>
            </div>
            {settings.max_security_mode && (
                <span style={{ fontSize: '10px', color: 'var(--accent-color)', fontWeight: 'bold' }}>
                    MAX SECURITY ACTIVE
                </span>
            )}
        </div>
    );
};

interface SecurityPolicyProps {
  config: AppConfig;
  updateConfig: (updates: Partial<AppConfig>) => void;
  appName: string;
}

export const SecurityPolicy: React.FC<SecurityPolicyProps> = ({
  config,
  updateConfig,
  appName,
}) => {
  return (
    <section className={styles.settingsGroup}>
      <div className={styles.settingsHeader}>
        <h2>Security Policy</h2>
        <p>Configure how {appName} responds to intrusions.</p>
      </div>
      <div className={styles.settingRow}>
        <div className={styles.settingLabel}>
          <span>Idle Lock</span>
          <span>Lock the dashboard automatically if you are away.</span>
        </div>
        <div className={styles.settingControl}>
          <ModernSelect
            value={String(config.auto_lock_duration || 0)}
            onChange={(val) =>
              updateConfig({ auto_lock_duration: parseInt(val) })
            }
            options={[
              { label: "Never", value: "0" },
              { label: "5 Minutes", value: "5" },
              { label: "15 Minutes", value: "15" },
              { label: "30 Minutes", value: "30" },
              { label: "1 Hour", value: "60" },
            ]}
          />
        </div>
      </div>

      <div className={styles.settingRow}>
        <div className={styles.settingLabel}>
          <span>Emergency Lock</span>
          <span>
            Press <code>Ctrl+Alt+L</code> to instantly lock everything.
          </span>
        </div>
        <div className={styles.settingControl}>
          <span className={styles.statusPill}>Active</span>
        </div>
      </div>

      <div className={styles.settingRow}>
        <div className={styles.settingLabel}>
          <span>Grace Period</span>
          <span>Time allowed before re-locking an app after use.</span>
        </div>
        <div className={styles.settingControl}>
          <GraceSettingsControl />
        </div>
      </div>

      <div className={styles.settingRow}>
        <div className={styles.settingLabel}>
          <span>Lock on Exit</span>
          <span>Instantly lock the app when you close it.</span>
        </div>
        <div className={styles.settingControl}>
          <div className={styles.pillSwitch}>
            <button
              className={clsx(
                styles.pillSwitchBtn,
                config.immediate_relock && styles.pillSwitchBtnActive
              )}
              onClick={() => updateConfig({ immediate_relock: true })}
            >
              ON
            </button>
            <button
              className={clsx(
                styles.pillSwitchBtn,
                !config.immediate_relock && styles.pillSwitchBtnActive
              )}
              onClick={() => updateConfig({ immediate_relock: false })}
            >
              OFF
            </button>
          </div>
        </div>
      </div>

      <div className={styles.settingRow}>
        <div className={styles.settingLabel}>
          <span>Advanced Protection</span>
          <span>Enhanced monitoring and persistence guard.</span>
        </div>
        <div className={styles.settingControl}>
          <div className={styles.pillSwitch}>
            <button
              className={clsx(
                styles.pillSwitchBtn,
                (config.strict_enforcement || config.protection_persistence) &&
                styles.pillSwitchBtnActive
              )}
              onClick={() =>
                updateConfig({
                  strict_enforcement: true,
                  protection_persistence: true,
                })
              }
            >
              ON
            </button>
            <button
              className={clsx(
                styles.pillSwitchBtn,
                !(config.strict_enforcement || config.protection_persistence) &&
                styles.pillSwitchBtnActive
              )}
              onClick={() =>
                updateConfig({
                  strict_enforcement: false,
                  protection_persistence: false,
                })
              }
            >
              OFF
            </button>
          </div>
        </div>
      </div>

      <div className={styles.settingRow}>
        <div className={styles.settingLabel}>
          <span>Safety Lockout</span>
          <span>Configures automatic cooldown after failed attempts.</span>
        </div>
        <div
          className={styles.settingControl}
          style={{ display: "flex", gap: "0.75rem" }}
        >
          <ModernSelect
            value={String(config.attempt_limit)}
            onChange={(val) => updateConfig({ attempt_limit: parseInt(val) })}
            options={[
              { label: "3 Failed", value: "3" },
              { label: "5 Failed", value: "5" },
            ]}
          />
          <ModernSelect
            value={String(config.lockout_duration)}
            onChange={(val) =>
              updateConfig({ lockout_duration: parseInt(val) })
            }
            options={[
              { label: "30s Wait", value: "30" },
              { label: "1m Wait", value: "60" },
            ]}
          />
        </div>
      </div>
    </section>
  );
};
